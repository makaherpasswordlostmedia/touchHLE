use crate::objc::{
    autorelease, id, msg, msg_class, msg_send, nil, objc_classes, ClassExports,
    HostObject, NSZonePtr,
};
use crate::abi::GuestFunction;

struct FakeCFTimerTargetHostObject {
    callout: GuestFunction,
}
impl HostObject for FakeCFTimerTargetHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation FakeCFTimerTarget: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(FakeCFTimerTargetHostObject {
        callout: GuestFunction::from_addr_with_thumb_bit(0),
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithCallout:(GuestFunction)callout {
    let host_object: &mut FakeCFTimerTargetHostObject = env.objc.borrow_mut(this);
    host_object.callout = callout;
    this
}

- (())timerFireMethod:(id)timer { // NSTimer *
    todo!();
}

@end

};