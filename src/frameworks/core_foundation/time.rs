/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Time things including `CFAbsoluteTime`.

use crate::abi::GuestArg;
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::CFTypeRef;
use crate::frameworks::foundation::NSTimeInterval;
use crate::libc::time::{time_t, timestamp_to_calendar_date};
use crate::mem::SafeRead;
use crate::objc::{msg_class, nil};
use crate::{impl_GuestRet_for_large_struct, Environment};
use std::time::SystemTime;

pub type CFTimeInterval = NSTimeInterval;
pub type CFAbsoluteTime = CFTimeInterval;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C, packed)]
pub struct CFGregorianDate {
    pub year: i32,    // SInt32
    pub month: i8,    // SInt8
    pub day: i8,      // SInt8
    pub hours: i8,    // SInt8
    pub minutes: i8,  // SInt8
    pub seconds: f64, // double
}
unsafe impl SafeRead for CFGregorianDate {}
impl_GuestRet_for_large_struct!(CFGregorianDate);
impl GuestArg for CFGregorianDate {
    const REG_COUNT: usize = 7;

    fn from_regs(regs: &[u32]) -> Self {
        CFGregorianDate {
            year: GuestArg::from_regs(&regs[0..1]),
            month: GuestArg::from_regs(&regs[1..2]),
            day: GuestArg::from_regs(&regs[2..3]),
            hours: GuestArg::from_regs(&regs[3..4]),
            minutes: GuestArg::from_regs(&regs[4..5]),
            seconds: GuestArg::from_regs(&regs[5..7]),
        }
    }
    fn to_regs(self, regs: &mut [u32]) {
        self.year.to_regs(&mut regs[0..1]);
        self.month.to_regs(&mut regs[1..2]);
        self.day.to_regs(&mut regs[2..3]);
        self.hours.to_regs(&mut regs[3..4]);
        self.minutes.to_regs(&mut regs[4..5]);
        self.seconds.to_regs(&mut regs[5..7]);
    }
}

fn CFAbsoluteTimeGetCurrent(env: &mut Environment) -> CFAbsoluteTime {
    // TODO: This should use "Jan 1 2001 00:00:00 GMT" as an absolute reference instead
    let time: NSTimeInterval = msg_class![env; NSProcessInfo systemUptime];
    time
}

type CFTimeZoneRef = CFTypeRef;

fn CFTimeZoneCopySystem(_env: &mut Environment) -> CFTimeZoneRef {
    // TODO: implement (nil seems to correspond to GMT)
    nil
}

fn CFAbsoluteTimeGetGregorianDate(
    _env: &mut Environment,
    _at: CFAbsoluteTime,
    tz: CFTimeZoneRef,
) -> CFGregorianDate {
    assert!(tz.is_null());
    log!(
        "TODO: CFAbsoluteTimeGetGregorianDate ignoring passed absolute time, using SystemTime::now"
    );
    let time64 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let time = time64 as time_t;
    let tm = timestamp_to_calendar_date(time);
    CFGregorianDate {
        year: 1900 + tm.tm_year,
        month: tm.tm_mon as i8,
        day: tm.tm_mday as i8,
        hours: tm.tm_hour as i8,
        minutes: tm.tm_min as i8,
        seconds: tm.tm_sec.into(),
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFAbsoluteTimeGetCurrent()),
    export_c_func!(CFTimeZoneCopySystem()),
    export_c_func!(CFAbsoluteTimeGetGregorianDate(_, _)),
];
