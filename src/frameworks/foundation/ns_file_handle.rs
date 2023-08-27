use super::ns_string;
use super::NSUInteger;
use crate::libc::posix_io;
use crate::mem::ConstPtr;
use crate::{msg, msg_class};
use crate::objc::{id, objc_classes, ClassExports, HostObject, nil, autorelease};

struct NSFileHandleHostObject {
    fd: posix_io::FileDescriptor,
}
impl HostObject for NSFileHandleHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSFileHandle: NSObject

+ (id)fileHandleForReadingAtPath:(id)path { // NSString*
    log!("fileHandleForReadingAtPath {}", ns_string::to_rust_string(env, path));
    let path_str: ConstPtr<u8> = msg![env; path UTF8String];
    match posix_io::open_direct(env, path_str, posix_io::O_RDONLY) {
        -1 => nil,
        fd => {
            let host_object = Box::new(NSFileHandleHostObject {
                fd
            });
            let new = env.objc.alloc_object(this, host_object, &mut env.mem);
            autorelease(env, new)
        },
    }
}

- (())seekToFileOffset:(i64)offset {
    let &NSFileHandleHostObject {
        fd
    } = env.objc.borrow(this);
    match posix_io::lseek(env, fd, offset, posix_io::SEEK_SET) {
        -1 => panic!("seekToFileOffset: failed"),
        _cur_pos => (),
    }
}

- (id)readDataOfLength:(NSUInteger)length { // NSData*
    let &NSFileHandleHostObject {
        fd
    } = env.objc.borrow(this);
    let buffer = env.mem.alloc(length);
    match posix_io::read(env, fd, buffer, length) {
        -1 => panic!("readDataOfLength: failed"),
        bytes_read => {
            assert_eq!(length, bytes_read.try_into().unwrap());
            msg_class![env; NSData dataWithBytesNoCopy:buffer length:length]
        }
    }
}

- (())closeFile {
    // file is closed on dealloc
    // TODO: keep closed state and raise an exception if handle is used after the closing
}

- (())dealloc {
    let &NSFileHandleHostObject {
        fd
    } = env.objc.borrow(this);
    posix_io::close(env, fd);
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};