/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSObject`, the root of most class hierarchies in Objective-C.
//!
//! Resources:
//! - Apple's [Advanced Memory Management Programming Guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/MemoryMgmt/Articles/MemoryMgmt.html)
//!   explains how reference counting works. Note that we are interested in what
//!   it calls "manual retain-release", not ARC.
//! - Apple's [Key-Value Coding Programming Guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/KeyValueCoding/SearchImplementation.html)
//!   explains the algorithm `setValue:forKey:` should follow.
//!
//! See also: [crate::objc], especially the `objects` module.

use super::ns_string::to_rust_string;
use super::ns_run_loop::NSDefaultRunLoopMode;
use super::{NSUInteger, ns_string};
use super::ns_dictionary::dict_from_keys_and_objects;
use crate::mem::MutVoidPtr;
use crate::objc::{
    id, nil, msg, msg_class, msg_send, objc_classes, Class, ClassExports, NSZonePtr, ObjC,
    TrivialHostObject, SEL,
};
use crate::frameworks::foundation::NSTimeInterval;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSObject

+ (id)alloc {
    msg![env; this allocWithZone:(MutVoidPtr::null())]
}
+ (id)allocWithZone:(NSZonePtr)_zone { // struct _NSZone*
    log_dbg!("[{:?} allocWithZone:]", this);
    env.objc.alloc_object(this, Box::new(TrivialHostObject), &mut env.mem)
}

+ (id)new {
    let new_object: id = msg![env; this alloc];
    msg![env; new_object init]
}

+ (Class)class {
    this
}

// See the instance method section for the normal versions of these.
+ (id)retain {
    this // classes are not refcounted
}
+ (())release {
    // classes are not refcounted
}
+ (())autorelease {
    // classes are not refcounted
}

+ (bool)instancesRespondToSelector:(SEL)selector {
    env.objc.class_has_method(this, selector)
}

+ (bool)conformsToProtocol:(MutVoidPtr)protocol {
    true
}

+ (())cancelPreviousPerformRequestsWithTarget:(id)target {
    
}

+ (bool)accessInstanceVariablesDirectly {
    true
}

- (id)init {
    this
}


- (id)retain {
    log_dbg!("[{:?} retain]", this);
    env.objc.increment_refcount(this);
    this
}
- (())release {
    log_dbg!("[{:?} release]", this);
    if env.objc.decrement_refcount(this) {
        () = msg![env; this dealloc];
    }
}
- (id)autorelease {
    () = msg_class![env; NSAutoreleasePool addObject:this];
    this
}

- (())dealloc {
    log_dbg!("[{:?} dealloc]", this);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (Class)class {
    ObjC::read_isa(this, &env.mem)
}
- (bool)isMemberOfClass:(Class)class {
    let this_class: Class = msg![env; this class];
    class == this_class
}
- (bool)isKindOfClass:(Class)class {
    let this_class: Class = msg![env; this class];
    env.objc.class_is_subclass_of(this_class, class)
}

- (bool)conformsToProtocol:(MutVoidPtr)protocol {
    true
}

- (NSUInteger)hash {
    this.to_bits()
}
- (bool)isEqual:(id)other {
    this == other
}

// TODO: description and debugDescription (both the instance and class method).
// This is not hard to add, but before adding a fallback implementation of it,
// we should make sure all the Foundation classes' overrides of it are there,
// to prevent weird behavior.
// TODO: localized description methods also? (not sure if NSObject has them)

// Helper for NSCopying
- (id)copy {
    msg![env; this copyWithZone:(MutVoidPtr::null())]
}

// Helper for NSMutableCopying
- (id)mutableCopy {
    msg![env; this mutableCopyWithZone:(MutVoidPtr::null())]
}

// NSKeyValueCoding
// https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/KeyValueCoding/SearchImplementation.html
- (())setValue:(id)value
       forKey:(id)key { // NSString*
    let key_string = to_rust_string(env, key); // TODO: avoid copy?
    assert!(key_string.is_ascii()); // TODO: do we have to handle non-ASCII keys?

    let class = msg![env; this class];

    // Look for the first accessor named set<Key>: or _set<Key>, in that order.
    // If found, invoke it with the input value (or unwrapped value, as needed)
    // and finish.
    if let Some(sel) = env.objc.lookup_selector(&format!(
        "set{}{}:",
        key_string.as_bytes()[0].to_ascii_uppercase() as char,
        &key_string[1..],
    )) {
        if env.objc.class_has_method(class, sel) {
            () = msg_send(env, (this, sel, value));
            return;
        }
    }
    if let Some(sel) = env.objc.lookup_selector(&format!(
        "_set{}{}:",
        key_string.as_bytes()[0].to_ascii_uppercase() as char,
        &key_string[1..],
    )) {
        if env.objc.class_has_method(class, sel) {
            () = msg_send(env, (this, sel, value));
            return;
        }
    }

    // If no simple accessor is found, and if the class method
    // accessInstanceVariablesDirectly returns YES, look for an instance
    // variable with a name like _<key>, _is<Key>, <key>, or is<Key>,
    // in that order.
    // If found, set the variable directly with the input value
    // (or unwrapped value) and finish.
    let sel = env.objc.lookup_selector("accessInstanceVariablesDirectly").unwrap();
    let accessInstanceVariablesDirectly = msg_send(env, (class, sel));
    // These ways of accessing are kinda hacky,
    // it'd be better if it was integrated with msg_send
    if accessInstanceVariablesDirectly {
        if let Some(sel) = env.objc.lookup_selector(&format!(
                "_{}",
                &key_string
        )) {
            if let Some(ivar_offset_ptr) = env.objc.class_has_ivar(class, sel) {
                let ivar_offset = env.mem.read(ivar_offset_ptr);
                // TODO: Use host_object's _instance_start property?
                let ivar_ptr = MutVoidPtr::from_bits(this.to_bits() + ivar_offset);
                env.mem.write(ivar_ptr.cast(), value);
                return;
            }
        }
        if let Some(sel) = env.objc.lookup_selector(&format!(
                "_is{}{}:",
                key_string.as_bytes()[0].to_ascii_uppercase() as char,
                &key_string[1..],
        )) {
            if let Some(ivar_offset_ptr) = env.objc.class_has_ivar(class, sel) {
                let ivar_offset = env.mem.read(ivar_offset_ptr);
                // TODO: Use host_object's _instance_start property?
                let ivar_ptr = MutVoidPtr::from_bits(this.to_bits() + ivar_offset);
                env.mem.write(ivar_ptr.cast(), value);
                return;
            }
        }
        if let Some(sel) = env.objc.lookup_selector(&format!(
                "{}",
                key_string,
        )) {
            if let Some(ivar_offset_ptr) = env.objc.class_has_ivar(class, sel) {
                let ivar_offset = env.mem.read(ivar_offset_ptr);
                // TODO: Use host_object's _instance_start property?
                let ivar_ptr = MutVoidPtr::from_bits(this.to_bits() + ivar_offset);
                env.mem.write(ivar_ptr.cast(), value);
                return;
            }
        }
        if let Some(sel) = env.objc.lookup_selector(&format!(
                "is{}{}:",
                key_string.as_bytes()[0].to_ascii_uppercase() as char,
                &key_string[1..],
        )) {
            if let Some(ivar_offset_ptr) = env.objc.class_has_ivar(class, sel) {
                let ivar_offset = env.mem.read(ivar_offset_ptr);
                // TODO: Use host_object's _instance_start property?
                let ivar_ptr = MutVoidPtr::from_bits(this.to_bits() + ivar_offset);
                env.mem.write(ivar_ptr.cast(), value);
                return;
            }
        }
    }

    // Upon finding no accessor or instance variable,
    // invoke setValue:forUndefinedKey:.
    // This raises an exception by default, but a subclass of NSObject
    // may provide key-specific behavior.
    let sel = env.objc.lookup_selector("setValue:forUndefinedKey:").unwrap();
    () = msg_send(env, (this, sel, value, key));
}

- (())setValue:(id)_value
forUndefinedKey:(id)key { // NSString*
    // TODO: Raise NSUnknownKeyException
    let class: Class = ObjC::read_isa(this, &env.mem);
    let class_name_string = env.objc.get_class_name(class).to_owned(); // TODO: Avoid copying
    let key_string = to_rust_string(env, key);
    panic!("Object {:?} of class {:?} ({:?}) does not have a setter for {} ({:?})\nAvailable selectors: {}", this, class_name_string, class, key_string, key, env.objc.all_class_selectors_as_strings(&env.mem, class).join(", "));
}

- (bool)respondsToSelector:(SEL)selector {
    let class = msg![env; this class];
    env.objc.class_has_method(class, selector)
}

- (id)performSelector:(SEL)sel {
    assert!(!sel.is_null());
    msg_send(env, (this, sel))
}

- (id)performSelector:(SEL)sel
           withObject:(id)o1 {
    assert!(!sel.is_null());
    msg_send(env, (this, sel, o1))
}

- (id)performSelector:(SEL)sel
           withObject:(id)o1
           withObject:(id)o2 {
    assert!(!sel.is_null());
    msg_send(env, (this, sel, o1, o2))
}

- (())performSelector:(SEL)aSelector 
           withObject:(id)anArgument 
           afterDelay:(NSTimeInterval)delay {
    
}

- (())performSelectorOnMainThread:(SEL)sel withObject:(id)arg waitUntilDone:(bool)wait {
    // // FIXME: main thread...
    // () = msg_send(env, (this, sel, arg));

    assert!(env.current_thread != 0);
    log!("performSelectorOnMainThread:{} withObject:{:?} waitUntilDone:{}", sel.as_str(&env.mem), arg, wait);
    assert!(!wait);

    let sel_key: id = ns_string::get_static_str(env, "SEL");
    let sel_str = ns_string::from_rust_string(env, sel.as_str(&env.mem).to_string());
    let arg_key: id = ns_string::get_static_str(env, "arg");
    let dict = dict_from_keys_and_objects(env, &[(sel_key, sel_str), (arg_key, arg)]);

    let selector = env.objc.lookup_selector("timerFireMethod:").unwrap();
    let timer:id = msg_class![env; NSTimer timerWithTimeInterval:0.0
                                              target:this
                                            selector:selector
                                            userInfo:dict
                                             repeats:false];

    let run_loop: id = msg_class![env; NSRunLoop mainRunLoop];
    let mode: id = ns_string::get_static_str(env, NSDefaultRunLoopMode);
    let _: () = msg![env; run_loop addTimer:timer forMode:mode];
}

- (())timerFireMethod:(id)which { // NSTimer *
    let dict: id = msg![env; which userInfo];

    let sel_key: id = ns_string::get_static_str(env, "SEL");
    let sel_str_id: id = msg![env; dict objectForKey:sel_key];
    let sel_str = ns_string::to_rust_string(env, sel_str_id);
    let sel = env.objc.lookup_selector(&sel_str).unwrap();

    let arg_key: id = ns_string::get_static_str(env, "arg");
    let arg: id = msg![env; dict objectForKey:arg_key];

    () = msg_send(env, (this, sel, arg));
}

@end

};
