//! A bundled AV1 decoder: rav1d, a BSD-2-Clause pure-Rust port of dav1d. rav1d exports dav1d's C
//! interface, so this file calls it through that interface. The structs it passes are mirrored
//! here from rav1d's own definitions, with only the fields this file reads or writes.

use std::ffi::{c_int, c_void};
use std::ptr;

use player_core::video::Codec;

use super::{DecodeError, Packet, Picture, StreamConfig, Surface, VideoDecoder, layout_code};

// Linking the crate is what makes its exported functions available.
use rav1d as _;

const BACKEND: &str = "rav1d";

/// The error code for "send the data again after taking pictures", as rav1d reports it.
#[cfg(target_os = "macos")]
const EAGAIN: c_int = 35;
#[cfg(not(target_os = "macos"))]
const EAGAIN: c_int = 11;

/// Pictures come back in whole layouts; a limit on the retries stops a stuck decoder.
const MAX_RETRIES: usize = 100;

/// Room for a `Dav1dSettings`, which is about 100 bytes. Zeroed, then filled by rav1d.
type SettingsBuffer = [u64; 32];
/// Room for a `Dav1dData`, which is about 100 bytes. Its first two fields are the pointer and size.
type DataBuffer = [u64; 32];
/// Where the timestamp sits in `Dav1dData`: the data props follow the buffer pointer, size and
/// reference, and the timestamp is their first field.
const DATA_PROPS_OFFSET: usize = 24;
/// Where the timestamp sits in `Dav1dPicture`: the props follow the seven fields before them.
const PICTURE_PROPS_OFFSET: usize = 72;
/// Room for a `Dav1dPicture`, which is a few hundred bytes.
type PictureBuffer = [u64; 64];

/// The head of rav1d's `Dav1dPicture`: the plane pointers and strides, then the parameters.
#[repr(C)]
struct PictureHead {
    sequence_header: *mut c_void,
    frame_header: *mut c_void,
    data: [*mut c_void; 3],
    stride: [isize; 2],
    width: c_int,
    height: c_int,
    layout: c_int,
    bpc: c_int,
}

type Context = *mut c_void;

unsafe extern "C" {
    fn dav1d_default_settings(settings: *mut c_void);
    fn dav1d_open(context_out: *mut Context, settings: *const c_void) -> c_int;
    fn dav1d_send_data(context: Context, data: *mut c_void) -> c_int;
    fn dav1d_get_picture(context: Context, picture: *mut c_void) -> c_int;
    fn dav1d_picture_unref(picture: *mut c_void);
    fn dav1d_data_create(data: *mut c_void, size: usize) -> *mut u8;
    fn dav1d_data_unref(data: *mut c_void);
    fn dav1d_flush(context: Context);
    fn dav1d_close(context_out: *mut Context);
}

/// An AV1 decoding session.
pub struct Av1 {
    context: Context,
    pictures: Vec<Picture>,
}

impl Av1 {
    /// Open a session for an AV1 stream. The configuration record's OBUs are sent first, so the
    /// sequence header is known before the first frame.
    pub fn open(stream: &StreamConfig) -> Result<Self, DecodeError> {
        if stream.codec != Codec::Av1 {
            return Err(DecodeError::Unsupported {
                backend: BACKEND,
                reason: "only AV1 is handled".into(),
            });
        }
        let mut settings: SettingsBuffer = [0; 32];
        let mut context: Context = ptr::null_mut();
        // SAFETY: the buffer is large enough for the settings struct, which rav1d fills in full.
        // We then set the first two fields (thread count and frame delay), which are `c_int`s at
        // offsets 0 and 4. The context is written by `dav1d_open` and checked below.
        unsafe {
            dav1d_default_settings(settings.as_mut_ptr().cast());
            let fields = settings.as_mut_ptr().cast::<c_int>();
            *fields = 1;
            *fields.add(1) = 1;
            let status = dav1d_open(&mut context, settings.as_ptr().cast());
            if status != 0 || context.is_null() {
                return Err(DecodeError::Unsupported {
                    backend: BACKEND,
                    reason: format!("dav1d_open returned {status}"),
                });
            }
        }
        let mut decoder = Av1 {
            context,
            pictures: Vec::new(),
        };
        // The configuration record is `av1C`: four header bytes, then the configuration OBUs.
        if stream.config.len() > 4 {
            let head = Packet {
                data: stream.config[4..].to_vec(),
                decode_time: 0,
                presentation_time: 0,
                sync: true,
            };
            decoder.submit(&head.data, 0)?;
        }
        Ok(decoder)
    }

    /// Send one buffer of OBUs to rav1d, then collect the pictures it can output. The timestamp
    /// travels with the data and comes back on the picture decoded from it.
    fn submit(&mut self, bytes: &[u8], timestamp: i64) -> Result<(), DecodeError> {
        let mut data: DataBuffer = [0; 32];
        let data_ptr = data.as_mut_ptr().cast::<c_void>();
        // SAFETY: rav1d allocates `bytes.len()` bytes and records them in the data struct. We
        // copy the packet into them, set its timestamp, then hand the struct to
        // `dav1d_send_data`, which takes ownership on success.
        unsafe {
            let buffer = dav1d_data_create(data_ptr, bytes.len());
            if buffer.is_null() {
                return Err(failed("could not allocate the packet".into()));
            }
            ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len());
            data.as_mut_ptr()
                .cast::<u8>()
                .add(DATA_PROPS_OFFSET)
                .cast::<i64>()
                .write(timestamp);
            for _ in 0..MAX_RETRIES {
                let status = dav1d_send_data(self.context, data_ptr);
                if status == 0 {
                    return self.collect();
                }
                if status != -EAGAIN {
                    dav1d_data_unref(data_ptr);
                    return Err(failed(format!("dav1d_send_data returned {status}")));
                }
                // rav1d needs its pictures taken before it takes more input.
                self.collect()?;
            }
            dav1d_data_unref(data_ptr);
        }
        Err(failed("the decoder did not accept the packet".into()))
    }

    /// Take every picture rav1d can output now.
    fn collect(&mut self) -> Result<(), DecodeError> {
        loop {
            let mut buffer: PictureBuffer = [0; 64];
            let picture = buffer.as_mut_ptr().cast::<c_void>();
            // SAFETY: the buffer holds a `Dav1dPicture`, which rav1d writes in full on success.
            let status = unsafe { dav1d_get_picture(self.context, picture) };
            if status == -EAGAIN {
                return Ok(());
            }
            if status != 0 {
                return Err(failed(format!("dav1d_get_picture returned {status}")));
            }
            // SAFETY: the picture is valid until `dav1d_picture_unref`, which runs below.
            let head = unsafe { &*(picture as *const PictureHead) };
            // SAFETY: the picture is valid until `dav1d_picture_unref`, and its props are inside it.
            let timestamp = unsafe {
                picture
                    .cast::<u8>()
                    .add(PICTURE_PROPS_OFFSET)
                    .cast::<i64>()
                    .read()
            };
            let converted = convert(head, timestamp);
            // SAFETY: we are done with the picture once its samples have been copied.
            unsafe { dav1d_picture_unref(picture) };
            self.pictures.push(converted);
        }
    }
}

/// Copy a picture's planes into tight rows, as the bundled decoders do.
fn convert(head: &PictureHead, timestamp: i64) -> Picture {
    let width = head.width as usize;
    let height = head.height as usize;
    let bytes_per_sample = if head.bpc > 8 { 2 } else { 1 };
    // rav1d layouts: 0 is 4:0:0, 1 is 4:2:0, 2 is 4:2:2 and 3 is 4:4:4.
    let (x, y) = match head.layout {
        1 => (1, 1),
        2 => (1, 0),
        _ => (0, 0),
    };
    let chroma_width = (width + x) >> x;
    let chroma_height = (height + y) >> y;
    let planes = [
        (width, height, 0usize),
        (chroma_width, chroma_height, 1),
        (chroma_width, chroma_height, 1),
    ];
    let mut samples = Vec::new();
    for (index, &(plane_width, plane_height, stride_index)) in planes.iter().enumerate() {
        let plane = head.data[index] as *const u8;
        if plane.is_null() || (index > 0 && head.layout == 0) {
            continue;
        }
        let stride = head.stride[stride_index];
        let row = plane_width * bytes_per_sample;
        for line in 0..plane_height {
            // SAFETY: rav1d guarantees each plane holds `plane_height` rows of `stride` bytes.
            let source = unsafe { plane.offset(line as isize * stride) };
            // SAFETY: the row is inside the plane, as above.
            samples.extend_from_slice(unsafe { std::slice::from_raw_parts(source, row) });
        }
    }
    Picture {
        width: head.width as u32,
        height: head.height as u32,
        presentation_time: timestamp,
        pixel_format: layout_code(x as u32, y as u32, head.bpc as u32),
        surface: Surface::Cpu(samples),
    }
}

fn failed(reason: String) -> DecodeError {
    DecodeError::Failed {
        backend: BACKEND,
        reason,
    }
}

impl VideoDecoder for Av1 {
    fn name(&self) -> &'static str {
        BACKEND
    }

    fn decode(&mut self, packet: &Packet) -> Result<(), DecodeError> {
        self.submit(&packet.data, packet.presentation_time)
    }

    fn finish(&mut self) -> Result<(), DecodeError> {
        // SAFETY: the context is valid until `Drop`.
        unsafe { dav1d_flush(self.context) };
        self.collect()
    }

    fn take_pictures(&mut self) -> Vec<Picture> {
        std::mem::take(&mut self.pictures)
    }
}

impl Drop for Av1 {
    fn drop(&mut self) {
        // SAFETY: the context came from `dav1d_open` and is closed once, here.
        unsafe { dav1d_close(&mut self.context) };
    }
}
