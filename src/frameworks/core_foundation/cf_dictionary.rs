/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFDictionary` and `CFMutableDictionary`.
//!
//! These are toll-free bridged to `NSDictionary` and `NSMutableDictionary` in
//! Apple's implementation. Here they are the same types.

use super::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use super::CFIndex;
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::foundation::{ns_string, NSUInteger};
use crate::mem::{ConstVoidPtr, MutPtr};
use crate::objc::{id, msg, msg_class, nil};
use crate::Environment;
use crate::frameworks::core_foundation::cf_data::CFDataRef;
use crate::frameworks::core_foundation::cf_string::CFStringRef;

pub type CFDictionaryRef = super::CFTypeRef;
pub type CFMutableDictionaryRef = super::CFTypeRef;

pub type CFPropertyListRef = super::CFTypeRef;

fn CFDictionaryCreateMutable(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    capacity: CFIndex,
    keyCallbacks: ConstVoidPtr, // TODO, should be `const CFDictionaryKeyCallBacks*`
    valueCallbacks: ConstVoidPtr, // TODO, should be `const CFDictionaryValueCallBacks*`
) -> CFMutableDictionaryRef {
    assert!(allocator == kCFAllocatorDefault); // unimplemented
    assert!(capacity == 0); // TODO: fixed capacity support
    assert!(keyCallbacks.is_null()); // TODO: support retaining etc
    assert!(valueCallbacks.is_null()); // TODO: support retaining etc

    msg_class![env; _touchHLE_NSMutableDictionary_non_retaining new]
}

fn CFDictionarySetValue(
    env: &mut Environment,
    dict: CFMutableDictionaryRef,
    key: ConstVoidPtr,
    value: ConstVoidPtr
) {
    let key: id = key.cast().cast_mut();
    let value: id = value.cast().cast_mut();
    msg![env; dict setValue:value forKey:key]
}

// CFDataRef CFPropertyListCreateXMLData(CFAllocatorRef allocator, CFPropertyListRef propertyList);
fn CFPropertyListCreateXMLData(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    property_list: CFPropertyListRef
) -> CFDataRef {
    nil
}

// CFPropertyListRef CFPropertyListCreateFromXMLData(CFAllocatorRef allocator, CFDataRef xmlData,
// CFOptionFlags mutabilityOption, CFStringRef *errorString);

fn CFPropertyListCreateFromXMLData(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    xml_data: CFDataRef,
    flags: u32,
    error_string: MutPtr<CFStringRef>
) -> CFPropertyListRef {
    let err = ns_string::get_static_str(env, "error");
    env.mem.write(error_string, err);
    nil
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFDictionaryCreateMutable(_, _, _, _)),
    export_c_func!(CFDictionarySetValue(_, _, _)),
    export_c_func!(CFPropertyListCreateXMLData(_, _)),
    export_c_func!(CFPropertyListCreateFromXMLData(_, _, _, _)),
];