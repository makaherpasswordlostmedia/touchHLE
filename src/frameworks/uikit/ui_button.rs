/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIButton`.

use crate::objc::{id, objc_classes, ClassExports};

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIButton: UIControl
// TODO
@end

@implementation UISlider: UIControl
// TODO
@end

@implementation UIRoundedRectButton: UIButton
// TODO
@end

@implementation UIRuntimeEventConnection: NSObject

- (id)initWithCoder:(id)coder {
    this
}

- (())connect {

}

@end

};
