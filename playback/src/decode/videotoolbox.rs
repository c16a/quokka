//! VideoToolbox, Apple's operating-system decoder. It handles H.264 and HEVC on the Macs this runs
//! on, and AV1 on Macs with hardware AV1 decoding. The decoder is the system's own. This file calls
//! it through the C interface of the system frameworks, so it needs no third-party code.

use std::ffi::c_void;
use std::ptr;
use std::sync::Mutex;

use quokka_core::video::{Chroma, Codec};

use super::{DecodeError, Packet, Picture, StreamConfig, Surface, VideoDecoder};

const BACKEND: &str = "VideoToolbox";

/// A Core Foundation, Core Media or Core Video object, or a null pointer.
type Handle = *mut c_void;
type OSStatus = i32;

const UTF8: u32 = 0x0800_0100;
const TIME_FLAG_VALID: u32 = 1;
const CODEC_H264: u32 = u32::from_be_bytes(*b"avc1");
const CODEC_HEVC: u32 = u32::from_be_bytes(*b"hvc1");
const CODEC_AV1: u32 = u32::from_be_bytes(*b"av01");

/// A Core Media time: a count of ticks, and how many ticks make a second.
#[repr(C)]
#[derive(Clone, Copy)]
struct CMTime {
    value: i64,
    timescale: i32,
    flags: u32,
    epoch: i64,
}

impl CMTime {
    /// A time that is not set.
    const INVALID: CMTime = CMTime {
        value: 0,
        timescale: 0,
        flags: 0,
        epoch: 0,
    };

    fn new(value: i64, timescale: i32) -> Self {
        CMTime {
            value,
            timescale,
            flags: TIME_FLAG_VALID,
            epoch: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CMSampleTimingInfo {
    duration: CMTime,
    presentation_time_stamp: CMTime,
    decode_time_stamp: CMTime,
}

/// The function VideoToolbox calls with each decoded picture.
type OutputCallback = extern "C" fn(
    refcon: *mut c_void,
    source_refcon: *mut c_void,
    status: OSStatus,
    info_flags: u32,
    image: Handle,
    presentation: CMTime,
    duration: CMTime,
);

#[repr(C)]
struct OutputCallbackRecord {
    callback: OutputCallback,
    refcon: *mut c_void,
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(object: Handle);
    fn CFRetain(object: Handle) -> Handle;
    fn CFDataCreate(allocator: *const c_void, bytes: *const u8, length: isize) -> Handle;
    fn CFStringCreateWithBytes(
        allocator: *const c_void,
        bytes: *const u8,
        length: isize,
        encoding: u32,
        is_external: u8,
    ) -> Handle;
    fn CFNumberCreate(allocator: *const c_void, number_type: isize, value: *const c_void)
    -> Handle;
    fn CFDictionaryCreate(
        allocator: *const c_void,
        keys: *const Handle,
        values: *const Handle,
        count: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> Handle;
    static kCFTypeDictionaryKeyCallBacks: u8;
    static kCFTypeDictionaryValueCallBacks: u8;
}

#[link(name = "CoreMedia", kind = "framework")]
unsafe extern "C" {
    static kCMFormatDescriptionExtension_SampleDescriptionExtensionAtoms: Handle;
    fn CMVideoFormatDescriptionCreate(
        allocator: *const c_void,
        codec: u32,
        width: i32,
        height: i32,
        extensions: Handle,
        format_out: *mut Handle,
    ) -> OSStatus;
    fn CMBlockBufferCreateWithMemoryBlock(
        allocator: *const c_void,
        memory: *mut c_void,
        length: usize,
        block_allocator: *const c_void,
        custom_source: *const c_void,
        offset: usize,
        data_length: usize,
        flags: u32,
        block_out: *mut Handle,
    ) -> OSStatus;
    fn CMBlockBufferReplaceDataBytes(
        source: *const c_void,
        destination: Handle,
        offset: usize,
        length: usize,
    ) -> OSStatus;
    fn CMSampleBufferCreateReady(
        allocator: *const c_void,
        data: Handle,
        format: Handle,
        samples: isize,
        timing_entries: isize,
        timing: *const CMSampleTimingInfo,
        size_entries: isize,
        sizes: *const usize,
        sample_out: *mut Handle,
    ) -> OSStatus;
}

#[link(name = "VideoToolbox", kind = "framework")]
unsafe extern "C" {
    fn VTIsHardwareDecodeSupported(codec: u32) -> u8;
    fn VTDecompressionSessionCreate(
        allocator: *const c_void,
        format: Handle,
        specification: Handle,
        attributes: Handle,
        callback: *const OutputCallbackRecord,
        session_out: *mut Handle,
    ) -> OSStatus;
    fn VTDecompressionSessionDecodeFrame(
        session: Handle,
        sample: Handle,
        flags: u32,
        source_refcon: *mut c_void,
        info_flags_out: *mut u32,
    ) -> OSStatus;
    fn VTDecompressionSessionWaitForAsynchronousFrames(session: Handle) -> OSStatus;
    fn VTDecompressionSessionInvalidate(session: Handle);
}

#[link(name = "CoreVideo", kind = "framework")]
unsafe extern "C" {
    static kCVPixelBufferPixelFormatTypeKey: Handle;
    fn CVPixelBufferGetWidth(buffer: Handle) -> usize;
    fn CVPixelBufferGetHeight(buffer: Handle) -> usize;
    fn CVPixelBufferGetPixelFormatType(buffer: Handle) -> u32;
    fn CVPixelBufferLockBaseAddress(buffer: Handle, flags: u64) -> OSStatus;
    fn CVPixelBufferUnlockBaseAddress(buffer: Handle, flags: u64) -> OSStatus;
    fn CVPixelBufferGetPlaneCount(buffer: Handle) -> usize;
    fn CVPixelBufferGetWidthOfPlane(buffer: Handle, plane: usize) -> usize;
    fn CVPixelBufferGetHeightOfPlane(buffer: Handle, plane: usize) -> usize;
    fn CVPixelBufferGetBaseAddressOfPlane(buffer: Handle, plane: usize) -> *mut u8;
    fn CVPixelBufferGetBytesPerRowOfPlane(buffer: Handle, plane: usize) -> usize;
}

/// The 8-bit biplanar pixel formats, as four-character codes. Others are taken to be 16-bit.
const EIGHT_BIT_FORMATS: [u32; 6] = [
    u32::from_be_bytes(*b"420v"),
    u32::from_be_bytes(*b"420f"),
    u32::from_be_bytes(*b"422v"),
    u32::from_be_bytes(*b"422f"),
    u32::from_be_bytes(*b"444v"),
    u32::from_be_bytes(*b"444f"),
];

/// Lock the buffer read-only, as the Core Video calls expect.
const LOCK_READ_ONLY: u64 = 1;

/// A decoded picture buffer, kept alive until it is dropped.
pub struct PixelBuffer(Handle);

// SAFETY: a retained Core Video pixel buffer is immutable from our side and may be released from
// any thread.
unsafe impl Send for PixelBuffer {}
// SAFETY: as above; its planes are only read, and reads may happen on any number of threads.
unsafe impl Sync for PixelBuffer {}

impl Clone for PixelBuffer {
    fn clone(&self) -> Self {
        // SAFETY: the buffer is alive while we hold our reference, so retaining it is valid.
        PixelBuffer(unsafe { CFRetain(self.0) })
    }
}

impl std::fmt::Debug for PixelBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PixelBuffer")
    }
}

impl PixelBuffer {
    /// The Core Video buffer itself, for handing to other Core Video or Metal calls. The buffer
    /// stays owned by this value.
    pub fn as_raw(&self) -> *const c_void {
        self.0
    }

    /// The picture's planes copied out, row by row, one after another, with no padding. The
    /// hardware decoder's pictures are biplanar: a luma plane, then a plane of interleaved chroma
    /// pairs.
    pub fn to_bytes(&self) -> Vec<u8> {
        // SAFETY: the buffer is retained by us, and locked for reading while its planes are read.
        // Every plane's rows are inside its bytes-per-row span, as Core Video describes them. When
        // a plane's bytes-per-row is its row width, the plane is one span of `row * height` bytes.
        unsafe {
            let bytes_per_sample =
                if EIGHT_BIT_FORMATS.contains(&CVPixelBufferGetPixelFormatType(self.0)) {
                    1
                } else {
                    2
                };
            // Chroma planes hold a pair of samples for each column.
            let components = |plane: usize| -> usize { if plane == 0 { 1 } else { 2 } };
            CVPixelBufferLockBaseAddress(self.0, LOCK_READ_ONLY);
            let planes = CVPixelBufferGetPlaneCount(self.0);
            let total: usize = (0..planes)
                .filter(|&plane| !CVPixelBufferGetBaseAddressOfPlane(self.0, plane).is_null())
                .map(|plane| {
                    let width = CVPixelBufferGetWidthOfPlane(self.0, plane);
                    let height = CVPixelBufferGetHeightOfPlane(self.0, plane);
                    width * height * components(plane) * bytes_per_sample
                })
                .sum();
            let mut out = Vec::with_capacity(total);
            for plane in 0..planes {
                let width = CVPixelBufferGetWidthOfPlane(self.0, plane);
                let height = CVPixelBufferGetHeightOfPlane(self.0, plane);
                let base = CVPixelBufferGetBaseAddressOfPlane(self.0, plane);
                let stride = CVPixelBufferGetBytesPerRowOfPlane(self.0, plane);
                let row = width * components(plane) * bytes_per_sample;
                if base.is_null() {
                    continue;
                }
                if stride == row {
                    out.extend_from_slice(std::slice::from_raw_parts(base, row * height));
                } else {
                    for line in 0..height {
                        out.extend_from_slice(std::slice::from_raw_parts(
                            base.add(line * stride),
                            row,
                        ));
                    }
                }
            }
            CVPixelBufferUnlockBaseAddress(self.0, LOCK_READ_ONLY);
            out
        }
    }
}

impl Drop for PixelBuffer {
    fn drop(&mut self) {
        // SAFETY: we hold one retain on the buffer, taken when VideoToolbox delivered it.
        unsafe { CFRelease(self.0) }
    }
}

/// What the output callback fills in. It sits on the heap, so its address is stable.
#[derive(Default)]
struct Output {
    pictures: Vec<Picture>,
    failures: Vec<OSStatus>,
}

/// A VideoToolbox decoding session for one stream.
pub struct VideoToolbox {
    session: Handle,
    format: Handle,
    timescale: i32,
    output: Box<Mutex<Output>>,
}

impl VideoToolbox {
    /// Open a session for the stream. Fails when VideoToolbox does not decode the codec, or the
    /// stream's parameters are not ones it accepts.
    pub fn open(stream: &StreamConfig) -> Result<Self, DecodeError> {
        let (codec, record_key) = match stream.codec {
            Codec::H264 => (CODEC_H264, "avcC"),
            Codec::Hevc => (CODEC_HEVC, "hvcC"),
            // AV1 only on machines whose hardware decodes it. Elsewhere VideoToolbox would decode in
            // software, which the software decoder does already.
            Codec::Av1 if Self::hardware_supported(Codec::Av1) => (CODEC_AV1, "av1C"),
            other => {
                return Err(refused(format!("{} is not decoded here", other.name())));
            }
        };
        let timescale = i32::try_from(stream.timescale)
            .map_err(|_| refused("the time scale is too large".into()))?;
        let output_format = output_format(
            stream.bit_depth,
            stream.chroma,
            stream.colour.full_range.unwrap_or(false),
        );

        // SAFETY: every pointer passed to the frameworks is either valid for the call or null, and
        // every object created here is released on each path.
        let format = unsafe { format_description(codec, record_key, stream) }?;

        let output = Box::new(Mutex::new(Output::default()));
        let record = OutputCallbackRecord {
            callback: on_output,
            refcon: &*output as *const Mutex<Output> as *mut c_void,
        };
        let mut session = ptr::null_mut();
        // SAFETY: `record` and `session` are valid for the call. The output box is owned by the
        // returned value, which invalidates the session before the box is dropped.
        let status = unsafe {
            let attributes = match output_format {
                Some(format) => output_attributes(format),
                None => ptr::null_mut(),
            };
            let status = VTDecompressionSessionCreate(
                ptr::null(),
                format,
                ptr::null_mut(),
                attributes,
                &record,
                &mut session,
            );
            if !attributes.is_null() {
                CFRelease(attributes);
            }
            status
        };
        if status != 0 || session.is_null() {
            // SAFETY: `format` was created above and is not used again.
            unsafe { CFRelease(format) };
            return Err(refused(format!(
                "VTDecompressionSessionCreate returned {status}"
            )));
        }
        Ok(VideoToolbox {
            session,
            format,
            timescale,
            output,
        })
    }

    /// Whether the hardware decodes the codec on this machine.
    pub fn hardware_supported(codec: Codec) -> bool {
        let codec = match codec {
            Codec::H264 => CODEC_H264,
            Codec::Hevc => CODEC_HEVC,
            Codec::Av1 => CODEC_AV1,
            _ => return false,
        };
        // SAFETY: the function takes a codec type and returns a flag.
        unsafe { VTIsHardwareDecodeSupported(codec) != 0 }
    }
}

/// The pixel format to ask VideoToolbox for. Its own format for 10-bit pictures is not laid out
/// in memory the way the frameworks describe, so a linear format is requested and VideoToolbox
/// converts into it. `None` keeps the default for 8-bit 4:2:0.
fn output_format(bit_depth: u8, chroma: Chroma, full_range: bool) -> Option<u32> {
    // A full-range stream gets a full-range picture: asking for limited levels would have the
    // framework scale it down.
    let code: &[u8; 4] = match (bit_depth, chroma, full_range) {
        (8, Chroma::Yuv420, false) => b"420v",
        (8, Chroma::Yuv422, false) => b"422v",
        (8, Chroma::Yuv444, false) => b"444v",
        (8, Chroma::Yuv420, true) => b"420f",
        (8, Chroma::Yuv422, true) => b"422f",
        (8, Chroma::Yuv444, true) => b"444f",
        (10, Chroma::Yuv420, false) => b"x420",
        (10, Chroma::Yuv422, false) => b"x422",
        (10, Chroma::Yuv444, false) => b"x444",
        (10, Chroma::Yuv420, true) => b"xf20",
        (10, Chroma::Yuv422, true) => b"xf22",
        (10, Chroma::Yuv444, true) => b"xf44",
        _ => return None,
    };
    Some(u32::from_be_bytes(*code))
}

/// A destination attribute dictionary that asks for one pixel format. The caller releases it.
///
/// # Safety
/// The returned dictionary must be released once, with `CFRelease`.
unsafe fn output_attributes(format: u32) -> Handle {
    // SAFETY: the number, key and dictionary are created here and only read by the framework.
    unsafe {
        let number = CFNumberCreate(ptr::null(), 3, &format as *const u32 as *const c_void);
        let keys = [kCVPixelBufferPixelFormatTypeKey];
        let values = [number];
        let dictionary = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &raw const kCFTypeDictionaryKeyCallBacks as *const c_void,
            &raw const kCFTypeDictionaryValueCallBacks as *const c_void,
        );
        CFRelease(number);
        dictionary
    }
}

/// A format description for the codec, carrying the configuration record the stream stores.
///
/// # Safety
/// The caller must not use the returned handle after releasing it, and must release it once.
unsafe fn format_description(
    codec: u32,
    record_key: &str,
    stream: &StreamConfig,
) -> Result<Handle, DecodeError> {
    // SAFETY: each object is created here, used for the call, and released before returning.
    unsafe {
        let record = CFDataCreate(
            ptr::null(),
            stream.config.as_ptr(),
            stream.config.len() as isize,
        );
        let key = CFStringCreateWithBytes(
            ptr::null(),
            record_key.as_ptr(),
            record_key.len() as isize,
            UTF8,
            0,
        );
        let inner_keys = [key];
        let inner_values = [record];
        let inner = CFDictionaryCreate(
            ptr::null(),
            inner_keys.as_ptr(),
            inner_values.as_ptr(),
            1,
            &raw const kCFTypeDictionaryKeyCallBacks as *const c_void,
            &raw const kCFTypeDictionaryValueCallBacks as *const c_void,
        );

        let outer_keys = [kCMFormatDescriptionExtension_SampleDescriptionExtensionAtoms];
        let outer_values = [inner];
        let extensions = CFDictionaryCreate(
            ptr::null(),
            outer_keys.as_ptr(),
            outer_values.as_ptr(),
            1,
            &raw const kCFTypeDictionaryKeyCallBacks as *const c_void,
            &raw const kCFTypeDictionaryValueCallBacks as *const c_void,
        );

        let mut format = ptr::null_mut();
        let status = CMVideoFormatDescriptionCreate(
            ptr::null(),
            codec,
            stream.width as i32,
            stream.height as i32,
            extensions,
            &mut format,
        );
        CFRelease(extensions);
        CFRelease(inner);
        CFRelease(key);
        CFRelease(record);
        if status != 0 || format.is_null() {
            return Err(refused(format!(
                "CMVideoFormatDescriptionCreate returned {status}"
            )));
        }
        Ok(format)
    }
}

/// The callback VideoToolbox calls for each decoded picture, on its own thread.
extern "C" fn on_output(
    refcon: *mut c_void,
    _source_refcon: *mut c_void,
    status: OSStatus,
    _info_flags: u32,
    image: Handle,
    presentation: CMTime,
    _duration: CMTime,
) {
    // SAFETY: `refcon` is the address of the session's output box, which outlives the session.
    let Some(output) = (unsafe { (refcon as *const Mutex<Output>).as_ref() }) else {
        return;
    };
    let Ok(mut output) = output.lock() else {
        return;
    };
    if status != 0 || image.is_null() {
        output.failures.push(status);
        return;
    }
    // SAFETY: `image` is a valid pixel buffer for the duration of the callback. Retaining it keeps
    // it alive after the callback returns, and `PixelBuffer` releases it.
    let (width, height, format, retained) = unsafe {
        (
            CVPixelBufferGetWidth(image) as u32,
            CVPixelBufferGetHeight(image) as u32,
            CVPixelBufferGetPixelFormatType(image).to_be_bytes(),
            CFRetain(image),
        )
    };
    output.pictures.push(Picture {
        width,
        height,
        presentation_time: presentation.value,
        pixel_format: format,
        surface: Surface::Platform(PixelBuffer(retained)),
    });
}

impl VideoDecoder for VideoToolbox {
    fn name(&self) -> &'static str {
        BACKEND
    }

    fn decode(&mut self, packet: &Packet) -> Result<(), DecodeError> {
        let length = packet.data.len();
        let mut block = ptr::null_mut();
        // SAFETY: the block is allocated by the framework with `length` bytes, then filled from
        // `packet.data`, which is valid for `length` bytes.
        unsafe {
            check(CMBlockBufferCreateWithMemoryBlock(
                ptr::null(),
                ptr::null_mut(),
                length,
                ptr::null(),
                ptr::null(),
                0,
                length,
                0,
                &mut block,
            ))?;
            let status =
                CMBlockBufferReplaceDataBytes(packet.data.as_ptr().cast(), block, 0, length);
            if status != 0 {
                CFRelease(block);
                return Err(backend(status));
            }
        }

        let timing = CMSampleTimingInfo {
            duration: CMTime::INVALID,
            presentation_time_stamp: CMTime::new(packet.presentation_time, self.timescale),
            decode_time_stamp: CMTime::new(packet.decode_time, self.timescale),
        };
        let size = length;
        let mut sample = ptr::null_mut();
        // SAFETY: the timing and size arrays hold one entry each, and live for the call.
        let status = unsafe {
            CMSampleBufferCreateReady(
                ptr::null(),
                block,
                self.format,
                1,
                1,
                &timing,
                1,
                &size,
                &mut sample,
            )
        };
        // SAFETY: the sample holds its own reference to the block, so ours can go.
        unsafe { CFRelease(block) };
        check(status)?;

        let mut info_flags = 0;
        // SAFETY: the session and sample are valid, and the output box lives as long as the session.
        let status = unsafe {
            VTDecompressionSessionDecodeFrame(
                self.session,
                sample,
                0,
                ptr::null_mut(),
                &mut info_flags,
            )
        };
        // SAFETY: the sample is no longer needed once the frame has been submitted.
        unsafe { CFRelease(sample) };
        check(status)
    }

    fn finish(&mut self) -> Result<(), DecodeError> {
        // SAFETY: the session is valid until `Drop`.
        check(unsafe { VTDecompressionSessionWaitForAsynchronousFrames(self.session) })?;
        let failure = self
            .output
            .lock()
            .ok()
            .and_then(|output| output.failures.first().copied());
        match failure {
            Some(status) => Err(backend(status)),
            None => Ok(()),
        }
    }

    fn take_pictures(&mut self) -> Vec<Picture> {
        let Ok(mut output) = self.output.lock() else {
            return Vec::new();
        };
        let mut pictures = std::mem::take(&mut output.pictures);
        pictures.sort_by_key(|picture| picture.presentation_time);
        pictures
    }
}

impl Drop for VideoToolbox {
    fn drop(&mut self) {
        // SAFETY: the session and format are valid, and not used after this. Invalidating first
        // stops the callback, so the output box can be freed after it.
        unsafe {
            VTDecompressionSessionInvalidate(self.session);
            CFRelease(self.session);
            CFRelease(self.format);
        }
    }
}

fn check(status: OSStatus) -> Result<(), DecodeError> {
    if status == 0 {
        Ok(())
    } else {
        Err(backend(status))
    }
}

fn backend(status: OSStatus) -> DecodeError {
    DecodeError::Backend {
        backend: BACKEND,
        status,
    }
}

fn refused(reason: String) -> DecodeError {
    DecodeError::Unsupported {
        backend: BACKEND,
        reason,
    }
}
