/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSBundle`.

use std::borrow::Cow;

use super::ns_array;
use super::ns_string;
use crate::bundle::Bundle;
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, ClassExports, HostObject,
};
use crate::frameworks::foundation::ns_dictionary::dict_from_keys_and_objects;
use crate::frameworks::core_foundation::cf_run_loop::kCFBundleExecutableKey;

#[derive(Default)]
pub struct State {
    main_bundle: Option<id>,
}

struct NSBundleHostObject {
    /// If this is [None], this is the main bundle's NSBundle instance and the
    /// [Bundle] is stored in [crate::Environment], not here.
    _bundle: Option<Bundle>,
    /// NSString with bundle path.
    bundle_path: id,
    /// NSURL with bundle path. [None] if not created yet.
    bundle_url: Option<id>,
}
impl HostObject for NSBundleHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSBundle: NSObject

+ (id)mainBundle {
    if let Some(bundle) = env.framework_state.foundation.ns_bundle.main_bundle {
        bundle
    } else {
        let bundle_path = env.bundle.bundle_path().as_str().to_string();
        let bundle_path = ns_string::from_rust_string(env, bundle_path);
        let host_object = NSBundleHostObject {
            _bundle: None,
            bundle_path,
            bundle_url: None,
        };
        let new = env.objc.alloc_object(
            this,
            Box::new(host_object),
            &mut env.mem
        );
        env.framework_state.foundation.ns_bundle.main_bundle = Some(new);
        new
   }
}

- (())dealloc {
    let &NSBundleHostObject { bundle_url, .. } = env.objc.borrow(this);
    if let Some(bundle_url) = bundle_url {
        release(env, bundle_url);
    }
    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)bundlePath {
    env.objc.borrow::<NSBundleHostObject>(this).bundle_path
}
- (id)bundleURL {
    if let Some(url) = env.objc.borrow::<NSBundleHostObject>(this).bundle_url {
        url
    } else {
        let bundle_path: id = msg![env; this bundlePath];
        let new: id = msg_class![env; NSURL alloc];
        let new: id = msg![env; new initFileURLWithPath:bundle_path];
        env.objc.borrow_mut::<NSBundleHostObject>(this).bundle_url = Some(new);
        new
    }
}

- (id)resourcePath {
    // This seems to be the same as the bundle path. The iPhone OS bundle
    // structure is a lot flatter than the macOS one.
    msg![env; this bundlePath]
}
- (id)resourceURL {
    // This seems to be the same as the bundle path. The iPhone OS bundle
    // structure is a lot flatter than the macOS one.
    msg![env; this bundleURL]
}

- (id)pathForResource:(id)name // NSString*
               ofType:(id)extension // NSString*
          inDirectory:(id)directory { // NSString*
    assert!(name != nil); // TODO

    // FIXME: localized resource handling?
    // FIXME: return nil if path does not exist

    let mut path: id = msg![env; this resourcePath];
    if directory != nil {
        path = msg![env; path stringByAppendingPathComponent:directory];
    }
    path = msg![env; path stringByAppendingPathComponent:name];
    if extension != nil {
        path = msg![env; path stringByAppendingPathExtension:extension];
    }
    path
}

- (id)pathsForResourcesOfType:(id)extension // NSString*
    inDirectory:(id)directory { // NSString*
    assert!(directory.is_null());
    let ext = ns_string::to_rust_string(env, extension);
    // let dir = ns_string::to_rust_string(env, directory);
    //log!("ext {}", ext);
    assert_eq!("xml", ext);
    let name = ns_string::from_rust_string(env, "worlds_list.xml".to_owned());
    let path = msg![env; this pathForResource:name ofType:extension];
    ns_array::from_vec(env, vec![path])
}

- (id)pathForResource:(id)name // NSString*
               ofType:(id)extension { // NSString*
    msg![env; this pathForResource:name ofType:extension inDirectory:nil]
}
- (id)URLForResource:(id)name // NSString*
       withExtension:(id)extension // NSString *
        subdirectory:(id)subpath { // NSString *
   let path_string: id = msg![env; this pathForResource:name
                                                 ofType:extension
                                            inDirectory:subpath];
   let path_url: id = msg_class![env; NSURL alloc];
   let path_url: id = msg![env; path_url initFileURLWithPath:path_string];
   autorelease(env, path_url)
}
- (id)URLForResource:(id)name // NSString*
       withExtension:(id)extension { // NSString *
   msg![env; this URLForResource:name withExtension:extension subdirectory:nil]
}

-(id)localizedStringForKey:(id)key
                          value:(id)value
                          table:(id)tableName {
    log!("localizedStringForKey {} {} {}",
            if key == nil { std::borrow::Cow::from("(null)") } else { ns_string::to_rust_string(env, key) },
            if value == nil { std::borrow::Cow::from("(null)") } else { ns_string::to_rust_string(env, value) },
            if tableName == nil { std::borrow::Cow::from("(null)") } else { ns_string::to_rust_string(env, tableName) }
    );
    value
}

- (id)infoDictionary {
    // TODO: convert info.plist to a dict
    dict_from_keys_and_objects(env, &[])
}

- (id)objectForInfoDictionaryKey:(id)key { // NSString*
    let key_str = ns_string::to_rust_string(env, key); // TODO: avoid copy
    match key_str {
       Cow::Borrowed(kCFBundleExecutableKey) =>
            ns_string::from_rust_string(env, env.bundle.executable().to_string()),
        _ => unimplemented!()
    }
}

// TODO: constructors, more accessors

@end

};
