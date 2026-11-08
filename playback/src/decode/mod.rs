//! Video decoding behind one trait. A decoder takes compressed frames and returns decoded pictures.
//! Each backend accepts or refuses a stream when it opens. The first backend that accepts it
//! decodes it: the operating system's decoder first, then a bundled decoder for what the system
//! does not handle.

pub mod audio;
pub mod av1;
pub mod frames;
#[cfg(not(any(target_os = "macos", windows)))]
pub mod libav;
#[cfg(windows)]
pub mod mediafoundation;
pub mod player;
#[cfg(target_os = "macos")]
pub mod videotoolbox;
pub mod vp9;

// Helpers for the Windows backend. They are plain byte handling, so they build and are tested on
// every platform.
#[cfg(any(windows, test))]
mod annexb;
#[cfg(any(windows, test))]
mod semiplanar;

use quokka_core::video::{Chroma, Codec, Colour};
use thiserror::Error;

/// What a decoder needs to know about a stream before it opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamConfig {
    pub codec: Codec,
    pub width: u32,
    pub height: u32,
    /// The codec's configuration record: `avcC` or `hvcC` body.
    pub config: Vec<u8>,
    /// Ticks per second for the packets' times.
    pub timescale: u64,
    /// Bits per sample, as the stream states them.
    pub bit_depth: u8,
    /// The chroma subsampling, as the stream states it.
    pub chroma: Chroma,
    /// The colour range and matrix, as the stream states them.
    pub colour: Colour,
}

/// One compressed frame, with its times in the stream's ticks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub data: Vec<u8>,
    pub decode_time: i64,
    pub presentation_time: i64,
    pub sync: bool,
}

/// A decoded picture.
pub struct Picture {
    pub width: u32,
    pub height: u32,
    /// The presentation time, in the stream's ticks.
    pub presentation_time: i64,
    /// The pixel format as a four-character code, such as `420v` for 8-bit 4:2:0 in video range.
    pub pixel_format: [u8; 4],
    pub surface: Surface,
}

/// Where a decoded picture's samples live.
pub enum Surface {
    /// A platform buffer, owned by the decoder's platform and released when dropped.
    #[cfg(target_os = "macos")]
    Platform(videotoolbox::PixelBuffer),
    /// Samples in system memory, for software decoders.
    Cpu(Vec<u8>),
}

#[derive(Debug, Error)]
pub enum DecodeError {
    /// This backend does not handle the stream.
    #[error("{backend} does not decode this stream: {reason}")]
    Unsupported {
        backend: &'static str,
        reason: String,
    },
    /// The backend failed while decoding.
    #[error("{backend} failed (status {status})")]
    Backend { backend: &'static str, status: i32 },
    /// The backend failed, with its own description.
    #[error("{backend} failed: {reason}")]
    Failed {
        backend: &'static str,
        reason: String,
    },
}

/// A video decoder. Frames go in with [`decode`](VideoDecoder::decode); pictures come out with
/// [`take_pictures`](VideoDecoder::take_pictures), in presentation order once
/// [`finish`](VideoDecoder::finish) has run.
pub trait VideoDecoder {
    /// The backend's name, for logs and the info panel.
    fn name(&self) -> &'static str;

    /// Send one compressed frame.
    fn decode(&mut self, packet: &Packet) -> Result<(), DecodeError>;

    /// Wait for every frame in flight to decode.
    fn finish(&mut self) -> Result<(), DecodeError>;

    /// The pictures decoded so far. They are taken out of the decoder.
    fn take_pictures(&mut self) -> Vec<Picture>;
}

/// Open the first backend that accepts the stream. The operating system's decoder is tried first.
/// Bundled decoders cover what it does not handle: VP9 and AV1 on every platform, and H.264 and HEVC
/// on Linux, through libavcodec. On Windows, Media Foundation decodes H.264 and HEVC, and nothing
/// bundled takes them over; ProRes is not decoded there.
pub fn open(stream: &StreamConfig) -> Result<Box<dyn VideoDecoder>, DecodeError> {
    #[cfg(target_os = "macos")]
    let system = videotoolbox::VideoToolbox::open(stream)
        .map(|decoder| Box::new(decoder) as Box<dyn VideoDecoder>);
    #[cfg(windows)]
    let system = mediafoundation::MediaFoundation::open(stream)
        .map(|decoder| Box::new(decoder) as Box<dyn VideoDecoder>);
    #[cfg(not(any(target_os = "macos", windows)))]
    let system: Result<Box<dyn VideoDecoder>, DecodeError> = Err(DecodeError::Unsupported {
        backend: "system",
        reason: "no system decoder on this platform".into(),
    });
    let system_error = match system {
        Ok(decoder) => return Ok(decoder),
        Err(error) => error,
    };
    match stream.codec {
        #[cfg(not(any(target_os = "macos", windows)))]
        Codec::H264 | Codec::Hevc => {
            libav::Libav::open(stream).map(|decoder| Box::new(decoder) as Box<dyn VideoDecoder>)
        }
        Codec::Vp9 => {
            vp9::Vp9::open(stream).map(|decoder| Box::new(decoder) as Box<dyn VideoDecoder>)
        }
        Codec::Av1 => {
            av1::Av1::open(stream).map(|decoder| Box::new(decoder) as Box<dyn VideoDecoder>)
        }
        _ => Err(system_error),
    }
}

/// The planar layout as a four-character code: `I420` for 8-bit 4:2:0, `I010` for 10-bit 4:2:0,
/// and so on. Samples above 8 bits are 16-bit little-endian.
pub(crate) fn layout_code(x: u32, y: u32, bit_depth: u32) -> [u8; 4] {
    match (bit_depth, x, y) {
        (8, 1, 1) => *b"I420",
        (8, 1, 0) => *b"I422",
        (8, 0, 0) => *b"I444",
        (10, 1, 1) => *b"I010",
        (10, 1, 0) => *b"I210",
        (10, 0, 0) => *b"I410",
        (12, 1, 1) => *b"I012",
        (12, 1, 0) => *b"I212",
        (12, 0, 0) => *b"I412",
        _ => *b"????",
    }
}
