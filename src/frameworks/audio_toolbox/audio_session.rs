/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `AudioSession.h` (Audio Session) // TODO: is this the real name?

use crate::abi::GuestFunction;
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::carbon_core::OSStatus;
use crate::frameworks::core_audio_types::{debug_fourcc, fourcc};
use crate::frameworks::core_foundation::cf_run_loop::{CFRunLoopMode, CFRunLoopRef};
use crate::mem::{guest_size_of, ConstVoidPtr, GuestUSize, MutPtr, MutVoidPtr};
use crate::Environment;

type AudioSessionInterruptionListener = GuestFunction;

const kAudioSessionBadPropertySizeError: OSStatus = fourcc(b"!siz") as _;

/// Usually a FourCC.
type AudioSessionPropertyID = u32;
const kAudioSessionProperty_OtherAudioIsPlaying: AudioSessionPropertyID = fourcc(b"othr");
const kAudioSessionProperty_AudioCategory: AudioSessionPropertyID = fourcc(b"acat");
const kAudioSessionProperty_CurrentHardwareSampleRate: AudioSessionPropertyID = fourcc(b"chsr");

const kAudioSessionCategory_SoloAmbientSound: u32 = fourcc(b"solo");

fn AudioSessionInitialize(
    _env: &mut Environment,
    _in_run_loop: CFRunLoopRef,
    _in_run_loop_mode: CFRunLoopMode,
    _in_interruption_listener: AudioSessionInterruptionListener,
    _in_client_data: MutVoidPtr,
) -> OSStatus {
    // TODO: actually implement this
    0 // success
}

fn AudioSessionGetProperty(
    env: &mut Environment,
    in_ID: AudioSessionPropertyID,
    io_data_size: MutPtr<u32>,
    out_data: MutVoidPtr,
) -> OSStatus {
    let required_size: GuestUSize = match in_ID {
        kAudioSessionProperty_OtherAudioIsPlaying => guest_size_of::<u32>(),
        kAudioSessionProperty_AudioCategory => guest_size_of::<u32>(),
        kAudioSessionProperty_CurrentHardwareSampleRate=> guest_size_of::<f64>(),
        _ => unimplemented!("Unimplemented property ID: {}", debug_fourcc(in_ID)),
    };
    if env.mem.read(io_data_size) != required_size {
        log!("Warning: AudioSessionGetProperty() failed");
        return kAudioSessionBadPropertySizeError;
    }

    match in_ID {
        kAudioSessionProperty_OtherAudioIsPlaying => {
            let value: u32 = 0;
            env.mem.write(out_data.cast(), value);
        }
        kAudioSessionProperty_AudioCategory => {
            // This is the default value. TODO: Actually support changing it?
            let value: u32 = kAudioSessionCategory_SoloAmbientSound;
            env.mem.write(out_data.cast(), value);
        }
        kAudioSessionProperty_CurrentHardwareSampleRate => {
            let value: f64 = 44100.0;
            env.mem.write(out_data.cast(), value);
        }
        _ => unreachable!(),
    }

    0 // success
}

fn AudioSessionSetProperty(
    _env: &mut Environment,
    in_ID: AudioSessionPropertyID,
    in_data_size: u32,
    _in_data: ConstVoidPtr,
) -> OSStatus {
    let required_size: GuestUSize = match in_ID {
        kAudioSessionProperty_AudioCategory => guest_size_of::<u32>(),
        _ => unimplemented!("Unimplemented property ID: {}", debug_fourcc(in_ID)),
    };
    if in_data_size != required_size {
        log!("Warning: AudioSessionGetProperty() failed");
        return kAudioSessionBadPropertySizeError;
    }

    // TODO: actually implement this

    0 // success
}

fn AudioSessionSetActive(_env: &mut Environment, _active: bool) -> OSStatus {
    0 // success
}

fn NewAUGraph(_env: &mut Environment, out_graph: MutVoidPtr) -> OSStatus {
    -1
}

fn DisposeAUGraph(_env: &mut Environment, in_graph: MutVoidPtr) -> OSStatus {
    -1
}

fn AUGraphAddNode(_env: &mut Environment, out_graph: MutVoidPtr, in_desc: MutVoidPtr, out_node: MutVoidPtr) -> OSStatus {
    -1
}

fn AUGraphConnectNodeInput(
    _env: &mut Environment, out_graph: MutVoidPtr, in_src_node: u32, in_src_out_num: u32,
    in_dest_node: u32, in_dest_in_num: u32
) -> OSStatus {
    -1
}

fn AUGraphOpen(_env: &mut Environment, in_graph: MutVoidPtr) -> OSStatus {
    -1
}

fn AUGraphStart(_env: &mut Environment, in_graph: MutVoidPtr) -> OSStatus {
    -1
}

fn AUGraphIsRunning(env: &mut Environment, in_graph: MutVoidPtr, out_is_running: MutVoidPtr) -> OSStatus {
    -1
}

fn AUGraphIsInitialized(env: &mut Environment, in_graph: MutVoidPtr, out_is_init: MutVoidPtr) -> OSStatus {
    -1
}

fn AUGraphNodeInfo(
    _env: &mut Environment, in_graph: MutVoidPtr, in_node: u32, out_desc: MutVoidPtr,
    out_audio_unit: MutVoidPtr
) -> OSStatus {
    -1
}

fn AUGraphSetNodeInputCallback(
    _env: &mut Environment, in_graph: MutVoidPtr, in_dest_node: u32, in_dest_in_num: u32,
    callback: MutVoidPtr
) -> OSStatus {
    -1
}

fn AUGraphInitialize(_env: &mut Environment, in_graph: MutVoidPtr) -> OSStatus {
    -1
}

fn AudioUnitSetProperty(
    _env: &mut Environment, in_unit: MutVoidPtr, in_id: u32, in_scope: u32,
    in_elem: u32, in_data: ConstVoidPtr, in_data_size: u32
) -> OSStatus {
    -1
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(AudioSessionInitialize(_, _, _, _)),
    export_c_func!(AudioSessionGetProperty(_, _, _)),
    export_c_func!(AudioSessionSetProperty(_, _, _)),
    export_c_func!(AudioSessionSetActive(_)),
    export_c_func!(NewAUGraph(_)),
    export_c_func!(DisposeAUGraph(_)),
    export_c_func!(AUGraphAddNode(_, _, _)),
    export_c_func!(AUGraphConnectNodeInput(_, _, _, _, _)),
    export_c_func!(AUGraphOpen(_)),
    export_c_func!(AUGraphStart(_)),
    export_c_func!(AUGraphIsRunning(_, _)),
    export_c_func!(AUGraphIsInitialized(_, _)),
    export_c_func!(AUGraphNodeInfo(_, _, _, _)),
    export_c_func!(AUGraphSetNodeInputCallback(_, _, _, _)),
    export_c_func!(AUGraphInitialize(_)),
    export_c_func!(AudioUnitSetProperty(_, _, _, _, _, _)),
];
