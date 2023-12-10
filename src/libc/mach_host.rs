/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
#![allow(non_camel_case_types)]

use crate::dyld::{export_c_func, FunctionExports};
use crate::libc::time::{time_t, timespec};
use crate::mem::{guest_size_of, MutPtr, SafeRead};
use crate::Environment;
use std::time::Instant;

type mach_port_t = u32;
type host_t = mach_port_t;
type integer_t = i32;
type host_flavor_t = integer_t;
type host_info_t = MutPtr<integer_t>;
type natural_t = u32;
type mach_msg_type_number_t = natural_t;
type kern_return_t = i32;

type task_t = mach_port_t;
type task_name_t = mach_port_t;
type task_flavor_t = natural_t;

type task_info_t = MutPtr<integer_t>;

type vm_size_t = natural_t;

type clock_id_t = i32;
type clock_serv_t = mach_port_t;

type mach_timespec_t = timespec;

const HOST_SELF: host_t = 0x484F5354; // HOST
const HOST_VM_INFO: host_flavor_t = 2;

#[repr(C, packed)]
struct vm_statistics {
    free_count: integer_t,
    active_count: integer_t,
    inactive_count: integer_t,
    wire_count: integer_t,
    zero_fill_count: integer_t,
    reactivations: integer_t,
    pageins: integer_t,
    pageouts: integer_t,
    faults: integer_t,
    cow_faults: integer_t,
    lookups: integer_t,
    hits: integer_t,
}
unsafe impl SafeRead for vm_statistics {}

/// Returns the host port of the current task
/// Since we do not have real mach port management
/// let's just hope that the fake value is good enough
fn mach_host_self(_: &mut Environment) -> host_t {
    return HOST_SELF;
}

/// Returns various kinds of host statistics
fn host_statistics(
    env: &mut Environment,
    host_priv: host_t,
    flavor: host_flavor_t,
    host_info_out: host_info_t,
    host_info_out_count: MutPtr<mach_msg_type_number_t>,
) -> kern_return_t {
    if host_priv != HOST_SELF {
        unimplemented!("Attempt to get statistics for non-self host")
    }

    let out_size_available = env.mem.read(host_info_out_count);
    match flavor {
        HOST_VM_INFO => {
            let out_size_expected = guest_size_of::<vm_statistics>() / guest_size_of::<integer_t>();
            assert!(out_size_expected <= out_size_available);
            env.mem.memset(host_info_out.cast(), 0, out_size_available);
            env.mem.write(
                host_info_out.cast(),
                vm_statistics {
                    free_count: 65536,  // 256MB
                    active_count: 8192, // 32MB
                    inactive_count: 57344,
                    wire_count: 0,
                    zero_fill_count: 57344,
                    reactivations: 0,
                    pageins: 0,
                    pageouts: 0,
                    faults: 0,
                    cow_faults: 0,
                    lookups: 0,
                    hits: 0,
                },
            );
        }
        _ => unimplemented!("TODO: flavor {:?}", flavor),
    }
    0 // Success
}

fn host_page_size(
    env: &mut Environment,
    _host_priv: host_t,
    page_size: MutPtr<vm_size_t>
) -> kern_return_t {
    env.mem.write(page_size, 4096);
    0 // Success
}

fn host_get_clock_service(
    env: &mut Environment,
    host_priv: host_t,
    clock_id: clock_id_t,
    clock_name: MutPtr<clock_serv_t>,
) -> kern_return_t {
    assert_eq!(clock_id, 0);
    0 // Success
}

fn clock_get_time(
    env: &mut Environment,
    clock_name: clock_serv_t,
    cur_time: MutPtr<mach_timespec_t>,
) -> kern_return_t {
    let dur = Instant::now().duration_since(env.startup_time);
    let tv_sec: time_t = dur.as_secs().try_into().unwrap();
    let tv_nsec: i32 = dur.subsec_nanos().try_into().unwrap();
    env.mem.write(cur_time, mach_timespec_t { tv_sec, tv_nsec });
    0 // Success
}

/*
kern_return_t task_info
(
	task_name_t target_task,
	task_flavor_t flavor,
	task_info_t task_info_out,
	mach_msg_type_number_t *task_info_outCnt
);
 */

fn task_info(
    env: &mut Environment,
    target_task: task_name_t,
    flavor: task_flavor_t,
    task_info_out: task_info_t,
    task_info_outCnt: MutPtr<mach_msg_type_number_t>
) -> kern_return_t {
    0 // Success
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(mach_host_self()),
    export_c_func!(host_statistics(_, _, _, _)),
    export_c_func!(host_page_size(_, _)),
    export_c_func!(host_get_clock_service(_, _, _)),
    export_c_func!(clock_get_time(_, _)),
    export_c_func!(task_info(_, _, _, _)),
];