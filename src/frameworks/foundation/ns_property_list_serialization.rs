//! `NSPropertyListSerialization`.

use super::{ns_array, ns_data, ns_dictionary, ns_string, NSInteger, NSUInteger};
use crate::fs::GuestPath;
use crate::mem::MutPtr;
use crate::Environment;
use crate::objc::{id, msg, msg_class, nil, release, objc_classes, Class, ClassExports};
use std::io::Cursor;

use plist::Value;
use crate::frameworks::foundation::ns_dictionary::{DictionaryHostObject, dict_from_keys_and_objects};
use crate::frameworks::foundation::ns_string::to_rust_string;
use crate::frameworks::foundation::ns_value::NSNumberHostObject;

// TODO: Implement reading of property lists other than Info.plist.
// [NSDictionary contentsOfFile:] and [NSArray contentsOfFile:] in particular.

/// Internals of `initWithContentsOfFile:` on `NSArray` and `NSDictionary`.
/// Returns `nil` on failure.
pub(super) fn deserialize_plist_from_file(
    env: &mut Environment,
    path: &GuestPath,
    array_expected: bool,
) -> id {
    log_dbg!("Reading plist from {:?}.", path);
    let Ok(bytes) = env.fs.read(path) else {
        log_dbg!("Couldn't read file, returning nil.");
        return nil;
    };

    let Ok(root) = Value::from_reader(Cursor::new(bytes)) else {
        log_dbg!("Couldn't parse plist, returning nil.");
        return nil;
    };

    if array_expected && root.as_array().is_none() {
        log_dbg!("Plist root is not array, returning nil.");
        return nil;
    }
    if !array_expected && root.as_dictionary().is_none() {
        log_dbg!("Plist root is not dictionary, returning nil.");
        return nil;
    }

    deserialize_plist(env, &root)
}

fn deserialize_plist(env: &mut Environment, value: &Value) -> id {
    match value {
        Value::Array(array) => {
            let array = array
                .iter()
                .map(|value| deserialize_plist(env, value))
                .collect();
            ns_array::from_vec(env, array)
        }
        Value::Dictionary(dict) => {
            let pairs: Vec<_> = dict
                .iter()
                .map(|(key, value)| {
                    (
                        ns_string::from_rust_string(env, key.clone()),
                        deserialize_plist(env, value),
                    )
                })
                .collect();
            // Unlike ns_array::from_vec and ns_string::from_rust_string,
            // this will retain the keys and values!
            let ns_dict = ns_dictionary::dict_from_keys_and_objects(env, &pairs);
            // ...so they need to be released.
            for (key, value) in pairs {
                release(env, key);
                release(env, value);
            }
            ns_dict
        }
        Value::Boolean(b) => {
            let number: id = msg_class![env; NSNumber alloc];
            let b: bool = *b;
            msg![env; number initWithBool:b]
        }
        Value::Data(d) => {
            let length: NSUInteger = d.len().try_into().unwrap();
            let alloc: MutPtr<u8> = env.mem.alloc(length).cast();
            env.mem.bytes_at_mut(alloc, length).copy_from_slice(d);
            let data: id = msg_class![env; NSData alloc];
            msg![env; data initWithBytesNoCopy:alloc length:length]
        }
        Value::Date(_) => {
            todo!("deserialize plist value: {:?}", value); // TODO
        }
        Value::Integer(int) => {
            let number: id = msg_class![env; NSNumber alloc];
            // TODO: is this the correct order of preference? does it matter?
            if let Some(int64) = int.as_signed() {
                let longlong: i64 = int64;
                msg![env; number initWithLongLong:longlong]
            } else if let Some(uint64) = int.as_unsigned() {
                let ulonglong: u64 = uint64;
                msg![env; number initWithUnsignedLongLong:ulonglong]
            } else {
                unreachable!(); // according to plist crate docs
            }
        }
        Value::Real(real) => {
            let number: id = msg_class![env; NSNumber alloc];
            let double: f64 = *real;
            msg![env; number initWithDouble:double]
        }
        Value::String(s) => ns_string::from_rust_string(env, s.clone()),
        Value::Uid(_) => {
            // These are probably only used by NSKeyedUnarchiver, which does not
            // currently use this code in our implementation.
            unimplemented!("deserialize plist value: {:?}", value);
        }
        _ => {
            unreachable!() // enum is marked inexhaustive, but shouldn't be
        }
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSPropertyListSerialization: NSObject

+ (id)dataFromPropertyList:(id)plist
                    format:(i32)format
                errorDescription:(id)errorString { // NSString**
    // 200 => NSPropertyListBinaryFormat_v1_0 = kCFPropertyListBinaryFormat_v1_0
    assert_eq!(format, 200);
    log_dbg!("dataFromPropertyList format {}", format);
    let value = build_plist_value_rec(env, plist);
    let mut buf = Vec::new();
    value.to_writer_binary(&mut buf).unwrap();
    let len: u32 = buf.len().try_into().unwrap();
    log_dbg!("dataFromPropertyList buf len {}", len);
    let ptr = env.mem.alloc_and_write_cstr(&buf[..]).cast_const().cast_void();
    msg_class![env; NSData dataWithBytes:ptr length:len]
}

+ (id)propertyListFromData:(id)data // NSData*
          mutabilityOption:(NSUInteger)opt
                    format:(MutPtr<i32>)format
          errorDescription:(id)errorString { // NSString**
    // assert!(format.is_null());
    let slice = ns_data::to_rust_slice(env, data);
    let plist = Value::from_reader(Cursor::new(slice)).unwrap();
    build_plist_id_rec(env, plist)
}

@end

};

fn build_plist_id_rec(env: &mut Environment, value: Value) -> id {
    match value {
        Value::Array(arr_val) => {
            let arr = msg_class![env; NSMutableArray array];
            for v in arr_val {
                let value = build_plist_id_rec(env, v);
                () = msg![env; arr addObject:value];
            }
            arr
        }
        Value::Dictionary(dict_val) => {
            let dict = msg_class![env; NSMutableDictionary dictionary];
            for (k, v) in dict_val {
                let key = ns_string::from_rust_string(env, k);
                let value = build_plist_id_rec(env, v);
                () = msg![env; dict setValue:value forKey:key];
            }
            dict
        }
        Value::String(str_val) => {
            ns_string::from_rust_string(env, str_val)
        }
        Value::Real(real_val) => {
            // TODO: avoid downcast
            let float: f32 = real_val as f32;
            msg_class![env; NSNumber numberWithFloat:float]
        }
        Value::Integer(int_val) => {
            let int: NSInteger = int_val.as_signed().unwrap().try_into().unwrap();
            msg_class![env; NSNumber numberWithInteger:int]
        }
        Value::Boolean(bool_val) => {
            msg_class![env; NSNumber numberWithBool:bool_val]
        }
        _ => unimplemented!("build_plist_id_rec value {:?}", value)
    }
}

fn build_plist_value_rec(env: &mut Environment, plist: id) -> Value {
    if plist == nil {
        return Value::from(0);
    }
    let class: Class = msg![env; plist class];

    // TODO: check subclass instead of exact match
    return if class == env.objc.get_known_class("NSMutableDictionary", &mut env.mem) {
        let mut dict = plist::dictionary::Dictionary::new();
        let dict_host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(plist));
        let mut key_vals = Vec::with_capacity(dict_host_obj.count as usize);
        for collisions in dict_host_obj.map.values() {
            for &(key, value) in collisions {
                key_vals.push((key, value));
            }
        }
        *env.objc.borrow_mut(plist) = dict_host_obj;
        for (key, val) in key_vals {
            let key_class: Class = msg![env; key class];

            // only string keys are supported
            let string_class = env.objc.get_known_class("_touchHLE_NSString", &mut env.mem);
            assert!(env.objc.class_is_subclass_of(key_class, string_class));

            let key_string = to_rust_string(env, key);
            let val_plist = build_plist_value_rec(env, val);
            dict.insert(String::from(key_string), val_plist);
        }
        Value::Dictionary(dict)
    } else if class == env.objc.get_known_class("NSNumber", &mut env.mem) {
        let num = env.objc.borrow::<NSNumberHostObject>(plist);
        match num {
            NSNumberHostObject::Bool(b) => Value::Boolean(*b),
            NSNumberHostObject::Int(i) => Value::from(*i),
            NSNumberHostObject::Float(f) => Value::from(*f),
            _ => todo!()
        }
    } else {
        unimplemented!("{}", env.objc.get_class_name(class).to_string())
    };
}
