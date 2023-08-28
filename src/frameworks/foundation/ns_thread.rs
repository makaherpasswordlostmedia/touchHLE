/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSThread`.

use crate::msg;
use crate::objc::{id, objc_classes, ClassExports, SEL, nil, msg_send};
use crate::objc::classes::ClassHostObject;
use crate::frameworks::foundation::NSTimeInterval;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSThread: NSObject

+ (f64)threadPriority {
    log!("TODO: [NSThread threadPriority] (not implemented yet)");
    1.0
}

+ (bool)setThreadPriority:(f64)priority {
    log!("TODO: [NSThread setThreadPriority:{:?}] (ignored)", priority);
    true
}

+ (id)currentThread {
    // Simple hack to make the `setThreadPriority:` work as an instance method
    // (it's both a class and an instance method). Must be replaced if we ever
    // need to support other methods.
    this
}

+ (())sleepForTimeInterval:(NSTimeInterval)interval {

}

// TODO: construction etc

- (id)initWithTarget:(id)target selector:(SEL)selector object:(id)object {
    // let target_class = msg![env; target class];
    // let class_host_object = env.objc.get_host_object(target_class).unwrap();
    // let &ClassHostObject {
    //     ref name,
    //     ..
    // } = class_host_object.as_any().downcast_ref().unwrap();
    //
    // log!("NSThread initWithTarget {} sel {}", name, selector.as_str(&env.mem));

    // () = msg_send(env, (target, selector, object));

    nil
}

@end

};
