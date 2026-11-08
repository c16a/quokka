//! Media Foundation, Windows' operating-system decoder. It handles H.264 with the decoder that ships
//! with Windows, and HEVC with the HEVC Video Extensions, which the user installs from the Microsoft
//! Store. It is the Windows counterpart of VideoToolbox: the decoder is the system's own, and this
//! file calls it through the `windows` crate. Windows-specific types stay in this file.
//!
//! Packets go in as Annex B byte streams (see `annexb`), and pictures come out as NV12 or P010, which
//! are repacked into I420 or I010 (see `semiplanar`). Media Foundation decoders come in two kinds: a
//! synchronous one takes input and gives output in calls, and an asynchronous one reports through
//! events when it wants input and when output is ready. Both are handled.

use std::mem::ManuallyDrop;
use std::ptr;

use quokka_core::video::Codec;
use windows::Win32::Media::MediaFoundation::{
    IMFActivate, IMFMediaBuffer, IMFMediaEventGenerator, IMFMediaType, IMFSample, IMFTransform,
    MEDIA_EVENT_GENERATOR_GET_EVENT_FLAGS, MEError, METransformDrainComplete,
    METransformHaveOutput, METransformNeedInput, MF_E_NO_MORE_TYPES, MF_E_NOTACCEPTING,
    MF_E_TRANSFORM_NEED_MORE_INPUT, MF_E_TRANSFORM_STREAM_CHANGE, MF_E_TRANSFORM_TYPE_NOT_SET,
    MF_MT_DEFAULT_STRIDE, MF_MT_FRAME_SIZE, MF_MT_INTERLACE_MODE, MF_MT_MAJOR_TYPE,
    MF_MT_MPEG_SEQUENCE_HEADER, MF_MT_SUBTYPE, MF_TRANSFORM_ASYNC, MF_TRANSFORM_ASYNC_UNLOCK,
    MF_VERSION, MFCreateMediaType, MFCreateMemoryBuffer, MFCreateSample, MFMediaType_Video,
    MFSTARTUP_FULL, MFSampleExtension_CleanPoint, MFShutdown, MFStartup,
    MFT_CATEGORY_VIDEO_DECODER, MFT_ENUM_FLAG_ASYNCMFT, MFT_ENUM_FLAG_LOCALMFT,
    MFT_ENUM_FLAG_SORTANDFILTER, MFT_ENUM_FLAG_SYNCMFT, MFT_MESSAGE_COMMAND_DRAIN,
    MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, MFT_MESSAGE_NOTIFY_END_OF_STREAM,
    MFT_MESSAGE_NOTIFY_START_OF_STREAM, MFT_OUTPUT_DATA_BUFFER, MFT_OUTPUT_STREAM_PROVIDES_SAMPLES,
    MFT_REGISTER_TYPE_INFO, MFTEnumEx, MFVideoFormat_H264, MFVideoFormat_HEVC, MFVideoFormat_NV12,
    MFVideoFormat_P010, MFVideoInterlace_Progressive,
};
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoTaskMemFree};
use windows::core::{GUID, Interface};

use super::annexb::{self, NalFormat};
use super::semiplanar;
use super::{DecodeError, Packet, Picture, StreamConfig, Surface, VideoDecoder};

const BACKEND: &str = "Media Foundation";

/// How many times one frame is offered to the decoder before it is given up on. The loop only goes
/// round again after the decoder has been asked for output, so a healthy decoder never reaches it.
const MAX_RETRIES: usize = 64;

/// A Media Foundation decoding session for one stream.
pub struct MediaFoundation {
    transform: IMFTransform,
    /// Set for an asynchronous transform. Its events say when it wants input or has output.
    events: Option<IMFMediaEventGenerator>,
    /// The inputs an asynchronous transform has asked for and not yet been given.
    input_wanted: u32,
    /// Set once an asynchronous transform has finished draining.
    drained: bool,
    /// Whether the output type has been agreed. It can only be set once the decoder has seen the
    /// stream's parameter sets, so it is set after the first input.
    output_ready: bool,
    /// Bits per sample of the stream: 8, or 10 for HEVC Main 10.
    bit_depth: u8,
    /// The stream's size, from its container. Pictures are cropped to it.
    stream_width: usize,
    stream_height: usize,
    /// The decoded frame's size, from the output type. Frames are padded to whole blocks, so it can
    /// be larger than the stream.
    frame_width: usize,
    frame_height: usize,
    /// Bytes in a row of the decoded frame's luma, and the size of the buffer that holds one frame.
    stride: usize,
    output_size: u32,
    /// Whether the transform fills in its own output samples, rather than taking ours.
    provides_samples: bool,
    length_size: usize,
    /// The parameter sets as Annex B. Containers keep them in the configuration record, not in the
    /// packets, so they go in front of each keyframe.
    parameter_sets: Vec<u8>,
    /// Packets fed in, and pictures produced. A stream that gives no pictures is an error, not an
    /// empty result.
    packets_in: usize,
    frames_out: usize,
    pictures: Vec<Picture>,
    /// Declared last, so Media Foundation is shut down only after the transform is released.
    _startup: Startup,
}

impl MediaFoundation {
    /// Open a session for an H.264 or HEVC stream. Fails with an `Unsupported` error when Windows has
    /// no decoder for the codec, which for HEVC means the HEVC Video Extensions are not installed.
    pub fn open(stream: &StreamConfig) -> Result<Self, DecodeError> {
        let subtype = match stream.codec {
            Codec::H264 => MFVideoFormat_H264,
            Codec::Hevc => MFVideoFormat_HEVC,
            other => return Err(unsupported(format!("{} is not decoded here", other.name()))),
        };
        let NalFormat {
            length_size,
            parameter_sets,
        } = annexb::parse_record(stream.codec, &stream.config)
            .ok_or_else(|| unsupported("the stream's configuration record is malformed"))?;

        let startup = Startup::new()?;
        // COM must be set up on the thread that drives the transform. A thread that already has it
        // reports a different mode, which is harmless here, so the result is ignored.
        // SAFETY: no pointers are passed, and the result is not needed.
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };

        let activation =
            find_decoder(subtype)?.ok_or_else(|| unsupported(missing_decoder(stream.codec)))?;
        // SAFETY: the activation object is alive, and the transform it creates is owned by the result.
        let transform: IMFTransform = unsafe { activation.ActivateObject() }
            .map_err(|error| step_error("ActivateObject", error))?;
        let events = configure(&transform)?;

        let input = input_type(subtype, stream.width, stream.height, &parameter_sets)?;
        // SAFETY: the transform and the media type are valid for the call.
        unsafe { transform.SetInputType(0, &input, 0) }
            .map_err(|error| step_error("SetInputType", error))?;
        for (name, message) in [
            (
                "ProcessMessage(BEGIN_STREAMING)",
                MFT_MESSAGE_NOTIFY_BEGIN_STREAMING,
            ),
            (
                "ProcessMessage(START_OF_STREAM)",
                MFT_MESSAGE_NOTIFY_START_OF_STREAM,
            ),
        ] {
            // SAFETY: the transform is valid, and the message takes no parameter.
            unsafe { transform.ProcessMessage(message, 0) }
                .map_err(|error| step_error(name, error))?;
        }

        let mut decoder = MediaFoundation {
            transform,
            events,
            input_wanted: 0,
            drained: false,
            output_ready: false,
            bit_depth: stream.bit_depth,
            stream_width: stream.width as usize,
            stream_height: stream.height as usize,
            frame_width: 0,
            frame_height: 0,
            stride: 0,
            output_size: 0,
            provides_samples: false,
            length_size,
            parameter_sets,
            packets_in: 0,
            frames_out: 0,
            pictures: Vec::new(),
            _startup: startup,
        };
        // Some decoders refuse input until their output type is set, and the input type carries the
        // parameter sets, so offer the output types now. When none is offered yet, the first call to
        // `drain_outputs` agrees it.
        decoder.negotiate_output()?;
        Ok(decoder)
    }

    /// Wait until an asynchronous transform has asked for input. A synchronous one never waits.
    fn wait_for_input(&mut self) -> Result<(), DecodeError> {
        while self.events.is_some() && self.input_wanted == 0 {
            self.pump_event()?;
        }
        Ok(())
    }

    /// Wait for one event from an asynchronous transform and act on it. Does nothing for a
    /// synchronous one.
    fn pump_event(&mut self) -> Result<(), DecodeError> {
        let event = match &self.events {
            Some(events) => unsafe { events.GetEvent(MEDIA_EVENT_GENERATOR_GET_EVENT_FLAGS(0)) }
                .map_err(backend_error)?,
            None => return Ok(()),
        };
        // SAFETY: the event is alive for the call.
        let kind = unsafe { event.GetType() }.map_err(backend_error)?;
        if kind == METransformNeedInput.0 as u32 {
            self.input_wanted += 1;
        } else if kind == METransformHaveOutput.0 as u32 {
            self.drain_outputs()?;
        } else if kind == METransformDrainComplete.0 as u32 {
            self.drained = true;
        } else if kind == MEError.0 as u32 {
            return Err(failed("the decoder reported an error"));
        }
        Ok(())
    }

    /// Offer one input sample to the transform. When it refuses, the output is taken first, since
    /// the transform does not accept input while its output is full.
    fn submit(&mut self, sample: &IMFSample) -> Result<(), DecodeError> {
        for _ in 0..MAX_RETRIES {
            // SAFETY: the transform and the sample are alive for the call.
            match unsafe { self.transform.ProcessInput(0, sample, 0) } {
                Ok(()) => {
                    self.input_wanted = self.input_wanted.saturating_sub(1);
                    return Ok(());
                }
                Err(error) if error.code() == MF_E_NOTACCEPTING => {
                    self.drain_outputs()?;
                    self.pump_event()?;
                }
                Err(error) => return Err(backend_error(error)),
            }
        }
        Err(failed("the decoder would not accept a frame"))
    }

    /// Take every picture the transform can give now.
    fn drain_outputs(&mut self) -> Result<(), DecodeError> {
        // Asking for output is what makes the decoder report that it needs an output type, so this
        // runs even before one is agreed.
        loop {
            let ours = if self.provides_samples {
                None
            } else {
                Some(output_sample(self.output_size.max(1))?)
            };
            let mut buffers = [MFT_OUTPUT_DATA_BUFFER {
                dwStreamID: 0,
                pSample: ManuallyDrop::new(ours),
                dwStatus: 0,
                pEvents: ManuallyDrop::new(None),
            }];
            let mut status = 0u32;
            // SAFETY: the buffer and the status word are valid for the call. The transform fills
            // the sample it is given, which is then taken back out below.
            let result = unsafe { self.transform.ProcessOutput(0, &mut buffers, &mut status) };
            let [mut buffer] = buffers;
            // SAFETY: each field is taken out once, so it is released exactly once.
            let produced = unsafe { ManuallyDrop::take(&mut buffer.pSample) };
            let _events = unsafe { ManuallyDrop::take(&mut buffer.pEvents) };
            match result {
                Ok(()) => {
                    if !self.output_ready {
                        return Err(failed(
                            "the decoder produced a frame before its output type",
                        ));
                    }
                    if let Some(sample) = produced {
                        let picture = self.picture(&sample)?;
                        self.frames_out += 1;
                        self.pictures.push(picture);
                    }
                }
                Err(error) if error.code() == MF_E_TRANSFORM_NEED_MORE_INPUT => return Ok(()),
                // The decoder needs an output type, or the frame size changed: agree one, then go on.
                Err(error)
                    if error.code() == MF_E_TRANSFORM_TYPE_NOT_SET
                        || error.code() == MF_E_TRANSFORM_STREAM_CHANGE =>
                {
                    self.output_ready = false;
                    if !self.negotiate_output()? {
                        return Ok(());
                    }
                }
                Err(error) => return Err(backend_error(error)),
            }
        }
    }

    /// Choose the output type: NV12 for 8-bit, P010 for 10-bit. Returns `false` while the decoder
    /// has not yet seen enough of the stream to offer one.
    fn negotiate_output(&mut self) -> Result<bool, DecodeError> {
        let wanted = if self.bit_depth > 8 {
            MFVideoFormat_P010
        } else {
            MFVideoFormat_NV12
        };
        let mut index = 0;
        loop {
            // SAFETY: the transform is alive; the index is the only input.
            let media = match unsafe { self.transform.GetOutputAvailableType(0, index) } {
                Ok(media) => media,
                Err(error) if error.code() == MF_E_TRANSFORM_TYPE_NOT_SET => return Ok(false),
                Err(error) if error.code() == MF_E_NO_MORE_TYPES => {
                    return Err(unsupported(format!(
                        "the decoder has no {} output for this stream",
                        if self.bit_depth > 8 {
                            "10-bit"
                        } else {
                            "8-bit"
                        }
                    )));
                }
                Err(error) => return Err(step_error("GetOutputAvailableType", error)),
            };
            index += 1;
            // SAFETY: the media type is alive, and the key is a constant.
            let subtype = unsafe { media.GetGUID(&MF_MT_SUBTYPE) }
                .map_err(|error| step_error("GetGUID(SUBTYPE)", error))?;
            if subtype != wanted {
                continue;
            }
            // SAFETY: the transform and the offered type are valid for the call.
            unsafe { self.transform.SetOutputType(0, &media, 0) }
                .map_err(|error| step_error("SetOutputType", error))?;

            // The frame size is packed as width in the high 32 bits and height in the low 32.
            // SAFETY: the media type is alive, and the key is a constant.
            let packed = unsafe { media.GetUINT64(&MF_MT_FRAME_SIZE) }
                .map_err(|error| step_error("GetUINT64(FRAME_SIZE)", error))?;
            self.frame_width = (packed >> 32) as usize;
            self.frame_height = (packed & 0xFFFF_FFFF) as usize;
            let bytes = if self.bit_depth > 8 { 2 } else { 1 };
            // SAFETY: as above.
            self.stride = match unsafe { media.GetUINT32(&MF_MT_DEFAULT_STRIDE) } {
                Ok(stride) if (stride as i32) <= 0 => {
                    return Err(unsupported("the decoder stores its frames bottom-up"));
                }
                Ok(stride) => stride as usize,
                Err(_) => self.frame_width * bytes,
            };
            // SAFETY: the transform is alive.
            let info = unsafe { self.transform.GetOutputStreamInfo(0) }
                .map_err(|error| step_error("GetOutputStreamInfo", error))?;
            self.provides_samples =
                (info.dwFlags & MFT_OUTPUT_STREAM_PROVIDES_SAMPLES.0 as u32) != 0;
            // A frame holds its luma and half as much chroma again, so one and a half times the
            // stride times the height. The transform's own figure is used when it is larger.
            let frame_bytes =
                u32::try_from(self.stride * self.frame_height * 3 / 2).unwrap_or(u32::MAX);
            self.output_size = info.cbSize.max(frame_bytes);
            self.output_ready = true;
            return Ok(true);
        }
    }

    /// Copy one decoded frame out of its sample into a picture, cropped to the stream's size.
    fn picture(&self, sample: &IMFSample) -> Result<Picture, DecodeError> {
        // SAFETY: the sample is alive for the call.
        let presentation_time = unsafe { sample.GetSampleTime() }.map_err(backend_error)?;
        // SAFETY: as above. The buffer is one contiguous block, even if the transform returned several.
        let buffer = unsafe { sample.ConvertToContiguousBuffer() }.map_err(backend_error)?;
        let mut data: *mut u8 = ptr::null_mut();
        let mut length = 0u32;
        // SAFETY: the pointers are valid for the call. The buffer stays locked until it is unlocked
        // below, so the bytes read from `data` stay valid while they are read.
        unsafe {
            buffer.Lock(
                &mut data as *mut *mut u8,
                None,
                Some(&mut length as *mut u32),
            )
        }
        .map_err(backend_error)?;
        let converted = if data.is_null() {
            None
        } else {
            // SAFETY: `length` bytes at `data` are readable while the buffer is locked.
            let bytes = unsafe { std::slice::from_raw_parts(data, length as usize) };
            self.repack(bytes)
        };
        // SAFETY: the buffer was locked above.
        unsafe { buffer.Unlock() }.map_err(backend_error)?;

        let samples =
            converted.ok_or_else(|| failed("a decoded frame is shorter than its size"))?;
        let width = self.stream_width.min(self.frame_width);
        let height = self.stream_height.min(self.frame_height);
        let pixel_format = if self.bit_depth > 8 {
            *b"I010"
        } else {
            *b"I420"
        };
        Ok(Picture {
            width: width as u32,
            height: height as u32,
            presentation_time,
            pixel_format,
            surface: Surface::Cpu(samples),
        })
    }

    /// Repack the frame's bytes into planar samples, cropped to the stream's size.
    fn repack(&self, bytes: &[u8]) -> Option<Vec<u8>> {
        let width = self.stream_width.min(self.frame_width);
        let height = self.stream_height.min(self.frame_height);
        if self.bit_depth > 8 {
            semiplanar::p010_to_i010(bytes, self.stride, self.frame_height, width, height)
        } else {
            semiplanar::nv12_to_i420(bytes, self.stride, self.frame_height, width, height)
        }
    }
}

impl VideoDecoder for MediaFoundation {
    fn name(&self) -> &'static str {
        BACKEND
    }

    fn decode(&mut self, packet: &Packet) -> Result<(), DecodeError> {
        let frame = annexb::sample(&packet.data, self.length_size)
            .ok_or_else(|| failed("a packet's length prefixes run past its end"))?;
        if frame.is_empty() {
            return self.drain_outputs();
        }
        let data = if packet.sync {
            let mut keyframe = self.parameter_sets.clone();
            keyframe.extend_from_slice(&frame);
            keyframe
        } else {
            frame
        };
        self.packets_in += 1;
        let sample = input_sample(&data, packet)?;
        self.wait_for_input()?;
        self.submit(&sample)?;
        self.drain_outputs()
    }

    fn finish(&mut self) -> Result<(), DecodeError> {
        // SAFETY: the transform is alive, and these messages take no parameter.
        unsafe {
            self.transform
                .ProcessMessage(MFT_MESSAGE_NOTIFY_END_OF_STREAM, 0)
        }
        .map_err(backend_error)?;
        unsafe { self.transform.ProcessMessage(MFT_MESSAGE_COMMAND_DRAIN, 0) }
            .map_err(backend_error)?;
        while self.events.is_some() && !self.drained {
            self.pump_event()?;
        }
        self.drain_outputs()?;
        if self.packets_in > 0 && self.frames_out == 0 {
            return Err(failed(format!(
                "the decoder gave no frames for {} packets",
                self.packets_in
            )));
        }
        Ok(())
    }

    fn take_pictures(&mut self) -> Vec<Picture> {
        let mut pictures = std::mem::take(&mut self.pictures);
        pictures.sort_by_key(|picture| picture.presentation_time);
        pictures
    }
}

/// Media Foundation is started for each session and shut down when it ends. Media Foundation counts
/// the starts, so sessions may overlap.
struct Startup;

impl Startup {
    fn new() -> Result<Self, DecodeError> {
        // SAFETY: the version and flags are constants, and Media Foundation takes no pointers.
        unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL) }.map_err(backend_error)?;
        Ok(Startup)
    }
}

impl Drop for Startup {
    fn drop(&mut self) {
        // SAFETY: each `Startup` pairs with one successful `MFStartup`. A failed shutdown leaves
        // nothing to clean up here.
        let _ = unsafe { MFShutdown() };
    }
}

/// Find the first decoder Windows lists for a subtype. The list is sorted, so the one Windows
/// prefers comes first. `None` when there is none.
fn find_decoder(subtype: GUID) -> Result<Option<IMFActivate>, DecodeError> {
    let input = MFT_REGISTER_TYPE_INFO {
        guidMajorType: MFMediaType_Video,
        guidSubtype: subtype,
    };
    let flags = MFT_ENUM_FLAG_SYNCMFT
        | MFT_ENUM_FLAG_ASYNCMFT
        | MFT_ENUM_FLAG_LOCALMFT
        | MFT_ENUM_FLAG_SORTANDFILTER;
    let mut list: *mut Option<IMFActivate> = ptr::null_mut();
    let mut count = 0u32;
    // SAFETY: the input type lives for the call. On success Windows writes `count` entries at
    // `list`, which are taken out below and the array freed.
    unsafe {
        MFTEnumEx(
            MFT_CATEGORY_VIDEO_DECODER,
            flags,
            Some(&input as *const MFT_REGISTER_TYPE_INFO),
            None,
            &mut list,
            &mut count,
        )
    }
    .map_err(backend_error)?;
    if list.is_null() {
        return Ok(None);
    }
    // SAFETY: `list` holds `count` entries, as the call reported.
    let entries = unsafe { std::slice::from_raw_parts_mut(list, count as usize) };
    let first = entries.first_mut().and_then(Option::take);
    for entry in entries.iter_mut() {
        // Releases every entry the first one did not take.
        drop(entry.take());
    }
    // SAFETY: the array came from `MFTEnumEx`, which allocates it with the COM task allocator.
    unsafe { CoTaskMemFree(Some(list.cast_const().cast())) };
    Ok(first)
}

/// Asynchronous transforms are unlocked before use. Synchronous ones have no events, so `None`.
fn configure(transform: &IMFTransform) -> Result<Option<IMFMediaEventGenerator>, DecodeError> {
    // SAFETY: the transform is alive, and the attributes are read and set with constant keys.
    let asynchronous = match unsafe { transform.GetAttributes() } {
        Ok(attributes) => {
            let asynchronous =
                unsafe { attributes.GetUINT32(&MF_TRANSFORM_ASYNC) }.unwrap_or(0) != 0;
            if asynchronous {
                unsafe { attributes.SetUINT32(&MF_TRANSFORM_ASYNC_UNLOCK, 1) }
                    .map_err(backend_error)?;
            }
            asynchronous
        }
        Err(_) => false,
    };
    if !asynchronous {
        return Ok(None);
    }
    transform
        .cast::<IMFMediaEventGenerator>()
        .map(Some)
        .map_err(backend_error)
}

/// The input type: the codec, the stream's size, and its parameter sets as an Annex B sequence
/// header, so the decoder has them before the first keyframe.
fn input_type(
    subtype: GUID,
    width: u32,
    height: u32,
    parameter_sets: &[u8],
) -> Result<IMFMediaType, DecodeError> {
    // SAFETY: each call takes constants or the values built here, and the media type is alive for
    // every call.
    let media = unsafe { MFCreateMediaType() }.map_err(backend_error)?;
    unsafe { media.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video) }.map_err(backend_error)?;
    unsafe { media.SetGUID(&MF_MT_SUBTYPE, &subtype) }.map_err(backend_error)?;
    unsafe {
        media.SetUINT64(
            &MF_MT_FRAME_SIZE,
            (u64::from(width) << 32) | u64::from(height),
        )
    }
    .map_err(backend_error)?;
    unsafe { media.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32) }
        .map_err(backend_error)?;
    if !parameter_sets.is_empty() {
        unsafe { media.SetBlob(&MF_MT_MPEG_SEQUENCE_HEADER, parameter_sets) }
            .map_err(backend_error)?;
    }
    Ok(media)
}

/// An input sample holding one packet, marked as a keyframe when the packet is one.
fn input_sample(data: &[u8], packet: &Packet) -> Result<IMFSample, DecodeError> {
    let length = u32::try_from(data.len()).map_err(|_| failed("a packet is too large"))?;
    // SAFETY: the length is the size of the block to create.
    let buffer = unsafe { MFCreateMemoryBuffer(length) }.map_err(backend_error)?;
    let mut target: *mut u8 = ptr::null_mut();
    // SAFETY: the pointers are valid for the call. The buffer is locked while `data` is copied in,
    // and `length` bytes are writable there.
    unsafe { buffer.Lock(&mut target as *mut *mut u8, None, None) }.map_err(backend_error)?;
    if !target.is_null() {
        // SAFETY: `target` has room for `data.len()` bytes, and `data` is a separate allocation.
        unsafe { ptr::copy_nonoverlapping(data.as_ptr(), target, data.len()) };
    }
    // SAFETY: the buffer was locked above.
    unsafe { buffer.Unlock() }.map_err(backend_error)?;
    // SAFETY: the buffer is alive, and `length` bytes were written.
    unsafe { buffer.SetCurrentLength(length) }.map_err(backend_error)?;

    // SAFETY: the sample is alive for each call, and the buffer outlives the sample's use.
    let sample = unsafe { MFCreateSample() }.map_err(backend_error)?;
    unsafe { sample.AddBuffer(&buffer) }.map_err(backend_error)?;
    unsafe { sample.SetSampleTime(packet.presentation_time) }.map_err(backend_error)?;
    if packet.sync {
        unsafe { sample.SetUINT32(&MFSampleExtension_CleanPoint, 1) }.map_err(backend_error)?;
    }
    Ok(sample)
}

/// An output sample with a buffer of `size` bytes, for the transform to fill.
fn output_sample(size: u32) -> Result<IMFSample, DecodeError> {
    // SAFETY: the size is the capacity of the buffer to create.
    let buffer: IMFMediaBuffer = unsafe { MFCreateMemoryBuffer(size) }.map_err(backend_error)?;
    // SAFETY: the sample is alive for the call.
    let sample = unsafe { MFCreateSample() }.map_err(backend_error)?;
    unsafe { sample.AddBuffer(&buffer) }.map_err(backend_error)?;
    Ok(sample)
}

/// Why Windows has no decoder for the codec, for the user.
fn missing_decoder(codec: Codec) -> String {
    match codec {
        Codec::Hevc => {
            "HEVC needs the HEVC Video Extensions. Install them from the Microsoft Store, \
                        then play the file again"
                .into()
        }
        other => format!("Windows has no decoder for {}", other.name()),
    }
}

/// A failure from one named Media Foundation call, with its HRESULT, so a log says which call failed.
fn step_error(step: &str, error: windows::core::Error) -> DecodeError {
    DecodeError::Failed {
        backend: BACKEND,
        reason: format!("{step} returned {:#010X}", error.code().0 as u32),
    }
}

fn backend_error(error: windows::core::Error) -> DecodeError {
    DecodeError::Backend {
        backend: BACKEND,
        status: error.code().0,
    }
}

fn failed(reason: impl Into<String>) -> DecodeError {
    DecodeError::Failed {
        backend: BACKEND,
        reason: reason.into(),
    }
}

fn unsupported(reason: impl Into<String>) -> DecodeError {
    DecodeError::Unsupported {
        backend: BACKEND,
        reason: reason.into(),
    }
}
