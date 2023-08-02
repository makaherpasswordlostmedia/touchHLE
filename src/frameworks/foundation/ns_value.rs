/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The `NSValue` class cluster, including `NSNumber`.

use super::{NSUInteger, NSInteger};
use crate::objc::{
    autorelease, id, msg, msg_class, objc_classes, retain, Class, ClassExports, HostObject,
    NSZonePtr,
};

enum NSNumberHostObject {
    Bool(bool),
    Int(i32),
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

+ (id)numberWithBool:(bool)value {
    // TODO: for greater efficiency we could return a static-lifetime value

    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithBool:value];
    autorelease(env, new)
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

// TODO: types other than booleans

- (id)initWithBool:(bool)value {
    *env.objc.borrow_mut::<NSNumberHostObject>(this) = NSNumberHostObject::Bool(
        value,
    );
    this
}

- (id)initWithInteger:(NSInteger)value {
    *env.objc.borrow_mut::<NSNumberHostObject>(this) = NSNumberHostObject::Int(
        value,
    );
    this
}

- (NSUInteger)hash {
    match env.objc.borrow(this) {
         &NSNumberHostObject::Bool(value) => super::hash_helper(&value),
         &NSNumberHostObject::Int(value) => super::hash_helper(&value),
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
    }
}

- (NSInteger)integerValue {
    let value = if let &NSNumberHostObject::Int(value) = env.objc.borrow(this) { value } else { todo!() };
    value
}

// TODO: accessors etc

@end

};
