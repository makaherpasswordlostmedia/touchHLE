/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Time things including `CFAbsoluteTime`.

use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::foundation::NSTimeInterval;
use crate::objc::{msg_class, nil};
use crate::Environment;
use crate::frameworks::core_foundation::CFTypeRef;

pub type CFTimeInterval = NSTimeInterval;
type CFAbsoluteTime = CFTimeInterval;

fn CFAbsoluteTimeGetCurrent(env: &mut Environment) -> CFAbsoluteTime {
    let time: NSTimeInterval = msg_class![env; NSProcessInfo systemUptime];
    time
}

type CFTimeZoneRef = CFTypeRef;

fn CFTimeZoneCopySystem(env: &mut Environment) -> CFTimeZoneRef {
    nil
}

type CFGregorianDate = CFTypeRef;

fn CFAbsoluteTimeGetGregorianDate(env: &mut Environment, _at: CFAbsoluteTime, _tz: CFTimeZoneRef) -> CFGregorianDate {
    // day, hours, minutes, months, seconds, years = 16 bytes
    let tmp = env.mem.alloc(16);
    msg_class![env; NSData dataWithBytes:tmp length:16]
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFAbsoluteTimeGetCurrent()),
    export_c_func!(CFTimeZoneCopySystem()),
    export_c_func!(CFAbsoluteTimeGetGregorianDate(_, _)),
];
