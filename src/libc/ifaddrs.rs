/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `ifaddrs.h` (interface addresses)

use crate::dyld::FunctionExports;
use crate::export_c_func;
use crate::mem::{ConstPtr, MutPtr, MutVoidPtr};
use crate::Environment;

// TODO: struct definition
#[allow(non_camel_case_types)]
struct ifaddrs {}

fn getifaddrs(_env: &mut Environment, _ifap: MutPtr<MutPtr<ifaddrs>>) -> i32 {
    // TODO: implement
    -1
}

// int
//      inet_pton(int af, const char * restrict src, void * restrict dst);
fn inet_pton(_env: &mut Environment, af: i32, src: ConstPtr<u8>, dst: MutVoidPtr) -> i32 {
    -1
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(getifaddrs(_)),
    export_c_func!(inet_pton(_, _, _)),
];
