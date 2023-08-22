/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSDate`.

use crate::objc::{msg_class, objc_classes, ClassExports};
use super::NSTimeInterval;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSDate: NSObject

+ (NSTimeInterval)timeIntervalSinceReferenceDate {
    // This should be consistent with CFAbsoluteTimeGetCurrent()
    // TODO: This should use "Jan 1 2001 00:00:00 GMT" as an absolute reference instead
    let time: NSTimeInterval = msg_class![env; NSProcessInfo systemUptime];
    time
}

@end

};