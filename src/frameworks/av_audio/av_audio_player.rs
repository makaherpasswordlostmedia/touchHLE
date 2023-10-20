use crate::objc::{id, ClassExports};
use crate::objc_classes;
use crate::frameworks::foundation::NSInteger;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation AVAudioPlayer: NSObject

- (id)initWithContentsOfURL:(id)url error:(id)error {
    this
}

- (())setDelegate:(id)delegate {
}

- (())setNumberOfLoops:(NSInteger)loops {
}

- (())setVolume:(f32)volume {
}

- (())prepareToPlay {
}

- (bool)isPlaying {
    true
}

- (bool)play {
    true
}

- (())pause {
}

- (())stop {
}

@end

};