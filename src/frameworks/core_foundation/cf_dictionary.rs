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
use crate::frameworks::foundation::NSUInteger;
use crate::mem::ConstVoidPtr;
use crate::objc::{id, msg, msg_class};
use crate::Environment;

pub type CFDictionaryRef = super::CFTypeRef;
pub type CFMutableDictionaryRef = super::CFTypeRef;

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

    crate::objc::nil
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFDictionaryCreateMutable(_, _, _, _)),
];