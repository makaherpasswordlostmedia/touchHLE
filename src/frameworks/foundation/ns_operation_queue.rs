use crate::objc::{
    id, objc_classes, ClassExports,
};

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSOperationQueue: NSObject

- (())addOperation:(id)op { // NSOperation*
    log!("WARNING NSOperationQueue ignoring addOperation: {:?}", op);
}

@end

};