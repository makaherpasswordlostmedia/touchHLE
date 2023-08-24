/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The `NSValue` class cluster, including `NSNumber`.

use super::{NSInteger, NSUInteger};
use crate::objc::{
    autorelease, id, msg, msg_class, objc_classes, retain, Class, ClassExports, HostObject,
    NSZonePtr,
};

#[derive(Debug)]
pub enum NSNumberHostObject {
    Bool(bool),
    UnsignedLongLong(u64),
    LongLong(i64),
    Double(f64),
    Int(i32),
    Float(f32),
}
impl HostObject for NSNumberHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// NSValue is an abstract class. None of the things it should provide are
// implemented here yet (TODO).
@implementation NSValue: NSObject

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

@end

// NSNumber is not an abstract class.
@implementation NSNumber: NSValue

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSNumberHostObject::Bool(false));
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)numberWithInteger:(NSInteger)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithInteger:value];
    autorelease(env, new)
}

+ (id)numberWithInt:(i32)value {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithInteger:value];
    autorelease(env, new)
}

+ (id)numberWithFloat:(f32)value {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithFloat:value];
    autorelease(env, new)
}

+ (id)numberWithBool:(bool)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithBool:value];
    autorelease(env, new)
}

+ (id)numberWithDouble:(f64)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithDouble:value];
    autorelease(env, new)
}

+ (id)numberWithLongLong:(i64)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithLongLong:value];
    autorelease(env, new)
}

+ (id)numberWithUnsignedLongLong:(u64)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithUnsignedLongLong:value];
    autorelease(env, new)
}

+ (id)numberWithFloat:(f32)value {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithFloat:value];
    autorelease(env, new)
}

// TODO: other types

- (id)initWithInteger:(NSInteger)value {
    *env.objc.borrow_mut::<NSNumberHostObject>(this) = NSNumberHostObject::Int(value);
    this
}

- (id)initWithFloat:(f32)value {
    *env.objc.borrow_mut::<NSNumberHostObject>(this) = NSNumberHostObject::Float(value);
    this
}

- (id)initWithBool:(bool)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::Bool(value);
    this
}

- (id)initWithDouble:(f64)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::Double(value);
    this
}

- (id)initWithLongLong:(i64)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::LongLong(value);
    this
}

- (id)initWithUnsignedLongLong:(u64)value {
    *env.objc.borrow_mut(this) = NSNumberHostObject::UnsignedLongLong(value);
    this
}

- (id)initWithFloat:(f32)value {
    *env.objc.borrow_mut::<NSNumberHostObject>(this) = NSNumberHostObject::Float(
        value,
    );
    this
}

- (NSUInteger)hash {
    match env.objc.borrow(this) {
        &NSNumberHostObject::Bool(value) => super::hash_helper(&value),
        &NSNumberHostObject::Int(value) => super::hash_helper(&value),
        &NSNumberHostObject::Float(value) => super::hash_helper(&value.to_bits()),
        _ => todo!()
    }
}
- (bool)isEqualTo:(id)other {
    if this == other {
        return true;
    }
    let class: Class = msg_class![env; NSNumber class];
    if !msg![env; other isKindOfClass:class] {
        return false;
    }
     match env.objc.borrow(this) {
         &NSNumberHostObject::Bool(a) => {
            let b = if let &NSNumberHostObject::Bool(b) = env.objc.borrow(other) { b } else { unreachable!() };
            a == b
         },
        &NSNumberHostObject::Int(a) => {
            let b = if let &NSNumberHostObject::Int(b) = env.objc.borrow(other) { b } else { unreachable!() };
            a == b
        },
        &NSNumberHostObject::Float(a) => {
            let b = if let &NSNumberHostObject::Float(b) = env.objc.borrow(other) { b } else { unreachable!() };
            a == b
        },
        _ => todo!()
    }
}

- (NSInteger)integerValue {
    let value = if let &NSNumberHostObject::Int(value) = env.objc.borrow(this) { value } else { todo!() };
    value
}

- (i32)intValue {
    match env.objc.borrow(this) {
        &NSNumberHostObject::Int(value) => value,
        &NSNumberHostObject::LongLong(value) => value as i32,
        x => todo!("{:?}", x)
    }
}

- (f32)floatValue {
    match env.objc.borrow(this) {
        &NSNumberHostObject::Float(value) => value,
        &NSNumberHostObject::Double(value) => value as f32,
        x => todo!("{:?}", x)
    }
}

- (f64)doubleValue {
    let value = if let &NSNumberHostObject::Float(value) = env.objc.borrow(this) { value } else { todo!() };
    value.try_into().unwrap()
}

// TODO: accessors etc

@end

};
