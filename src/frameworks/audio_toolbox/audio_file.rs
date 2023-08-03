/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `AudioFile.h` (Audio File Services)

use crate::audio; // Keep this module namespaced to avoid confusion
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::carbon_core::OSStatus;
use crate::frameworks::core_audio_types::{debug_fourcc, fourcc, kAudioFormatAppleIMA4, kAudioFormatFlagIsBigEndian, kAudioFormatFlagIsFloat, kAudioFormatFlagIsPacked, kAudioFormatFlagIsSignedInteger, kAudioFormatLinearPCM, AudioStreamBasicDescription, kAudioFormatMPEG4AAC};
use crate::frameworks::core_foundation::cf_url::CFURLRef;
use crate::frameworks::foundation::ns_url::to_rust_path;
use crate::mem::{guest_size_of, GuestUSize, MutPtr, MutVoidPtr, SafeRead};
use crate::Environment;
use std::collections::HashMap;
use crate::abi::{CallFromHost, GuestFunction};

#[derive(Default)]
pub struct State {
    audio_files: HashMap<AudioFileID, AudioFileHostObject>,
}
impl State {
    pub fn get(framework_state: &mut crate::frameworks::State) -> &mut Self {
        &mut framework_state.audio_toolbox.audio_file
    }
}

struct AudioFileHostObject {
    audio_file: audio::AudioFile,
}

#[repr(C, packed)]
struct OpaqueAudioFileID {
    _filler: u8,
}
unsafe impl SafeRead for OpaqueAudioFileID {}

type AudioFileID = MutPtr<OpaqueAudioFileID>;

const kAudioFileFileNotFoundError: OSStatus = -43;
const kAudioFileBadPropertySizeError: OSStatus = fourcc(b"!siz") as _;
const kAudioFileUnsupportedProperty: OSStatus = fourcc(b"pty?") as _;

type AudioFilePermissions = i8;
const kAudioFileReadPermission: AudioFilePermissions = 1;

/// Usually a FourCC.
type AudioFileTypeID = u32;

/// Usually a FourCC.
type AudioFilePropertyID = u32;
const kAudioFilePropertyDataFormat: AudioFilePropertyID = fourcc(b"dfmt");
const kAudioFilePropertyAudioDataByteCount: AudioFilePropertyID = fourcc(b"bcnt");
const kAudioFilePropertyAudioDataPacketCount: AudioFilePropertyID = fourcc(b"pcnt");
const kAudioFilePropertyPacketSizeUpperBound: AudioFilePropertyID = fourcc(b"pkub");
const kAudioFilePropertyMagicCookieData: AudioFilePropertyID = fourcc(b"mgic");
const kAudioFilePropertyChannelLayout: AudioFilePropertyID = fourcc(b"cmap");

fn AudioFileOpenURL(
    env: &mut Environment,
    in_file_ref: CFURLRef,
    in_permissions: AudioFilePermissions,
    in_file_type_hint: AudioFileTypeID,
    out_audio_file: MutPtr<AudioFileID>,
) -> OSStatus {
    return_if_null!(in_file_ref);

    assert!(in_permissions == kAudioFileReadPermission); // writing TODO
                                                         // The hint is optional and is supposed to only be used for certain file
                                                         // formats that can't be uniquely identified, which we don't support so far.
    assert!(in_file_type_hint == 0);

    let path = to_rust_path(env, in_file_ref);
    let bytes = env.fs.read(path.as_ref());
    if bytes.is_err() {
        log!("Warning: AudioFileOpenURL() for path {:?} failed", in_file_ref);
        return kAudioFileFileNotFoundError;
    };
    let audio_file = audio::AudioFile::open_for_reading(bytes.unwrap()).unwrap();

    let host_object = AudioFileHostObject { audio_file };

    let guest_audio_file = env.mem.alloc_and_write(OpaqueAudioFileID { _filler: 0 });
    State::get(&mut env.framework_state)
        .audio_files
        .insert(guest_audio_file, host_object);

    env.mem.write(out_audio_file, guest_audio_file);

    log_dbg!(
        "AudioFileOpenURL() opened path {:?}, new audio file handle: {:?}",
        in_file_ref,
        guest_audio_file
    );

    0 // success
}

/// typedef SInt64 (*AudioFile_GetSizeProc)(void *inClientData)
type AudioFile_GetSizeProc = GuestFunction;

/// typedef OSStatus (*AudioFile_ReadProc)(void *inClientData, SInt64 inPosition, UInt32 requestCount, void *buffer, UInt32 *actualCount);
type AudioFile_ReadProc = GuestFunction;

fn AudioFileOpenWithCallbacks(
    env: &mut Environment,
    in_client_data: MutVoidPtr,
    in_read_func: AudioFile_ReadProc,
    in_write_func: MutVoidPtr,
    in_get_size_func: AudioFile_GetSizeProc,
    in_set_size_func: MutVoidPtr,
    in_file_type_hint: AudioFileTypeID,
    out_audio_file: MutPtr<AudioFileID>,
) -> OSStatus {
    //assert!(in_client_data.is_null());
    //assert!(in_read_func.is_null());
    assert!(in_write_func.is_null());
    //assert!(in_get_size_func.is_null());
    assert!(in_set_size_func.is_null());

    assert_eq!(in_file_type_hint, 0);

    let size: i64 = in_get_size_func.call_from_host(env, (in_client_data,));
    log!("AudioFileOpenWithCallbacks callback get size {}", size);

    let guest_size: GuestUSize = size.try_into().unwrap();
    let guest_buffer = env.mem.alloc(guest_size);
    let actual_count: MutPtr<u32> = env.mem.alloc(guest_size_of::<u32>()).cast();
    let status: OSStatus = in_read_func.call_from_host(env, (in_client_data, 0i64, guest_size, guest_buffer, actual_count));
    log!("AudioFileOpenWithCallbacks callback read status {}", status);
    assert_eq!(status, 0);
    assert_eq!(guest_size, env.mem.read(actual_count));
    env.mem.free(actual_count.cast());

    let mut audio_data: Vec<u8> = Vec::new();
    audio_data.extend_from_slice(env.mem.bytes_at(guest_buffer.cast(), guest_size));
    env.mem.free(guest_buffer.cast());

    // let path  = "fn_track_0.bin";
    // std::fs::write(path, audio_data.clone()).unwrap();

    let audio_file = audio::AudioFile::open_for_reading(audio_data).unwrap();

    let host_object = AudioFileHostObject { audio_file };

    let guest_audio_file = env.mem.alloc_and_write(OpaqueAudioFileID { _filler: 0 });
    State::get(&mut env.framework_state)
        .audio_files
        .insert(guest_audio_file, host_object);

    env.mem.write(out_audio_file, guest_audio_file);

    log_dbg!(
        "AudioFileOpenWithCallbacks(), new audio file handle: {:?}",
        guest_audio_file
    );

    -1 // success
}

fn property_size(property_id: AudioFilePropertyID) -> GuestUSize {
    match property_id {
        kAudioFilePropertyDataFormat => guest_size_of::<AudioStreamBasicDescription>(),
        kAudioFilePropertyAudioDataByteCount => guest_size_of::<u64>(),
        kAudioFilePropertyAudioDataPacketCount => guest_size_of::<u64>(),
        kAudioFilePropertyPacketSizeUpperBound => guest_size_of::<u32>(),
        _ => 0 //unimplemented!("Unimplemented property ID: {}", debug_fourcc(property_id)),
    }
}

fn AudioFileGetPropertyInfo(
    env: &mut Environment,
    in_audio_file: AudioFileID,
    in_property_id: AudioFilePropertyID,
    out_data_size: MutPtr<u32>,
    is_writable: MutPtr<u32>,
) -> OSStatus {
    return_if_null!(in_audio_file);

    if in_property_id == kAudioFilePropertyMagicCookieData
        || in_property_id == kAudioFilePropertyChannelLayout
    {
        // Our currently supported formats probably don't use these properties.
        // Not sure if this is correct, but it skips some code we don't want to
        // run in Touch & Go.
        if !out_data_size.is_null() {
            env.mem.write(out_data_size, 0);
        }
        if !is_writable.is_null() {
            env.mem.write(is_writable, 0);
        }
        return kAudioFileUnsupportedProperty;
    }
    if !out_data_size.is_null() {
        env.mem.write(out_data_size, property_size(in_property_id));
    }
    if !is_writable.is_null() {
        env.mem.write(is_writable, 0); // TODO: probably not always correct
    }
    0 // success
}

fn AudioFileGetProperty(
    env: &mut Environment,
    in_audio_file: AudioFileID,
    in_property_id: AudioFilePropertyID,
    io_data_size: MutPtr<u32>,
    out_property_data: MutVoidPtr,
) -> OSStatus {
    return_if_null!(in_audio_file);

    log!("in_property_id {}", debug_fourcc(in_property_id));

    let required_size = property_size(in_property_id);
    if env.mem.read(io_data_size) != required_size {
        log!("Warning: AudioFileGetProperty() failed");
        return kAudioFileBadPropertySizeError;
    }

    let host_object = State::get(&mut env.framework_state)
        .audio_files
        .get_mut(&in_audio_file)
        .unwrap();

    match in_property_id {
        kAudioFilePropertyDataFormat => {
            let audio::AudioDescription {
                sample_rate,
                format,
                bytes_per_packet,
                frames_per_packet,
                channels_per_frame,
                bits_per_channel,
            } = host_object.audio_file.audio_description();

            let desc: AudioStreamBasicDescription = match format {
                audio::AudioFormat::LinearPcm {
                    is_float,
                    is_little_endian,
                } => {
                    let is_packed = (bits_per_channel * channels_per_frame * frames_per_packet)
                        == (bytes_per_packet * 8);
                    let format_flags = (u32::from(is_float) * kAudioFormatFlagIsFloat)
                        | (u32::from(!is_float) * kAudioFormatFlagIsSignedInteger)
                        | (u32::from(is_packed) * kAudioFormatFlagIsPacked)
                        | (u32::from(!is_little_endian) * kAudioFormatFlagIsBigEndian);
                    AudioStreamBasicDescription {
                        sample_rate,
                        format_id: kAudioFormatLinearPCM,
                        format_flags,
                        bytes_per_packet,
                        frames_per_packet,
                        bytes_per_frame: bytes_per_packet / frames_per_packet,
                        channels_per_frame,
                        bits_per_channel,
                        _reserved: 0,
                    }
                }
                audio::AudioFormat::AppleIma4 => {
                    AudioStreamBasicDescription {
                        sample_rate,
                        format_id: kAudioFormatAppleIMA4,
                        format_flags: 0,
                        bytes_per_packet,
                        frames_per_packet,
                        bytes_per_frame: 0, // compressed
                        channels_per_frame,
                        bits_per_channel,
                        _reserved: 0,
                    }
                }
                audio::AudioFormat::Mpeg4Aac => {
                    AudioStreamBasicDescription {
                        sample_rate,
                        format_id: kAudioFormatMPEG4AAC,
                        format_flags: 0,
                        bytes_per_packet,
                        frames_per_packet,
                        bytes_per_frame: 0, // compressed
                        channels_per_frame,
                        bits_per_channel,
                        _reserved: 0,
                    }
                }
            };
            env.mem.write(out_property_data.cast(), desc);
        }
        kAudioFilePropertyAudioDataByteCount => {
            let byte_count: u64 = host_object.audio_file.byte_count();
            env.mem.write(out_property_data.cast(), byte_count);
        }
        kAudioFilePropertyAudioDataPacketCount => {
            let packet_count: u64 = host_object.audio_file.packet_count();
            env.mem.write(out_property_data.cast(), packet_count);
        }
        kAudioFilePropertyPacketSizeUpperBound => {
            let packet_size_upper_bound: u32 = host_object.audio_file.packet_size_upper_bound();
            env.mem
                .write(out_property_data.cast(), packet_size_upper_bound);
        }
        _ => unreachable!(),
    }

    0 // success
}

fn AudioFileReadBytes(
    env: &mut Environment,
    in_audio_file: AudioFileID,
    _in_use_cache: bool,
    in_starting_byte: i64,
    io_num_bytes: MutPtr<u32>,
    out_buffer: MutVoidPtr,
) -> OSStatus {
    return_if_null!(in_audio_file);

    let host_object = State::get(&mut env.framework_state)
        .audio_files
        .get_mut(&in_audio_file)
        .unwrap();

    let bytes_to_read = env.mem.read(io_num_bytes);
    let buffer_slice = env.mem.bytes_at_mut(out_buffer.cast(), bytes_to_read);

    let bytes_read = host_object
        .audio_file
        .read_bytes(in_starting_byte.try_into().unwrap(), buffer_slice)
        .unwrap(); // TODO: handle seek error?
    //assert!((bytes_read as u64) == (bytes_to_read as u64)); // TODO: return eofErr
    env.mem.write(io_num_bytes, bytes_read.try_into().unwrap());

    0 // success
}

fn AudioFileReadPackets(
    env: &mut Environment,
    in_audio_file: AudioFileID,
    in_use_cache: bool,
    out_num_bytes: MutPtr<u32>,
    out_packet_descriptions: MutVoidPtr, // unimplemented
    in_starting_packet: i64,
    io_num_packets: MutPtr<u32>,
    out_buffer: MutVoidPtr,
) -> OSStatus {
    return_if_null!(in_audio_file);

    // Variable-size packets are not implemented currently. When they are,
    // this parameter should be a `MutPtr<AudioStreamPacketDescription>`.
    assert!(out_packet_descriptions.is_null());

    let host_object = State::get(&mut env.framework_state)
        .audio_files
        .get_mut(&in_audio_file)
        .unwrap();
    let packet_size = host_object.audio_file.packet_size_fixed();

    let packets_to_read = env.mem.read(io_num_packets);

    let starting_byte = i64::from(packet_size)
        .checked_mul(in_starting_packet)
        .unwrap();
    let bytes_to_read = packets_to_read.checked_mul(packet_size).unwrap();

    env.mem.write(out_num_bytes, bytes_to_read);
    let res = AudioFileReadBytes(
        env,
        in_audio_file,
        in_use_cache,
        starting_byte,
        out_num_bytes,
        out_buffer,
    );

    let bytes_read = env.mem.read(out_num_bytes);
    let packets_read = bytes_read / packet_size;
    env.mem.write(io_num_packets, packets_read);

    res
}

fn AudioFileReadPacketData(
    env: &mut Environment,
    in_audio_file: AudioFileID,
    in_use_cache: bool,
    out_num_bytes: MutPtr<u32>,
    out_packet_descriptions: MutVoidPtr, // unimplemented
    in_starting_packet: i64,
    io_num_packets: MutPtr<u32>,
    out_buffer: MutVoidPtr,
) -> OSStatus {
    AudioFileReadPackets(env, in_audio_file, in_use_cache, out_num_bytes, out_packet_descriptions, in_starting_packet, io_num_packets, out_buffer)
}

fn AudioFileClose(env: &mut Environment, in_audio_file: AudioFileID) -> OSStatus {
    return_if_null!(in_audio_file);

    let _host_object = State::get(&mut env.framework_state)
        .audio_files
        .remove(&in_audio_file)
        .unwrap();
    env.mem.free(in_audio_file.cast());
    log_dbg!(
        "AudioFileClose() destroyed audio file handle: {:?}",
        in_audio_file
    );
    0 // success
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(AudioFileOpenURL(_, _, _, _)),
    export_c_func!(AudioFileOpenWithCallbacks(_, _, _, _, _, _, _)),
    export_c_func!(AudioFileGetPropertyInfo(_, _, _, _)),
    export_c_func!(AudioFileGetProperty(_, _, _, _)),
    export_c_func!(AudioFileReadBytes(_, _, _, _, _)),
    export_c_func!(AudioFileReadPackets(_, _, _, _, _, _, _)),
    export_c_func!(AudioFileReadPacketData(_, _, _, _, _, _, _)),
    export_c_func!(AudioFileClose(_)),
];
