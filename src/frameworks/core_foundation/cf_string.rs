/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFString`.
//!
//! This is toll-free bridged to `NSString` in Apple's implementation. Here it
//! is the same type.

use super::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use super::cf_dictionary::CFDictionaryRef;
use crate::abi::{DotDotDot, VaList};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::CFIndex;
use crate::frameworks::foundation::ns_string;
use crate::frameworks::foundation::ns_string::NSCaseInsensitiveSearch;
use crate::mem::{ConstPtr, MutPtr};
use crate::objc::{id, msg, msg_class};
use crate::Environment;
use crate::frameworks::core_foundation::cf_array::CFArrayRef;

pub type CFStringRef = super::CFTypeRef;

pub type CFStringEncoding = u32;
pub const kCFStringEncodingASCII: CFStringEncoding = 0x600;
pub const kCFStringEncodingUTF8: CFStringEncoding = 0x8000100;
pub const kCFStringEncodingUnicode: CFStringEncoding = 0x100;
pub const kCFStringEncodingUTF16: CFStringEncoding = kCFStringEncodingUnicode;
pub const kCFStringEncodingUTF16BE: CFStringEncoding = 0x10000100;
pub const kCFStringEncodingUTF16LE: CFStringEncoding = 0x14000100;
fn CFStringConvertEncodingToNSStringEncoding(
    _env: &mut Environment,
    encoding: CFStringEncoding,
) -> ns_string::NSStringEncoding {
    match encoding {
        0 => ns_string::NSASCIIStringEncoding, // TODO: kCFStringEncodingMacRoman
        kCFStringEncodingASCII => ns_string::NSASCIIStringEncoding,
        kCFStringEncodingUTF8 => ns_string::NSUTF8StringEncoding,
        kCFStringEncodingUTF16 => ns_string::NSUTF16StringEncoding,
        kCFStringEncodingUTF16BE => ns_string::NSUTF16BigEndianStringEncoding,
        kCFStringEncodingUTF16LE => ns_string::NSUTF16LittleEndianStringEncoding,
        _ => unimplemented!("Unhandled: CFStringEncoding {:#x}", encoding),
    }
}
fn CFStringConvertNSStringEncodingToEncoding(
    _env: &mut Environment,
    encoding: ns_string::NSStringEncoding,
) -> CFStringEncoding {
    match encoding {
        ns_string::NSASCIIStringEncoding => kCFStringEncodingASCII,
        ns_string::NSUTF8StringEncoding => kCFStringEncodingUTF8,
        ns_string::NSUTF16StringEncoding => kCFStringEncodingUTF16,
        ns_string::NSUTF16BigEndianStringEncoding => kCFStringEncodingUTF16BE,
        ns_string::NSUTF16LittleEndianStringEncoding => kCFStringEncodingUTF16LE,
        _ => unimplemented!("Unhandled: NSStringEncoding {:#x}", encoding),
    }
}

fn CFStringCompare(
    env: &mut Environment,
    string1: CFStringRef,
    string2: CFStringRef,
    flags: i32,
) -> i32 {
    assert_eq!(flags, 1);
    msg![env; string1 compare:string2 options:NSCaseInsensitiveSearch]
}

fn CFStringCreateWithBytes(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    bytes: ConstPtr<u8>,
    num_bytes: CFIndex,
    encoding: CFStringEncoding,
    is_external_repr: bool,
) -> CFStringRef {
    assert!(allocator == kCFAllocatorDefault); // unimplemented
    assert!(!is_external_repr);
    let len: u32 = num_bytes.try_into().unwrap();
    let encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    let ns_string: id = msg_class![env; NSString alloc];
    msg![env; ns_string initWithBytes:bytes length:len encoding:encoding]
}

fn CFStringCreateWithCString(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    c_string: ConstPtr<u8>,
    encoding: CFStringEncoding,
) -> CFStringRef {
    assert!(allocator == kCFAllocatorDefault); // unimplemented
    let encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    let ns_string: id = msg_class![env; NSString alloc];
    msg![env; ns_string initWithCString:c_string encoding:encoding]
}

fn CFStringCreateWithFormat(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    format_options: CFDictionaryRef,
    format: CFStringRef,
    args: DotDotDot,
) -> CFStringRef {
    CFStringCreateWithFormatAndArguments(env, allocator, format_options, format, args.start())
}

fn CFStringCreateWithFormatAndArguments(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    // Apple's own docs say these are unimplemented!
    _format_options: CFDictionaryRef,
    format: CFStringRef,
    args: VaList,
) -> CFStringRef {
    assert!(allocator == kCFAllocatorDefault); // unimplemented
    let res = ns_string::with_format(env, format, args);
    ns_string::from_rust_string(env, res)
}

fn CFStringGetCString(
    env: &mut Environment,
    string: CFStringRef,
    buffer: MutPtr<u8>,
    buffer_size: CFIndex,
    encoding: CFStringEncoding,
) -> bool {
    let encoding = CFStringConvertEncodingToNSStringEncoding(env, encoding);
    let buffer_size: u32 = buffer_size.try_into().unwrap();
    msg![env; string getCString:buffer
                      maxLength:buffer_size
                       encoding:encoding]
}

fn CFStringGetSystemEncoding(_env: &mut Environment) -> CFStringEncoding {
    kCFStringEncodingASCII
}

fn CFStringCreateArrayBySeparatingStrings(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    string: CFStringRef,
    separator: CFStringRef
) -> CFArrayRef {
    assert!(allocator == kCFAllocatorDefault); // unimplemented
    msg![env; string componentsSeparatedByString:separator]
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFStringConvertEncodingToNSStringEncoding(_)),
    export_c_func!(CFStringConvertNSStringEncodingToEncoding(_)),
    export_c_func!(CFStringCompare(_, _, _)),
    export_c_func!(CFStringCreateWithBytes(_, _, _, _, _)),
    export_c_func!(CFStringCreateWithCString(_, _, _)),
    export_c_func!(CFStringCreateWithFormat(_, _, _, _)),
    export_c_func!(CFStringCreateWithFormatAndArguments(_, _, _, _)),
    export_c_func!(CFStringGetCString(_, _, _, _)),
    export_c_func!(CFStringGetSystemEncoding()),
    export_c_func!(CFStringCreateArrayBySeparatingStrings(_, _, _))
];
