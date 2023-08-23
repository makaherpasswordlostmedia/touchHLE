use plist::Value;
use std::io::Cursor;

use crate::objc::{
    id, objc_classes, Class, ClassExports, retain,
};
use crate::{Environment, msg, msg_class};
use crate::mem::MutPtr;
use crate::frameworks::foundation::{ns_data, ns_string, NSInteger};
use crate::frameworks::foundation::ns_dictionary::{DictionaryHostObject, dict_from_keys_and_objects};
use crate::frameworks::foundation::ns_string::to_rust_string;
use crate::frameworks::foundation::ns_value::NSNumberHostObject;
use crate::frameworks::foundation::NSUInteger;

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
    let ptr = env.mem.alloc_and_write_cstr(&buf[..]);
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
        _ => unimplemented!("build_plist_id_rec value {:?}", value)
    }
}

fn build_plist_value_rec(env: &mut Environment, plist: id) -> Value {
    let class: Class = msg![env; plist class];
    // TODO: check subclass instead of exact match
    return if class == env.objc.get_known_class("NSMutableDictionary", &mut env.mem) {
        let mut dict = plist::dictionary::Dictionary::new();
        let dict_host_obj: DictionaryHostObject = std::mem::take(env.objc.borrow_mut(plist));
        let mut iter = dict_host_obj.iter_keys();
        while let Some(key) = iter.next() {
            let key_class: Class = msg![env; key class];

            // only string keys are supported
            let string_class = env.objc.get_known_class("_touchHLE_NSString", &mut env.mem);
            assert!(env.objc.class_is_subclass_of(key_class, string_class));

            let key_string = to_rust_string(env, key);
            let val = dict_host_obj.lookup(env, key);
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
        }
    } else {
        unimplemented!()
    };
}