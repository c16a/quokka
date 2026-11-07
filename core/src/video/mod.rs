//! The format of any video file: its container, codec, picture size, display aspect ratio, colour
//! depth, chroma subsampling and frame rate.
//!
//! [`inspect`] reads the file's own bytes, not its name. It understands MP4 (and MOV), Matroska
//! and WebM, and the raw AV1 OBU, H.264 Annex B and HEVC Annex B streams. The codecs are AV1,
//! H.264, HEVC and VP9. It needs no codec library at run time: the bytes are read here.
//!
//! Where a file states a value more than once, the codec's own headers win for the picture size,
//! colour depth and chroma subsampling. The container wins for the display aspect ratio and the
//! frame rate, because those are what players show.

mod audio;
mod av1;
mod bits;
mod bmff;
mod ebml;
mod h264;
mod hevc;
mod matroska;
mod mp4;
mod raw;
mod samples;
mod vp9;

pub use audio::{AudioCodec, AudioTrack};
pub use samples::{Sample, SampleTrack, read_audio_samples, read_samples};

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::Result;

/// How many bytes of a file are read into memory when it is not an MP4. Headers come at the start
/// of a stream or a Matroska file.
const HEAD_LIMIT: u64 = 4 * 1024 * 1024;

/// The first four bytes of every EBML file, Matroska and WebM included.
const EBML_MAGIC: [u8; 4] = [0x1A, 0x45, 0xDF, 0xA3];

/// The container a file uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    /// MP4 or MOV.
    Mp4,
    /// Matroska or WebM.
    Matroska,
    /// A raw AV1 stream of OBUs, as `.obu` files hold.
    RawObu,
    /// A raw H.264 or HEVC stream with start codes, as `.h264` and `.hevc` files hold.
    AnnexB,
}

/// The video codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Av1,
    H264,
    Hevc,
    Vp9,
}

impl Codec {
    /// The name people know the codec by.
    pub fn name(self) -> &'static str {
        match self {
            Codec::Av1 => "AV1",
            Codec::H264 => "H.264",
            Codec::Hevc => "HEVC",
            Codec::Vp9 => "VP9",
        }
    }
}

/// How the colour samples are subsampled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chroma {
    Monochrome,
    Yuv420,
    Yuv422,
    Yuv444,
}

impl Chroma {
    /// The notation people know the subsampling by, such as `4:2:0`.
    pub fn name(self) -> &'static str {
        match self {
            Chroma::Monochrome => "4:0:0",
            Chroma::Yuv420 => "4:2:0",
            Chroma::Yuv422 => "4:2:2",
            Chroma::Yuv444 => "4:4:4",
        }
    }
}

/// A frame rate: frames per second, as a fraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRate {
    pub numerator: u64,
    pub denominator: u64,
}

impl FrameRate {
    /// The rate as a number of frames per second.
    pub fn per_second(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    /// The fraction `numerator / denominator`, reduced. `None` for zero.
    pub(crate) fn new(numerator: u64, denominator: u64) -> Option<Self> {
        if numerator == 0 || denominator == 0 {
            return None;
        }
        let divisor = gcd(numerator, denominator);
        Some(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    /// The fraction nearest to a measured rate, with a denominator of at most `max_denominator`.
    /// A measured rate is never exact, so this finds the rate it was measured from, such as
    /// 24000/1001 for 23.976.
    pub(crate) fn approximate(value: f64, max_denominator: u64) -> Option<Self> {
        if !value.is_finite() || value <= 0.0 {
            return None;
        }
        let mut best: Option<(f64, FrameRate)> = None;
        for denominator in 1..=max_denominator {
            let numerator = (value * denominator as f64).round();
            if numerator < 1.0 {
                continue;
            }
            let error = (numerator / denominator as f64 - value).abs();
            if best.is_none_or(|(known, _)| error < known - 1e-12) {
                best = Some((
                    error,
                    FrameRate {
                        numerator: numerator as u64,
                        denominator,
                    },
                ));
            }
        }
        best.map(|(_, rate)| rate)
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// The format of one video file. Each field is `None` when the file does not say.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VideoFormat {
    pub container: Option<Container>,
    pub codec: Option<Codec>,
    /// Picture size in pixels, after any cropping the codec states.
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Display aspect ratio, width over height. Taken from the container when it states one, else
    /// from the codec's pixel aspect ratio, else from the picture size with square pixels.
    pub display_aspect: Option<f64>,
    pub bit_depth: Option<u8>,
    pub chroma: Option<Chroma>,
    pub frame_rate: Option<FrameRate>,
    /// True when frames do not follow one fixed duration. `frame_rate` is then the average.
    pub variable_frame_rate: bool,
}

/// Read the format of the video in `path`. Files that are not video, or that use a format this
/// module does not read, give a [`VideoFormat`] with nothing set. Errors are only for files that
/// cannot be read.
pub fn inspect(path: &Path) -> Result<VideoFormat> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 8];
    let read = file.read(&mut magic)?;
    if read == 8
        && matches!(
            &magic[4..8],
            b"ftyp" | b"moov" | b"mdat" | b"free" | b"wide"
        )
    {
        return Ok(mp4::inspect(&mut file)?);
    }
    file.seek(SeekFrom::Start(0))?;
    let mut data = Vec::new();
    (&mut file).take(HEAD_LIMIT).read_to_end(&mut data)?;
    if data.starts_with(&EBML_MAGIC) {
        return Ok(matroska::inspect(&data));
    }
    Ok(raw::inspect(&data))
}

/// What a codec's own headers state. Each field is `None` when the headers do not say.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Headers {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bit_depth: Option<u8>,
    pub chroma: Option<Chroma>,
    /// Pixel aspect ratio, horizontal then vertical.
    pub sar: Option<(u32, u32)>,
    /// Time scale and ticks per frame: the frame rate is `scale / ticks`.
    pub timing: Option<(u64, u64)>,
}

/// The pixel aspect ratios by `aspect_ratio_idc` 1 to 16 (H.264 and HEVC Table E-1).
pub(crate) const ASPECT_RATIOS: [(u32, u32); 16] = [
    (1, 1),
    (12, 11),
    (10, 11),
    (16, 11),
    (40, 33),
    (24, 11),
    (20, 11),
    (32, 11),
    (80, 33),
    (18, 11),
    (15, 11),
    (64, 33),
    (160, 99),
    (4, 3),
    (3, 2),
    (2, 1),
];

/// What a container reader found, before the codec headers are read.
#[derive(Debug, Default)]
pub(crate) struct Found {
    pub container: Option<Container>,
    pub codec: Option<Codec>,
    /// The codec headers, already read.
    pub headers: Headers,
    /// The picture size as the container states it.
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub display_aspect: Option<f64>,
    /// The frame rate the container states or measures, and whether frames vary.
    pub frame_rate: Option<(FrameRate, bool)>,
    pub bit_depth: Option<u8>,
    pub chroma: Option<Chroma>,
}

/// The headers of a codec from its configuration record: `avcC`, `hvcC`, `av1C` or `vpcC`.
pub(crate) fn codec_headers(codec: Codec, config: &[u8]) -> Headers {
    let parsed = match codec {
        Codec::H264 => h264::from_avcc(config),
        Codec::Hevc => hevc::from_hvcc(config),
        Codec::Av1 => av1::from_av1c(config),
        Codec::Vp9 => vp9::from_vpcc(config),
    };
    parsed.unwrap_or_default()
}

/// Combine what the container and the codec say into one format.
pub(crate) fn assemble(found: Found) -> VideoFormat {
    let headers = found.headers;
    let width = headers.width.or(found.width);
    let height = headers.height.or(found.height);
    // With no display ratio and no pixel aspect ratio, the pixels are taken to be square, which is
    // what players assume.
    let display_aspect = found.display_aspect.or_else(|| {
        let (sar_width, sar_height) = headers.sar.unwrap_or((1, 1));
        let w = f64::from(width?);
        let h = f64::from(height?);
        Some(w * f64::from(sar_width) / (h * f64::from(sar_height)))
    });
    let (frame_rate, variable_frame_rate) = match found.frame_rate {
        Some((rate, variable)) => (Some(rate), variable),
        None => (
            headers
                .timing
                .and_then(|(scale, ticks)| FrameRate::new(scale, ticks)),
            false,
        ),
    };
    VideoFormat {
        container: found.container,
        codec: found.codec,
        width,
        height,
        display_aspect,
        bit_depth: headers.bit_depth.or(found.bit_depth),
        chroma: headers.chroma.or(found.chroma),
        frame_rate,
        variable_frame_rate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_rates_reduce_and_approximate() {
        assert_eq!(
            FrameRate::new(48000, 2002),
            Some(FrameRate {
                numerator: 24000,
                denominator: 1001
            })
        );
        assert_eq!(FrameRate::new(0, 1), None);
        let measured = FrameRate::approximate(1e9 / 41_708_333.0, 1001).unwrap();
        assert_eq!(
            measured,
            FrameRate {
                numerator: 24000,
                denominator: 1001
            }
        );
        let whole = FrameRate::approximate(25.0, 1001).unwrap();
        assert_eq!((whole.numerator, whole.denominator), (25, 1));
    }
}
