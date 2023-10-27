/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFAllocator`. Currently there is no actual support for multiple allocators.

use super::CFTypeRef;
use crate::dyld::{ConstantExports, HostConstant};
use crate::mem::Ptr;

pub type CFAllocatorRef = CFTypeRef;

pub const kCFAllocatorDefault: CFAllocatorRef = Ptr::null();

pub const CONSTANTS: ConstantExports = &[
    ("_kCFAllocatorDefault", HostConstant::NullPtr),
    ("_mach_task_self_", HostConstant::NullPtr),
    ("_kCATransitionPush", HostConstant::NullPtr),
    ("_kCATransitionFromLeft", HostConstant::NullPtr),
    ("_kCATransitionFromTop", HostConstant::NullPtr),
    ("_kCATransitionFade", HostConstant::NullPtr),
    ("_kCATransitionFromRight", HostConstant::NullPtr),
    ("_kCATransitionFromBottom", HostConstant::NullPtr),
    ("_kCAMediaTimingFunctionEaseInEaseOut", HostConstant::NullPtr)
];
