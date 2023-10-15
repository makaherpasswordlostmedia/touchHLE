/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSProcessInfo`.

use super::NSTimeInterval;
use crate::objc::{id, objc_classes, ClassExports};
use std::time::Instant;
use crate::frameworks::foundation::ns_string;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSProcessInfo: NSObject

+ (id)processInfo {
    this
}

// FIXME: it should be an instance method
+ (id)processName {
    ns_string::get_static_str(env, "MY PROCESS NAME")
}

+ (NSTimeInterval)systemUptime {
    Instant::now().duration_since(env.startup_time).as_secs_f64()
}

@end

};
