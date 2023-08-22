use plist::Value;
use crate::objc::{
    id, objc_classes, Class, ClassExports,
};
use crate::{Environment, msg, msg_class};
use crate::frameworks::foundation::ns_dictionary::DictionaryHostObject;
use crate::frameworks::foundation::ns_string::to_rust_string;
use crate::frameworks::foundation::ns_value::NSNumberHostObject;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSPropertyListSerialization: NSObject

+ (id)dataFromPropertyList:(id)plist
                    format:(i32)format
                errorDescription:(id)errorString {
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

@end

};

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