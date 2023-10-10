/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSLocale`.

use super::{ns_array, ns_string};
use crate::dyld::{ConstantExports, HostConstant};
use crate::objc::{id, objc_classes, ClassExports, HostObject};
use crate::Environment;

const NSLocaleCountryCode: &str = "NSLocaleCountryCode";

pub const CONSTANTS: ConstantExports = &[(
    "_NSLocaleCountryCode",
    HostConstant::NSString(NSLocaleCountryCode),
)];

#[derive(Default)]
pub struct State {
    current_locale: Option<id>,
    preferred_languages: Option<id>,
}
impl State {
    fn get(env: &mut Environment) -> &mut State {
        &mut env.framework_state.foundation.ns_locale
    }
}

struct NSLocalHostObject {
    country_code: id,
}
impl HostObject for NSLocalHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSLocale: NSObject

// The documentation isn't clear about what the format of the strings should be,
// but Super Monkey Ball does `isEqualToString:` against "fr", "es", "de", "it"
// and "ja", and its locale detection works properly, so presumably they do not
// usually have region suffixes.
+ (id)preferredLanguages {
    if let Some(existing) = State::get(env).preferred_languages {
        existing
    } else {
        let lang = if let Ok(lang) = std::env::var("LANG") {
            // turn e.g. "sv_SE.UTF-8" into just "sv"
            let lang = lang.split_once(['_', '.'])
                           .map(|(a, _b)| a)
                           .unwrap_or(&lang)
                           .to_string();
            log!("The app requested your preferred languages. {:?} will reported based on your LANG environment variable.", lang);
            lang
        } else {
            let lang = "en".to_string();
            log!("The app requested your preferred language. No LANG environment variable was found, so {:?} (English) will be reported.", lang);
            lang
        };
        let lang_ns_string = ns_string::from_rust_string(env, lang);
        let new = ns_array::from_vec(env, vec![lang_ns_string]);
        State::get(env).preferred_languages = Some(new);
        new
    }
}

+ (id)currentLocale {
    if let Some(locale) = State::get(env).current_locale {
        locale
    } else {
        // TODO: guess country code from LANG ?
        let country_code = ns_string::get_static_str(env, "US");
        let host_object = NSLocalHostObject {
            country_code
        };
        let new_locale = env.objc.alloc_object(
            this,
            Box::new(host_object),
            &mut env.mem
        );
        State::get(env).current_locale = Some(new_locale);
        new_locale
    }
}

// TODO: constructors, more accessors

- (id)objectForKey:(id)key {
    let key_str: &str = &ns_string::to_rust_string(env, key);
    match key_str {
        NSLocaleCountryCode => {
            let &NSLocalHostObject { country_code } = env.objc.borrow(this);
            country_code
        },
        _ => unimplemented!()
    }
}

@end

};
