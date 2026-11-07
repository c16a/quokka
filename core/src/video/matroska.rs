//! Matroska and WebM files: the first video track, read from the `Tracks` element. The frame rate
//! comes from the track's default frame duration, or is measured from the block timestamps when
//! the track does not state one.

use std::io::{self, Read, Seek, SeekFrom};

use super::ebml::{elements, text, uint, vint};
use super::samples::{Sample, SampleTrack};
use super::{
    Chroma, Codec, Container, Found, FrameRate, Headers, VideoFormat, assemble, codec_headers, vp9,
};

const SEGMENT: u64 = 0x1853_8067;
const INFO: u64 = 0x1549_A966;
const TIMESTAMP_SCALE: u64 = 0x2A_D7B1;
const CLUSTER: u64 = 0x1F43_B675;
const TIMESTAMP: u64 = 0xE7;
const SIMPLE_BLOCK: u64 = 0xA3;
const BLOCK_GROUP: u64 = 0xA0;
const BLOCK: u64 = 0xA1;
const TRACKS: u64 = 0x1654_AE6B;
const TRACK_ENTRY: u64 = 0xAE;
const TRACK_NUMBER: u64 = 0xD7;
const TRACK_TYPE: u64 = 0x83;
const CODEC_ID: u64 = 0x86;
const CODEC_PRIVATE: u64 = 0x63A2;
const DEFAULT_DURATION: u64 = 0x23_E383;
const VIDEO: u64 = 0xE0;
const PIXEL_WIDTH: u64 = 0xB0;
const PIXEL_HEIGHT: u64 = 0xBA;
const DISPLAY_WIDTH: u64 = 0x54B0;
const DISPLAY_HEIGHT: u64 = 0x54BA;
const COLOUR: u64 = 0x55B0;
const BITS_PER_CHANNEL: u64 = 0x55B2;
const CHROMA_SUBSAMPLING_HORZ: u64 = 0x55B3;
const CHROMA_SUBSAMPLING_VERT: u64 = 0x55B4;

/// The TrackType of a video track.
const VIDEO_TRACK: u64 = 1;

/// The default timestamp scale: each timestamp tick is a millisecond.
const DEFAULT_TIMESTAMP_SCALE: u64 = 1_000_000;

/// The most blocks read to measure a frame rate that the track does not state.
const MAX_BLOCKS: usize = 240;

/// The fields of one track that the format needs.
#[derive(Default)]
struct Track {
    number: u64,
    kind: u64,
    codec_id: String,
    private: Vec<u8>,
    default_duration: Option<u64>,
    pixel_width: Option<u64>,
    pixel_height: Option<u64>,
    display_width: Option<u64>,
    display_height: Option<u64>,
    bits_per_channel: Option<u64>,
    chroma_horizontal: Option<u64>,
    chroma_vertical: Option<u64>,
}

/// The format of the first video track in a Matroska or WebM file, given its first bytes.
pub(crate) fn inspect(data: &[u8]) -> VideoFormat {
    let found = found(data).unwrap_or(Found {
        container: Some(Container::Matroska),
        ..Found::default()
    });
    assemble(found)
}

fn found(data: &[u8]) -> Option<Found> {
    let segment = elements(data).find(|(id, _)| *id == SEGMENT)?.1;
    let timestamp_scale = timestamp_scale(segment);
    let tracks = elements(segment).find(|(id, _)| *id == TRACKS)?.1;
    let track = elements(tracks)
        .filter(|(id, _)| *id == TRACK_ENTRY)
        .map(|(_, body)| parse_track(body))
        .find(|track| track.kind == VIDEO_TRACK)?;

    let codec = codec_of(&track.codec_id);
    let headers = match codec {
        Some(codec) if !track.private.is_empty() => codec_headers(codec, &track.private),
        Some(Codec::Vp9) => first_frame(segment, track.number)
            .and_then(vp9::from_keyframe)
            .unwrap_or_default(),
        _ => Headers::default(),
    };
    let frame_rate = track
        .default_duration
        .filter(|&nanoseconds| nanoseconds > 0)
        .and_then(|nanoseconds| FrameRate::approximate(1e9 / nanoseconds as f64, 1001))
        .map(|rate| (rate, false))
        .or_else(|| block_rate(segment, track.number, timestamp_scale));

    let display_aspect = match (track.display_width, track.display_height) {
        (Some(width), Some(height)) if width > 0 && height > 0 => {
            Some(width as f64 / height as f64)
        }
        _ => None,
    };
    Some(Found {
        container: Some(Container::Matroska),
        codec,
        headers,
        width: track.pixel_width.map(|value| value as u32),
        height: track.pixel_height.map(|value| value as u32),
        display_aspect,
        frame_rate,
        bit_depth: track
            .bits_per_channel
            .and_then(|bits| u8::try_from(bits).ok()),
        chroma: chroma_from_colour(track.chroma_horizontal, track.chroma_vertical),
    })
}

/// The time of one timestamp tick, in nanoseconds.
fn timestamp_scale(segment: &[u8]) -> u64 {
    elements(segment)
        .find(|(id, _)| *id == INFO)
        .and_then(|(_, info)| {
            elements(info)
                .find(|(id, _)| *id == TIMESTAMP_SCALE)
                .map(|(_, body)| uint(body))
        })
        .filter(|&scale| scale > 0)
        .unwrap_or(DEFAULT_TIMESTAMP_SCALE)
}

fn parse_track(body: &[u8]) -> Track {
    let mut track = Track::default();
    for (id, value) in elements(body) {
        match id {
            TRACK_NUMBER => track.number = uint(value),
            TRACK_TYPE => track.kind = uint(value),
            CODEC_ID => track.codec_id = text(value),
            CODEC_PRIVATE => track.private = value.to_vec(),
            DEFAULT_DURATION => track.default_duration = Some(uint(value)),
            VIDEO => parse_video(value, &mut track),
            _ => {}
        }
    }
    track
}

fn parse_video(body: &[u8], track: &mut Track) {
    for (id, value) in elements(body) {
        match id {
            PIXEL_WIDTH => track.pixel_width = Some(uint(value)),
            PIXEL_HEIGHT => track.pixel_height = Some(uint(value)),
            DISPLAY_WIDTH => track.display_width = Some(uint(value)),
            DISPLAY_HEIGHT => track.display_height = Some(uint(value)),
            COLOUR => {
                for (colour_id, colour_value) in elements(value) {
                    match colour_id {
                        BITS_PER_CHANNEL => track.bits_per_channel = Some(uint(colour_value)),
                        CHROMA_SUBSAMPLING_HORZ => {
                            track.chroma_horizontal = Some(uint(colour_value));
                        }
                        CHROMA_SUBSAMPLING_VERT => {
                            track.chroma_vertical = Some(uint(colour_value));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn codec_of(codec_id: &str) -> Option<Codec> {
    match codec_id {
        "V_AV1" => Some(Codec::Av1),
        "V_MPEG4/ISO/AVC" => Some(Codec::H264),
        "V_MPEGH/ISO/HEVC" => Some(Codec::Hevc),
        "V_VP9" => Some(Codec::Vp9),
        _ => None,
    }
}

/// The chroma subsampling from the Colour element: the pixels removed horizontally and vertically
/// to get the chroma samples.
fn chroma_from_colour(horizontal: Option<u64>, vertical: Option<u64>) -> Option<Chroma> {
    match (horizontal?, vertical?) {
        (1, 1) => Some(Chroma::Yuv420),
        (1, 0) => Some(Chroma::Yuv422),
        (0, 0) => Some(Chroma::Yuv444),
        _ => None,
    }
}

/// The frame rate measured from the timestamps of the first blocks of the track. The timestamps
/// are in whole ticks, so this is the average over the blocks, not a per-frame rate.
fn block_rate(segment: &[u8], track: u64, timestamp_scale: u64) -> Option<(FrameRate, bool)> {
    let mut times: Vec<u64> = Vec::new();
    'clusters: for (_, cluster) in elements(segment).filter(|(id, _)| *id == CLUSTER) {
        let mut base = 0u64;
        for (id, value) in elements(cluster) {
            match id {
                TIMESTAMP => base = uint(value),
                SIMPLE_BLOCK => push_block(value, base, track, timestamp_scale, &mut times),
                BLOCK_GROUP => {
                    for (block_id, block) in elements(value) {
                        if block_id == BLOCK {
                            push_block(block, base, track, timestamp_scale, &mut times);
                        }
                    }
                }
                _ => {}
            }
            if times.len() >= MAX_BLOCKS {
                break 'clusters;
            }
        }
    }
    if times.len() < 3 {
        return None;
    }
    times.sort_unstable();
    let span = times.last()? - times.first()?;
    let intervals = (times.len() - 1) as f64;
    let average = intervals * 1e9 / span.max(1) as f64;
    FrameRate::approximate(average, 1001).map(|rate| (rate, false))
}

/// The payload of the first block of `track`, which for VP9 is a keyframe.
fn first_frame(segment: &[u8], track: u64) -> Option<&[u8]> {
    for (_, cluster) in elements(segment).filter(|(id, _)| *id == CLUSTER) {
        for (id, value) in elements(cluster) {
            let block = match id {
                SIMPLE_BLOCK => value,
                BLOCK_GROUP => match elements(value).find(|(inner, _)| *inner == BLOCK) {
                    Some((_, block)) => block,
                    None => continue,
                },
                _ => continue,
            };
            let Some((number, used)) = vint(block, false) else {
                continue;
            };
            // After the track number come a 16-bit timecode and a flags byte.
            if number == track {
                return block.get(used + 3..);
            }
        }
    }
    None
}

/// Add the time of a block that belongs to `track`. A block starts with its track number as a
/// variable-length integer, then a 16-bit timecode relative to its cluster.
fn push_block(block: &[u8], base: u64, track: u64, timestamp_scale: u64, times: &mut Vec<u64>) {
    let Some((number, used)) = vint(block, false) else {
        return;
    };
    if number != track {
        return;
    }
    let Some(bytes) = block.get(used..used + 2) else {
        return;
    };
    let timecode = i64::from(i16::from_be_bytes([bytes[0], bytes[1]]));
    let ticks = (base as i64 + timecode).max(0) as u64;
    times.push(ticks.saturating_mul(timestamp_scale));
}

/// One element header: its id, the length of the header, and the body's size when it is known.
struct Header {
    id: u64,
    length: u64,
    size: Option<u64>,
}

/// The samples of the video track of a Matroska or WebM file. The file is read in one pass over
/// its element headers. Clusters and block groups are entered rather than skipped, so only the
/// headers are read, plus the small `Info` and `Tracks` elements. Each block is one sample, and its
/// bytes stay where they are in the file.
///
/// Blocks carry only presentation times, so the decode time is set to the presentation time. That
/// is correct for streams without reordered frames, which covers the VP9 and AV1 fixtures.
pub(crate) fn read_samples<R: Read + Seek>(file: &mut R) -> io::Result<Option<SampleTrack>> {
    let end = file.seek(SeekFrom::End(0))?;
    let mut scale = DEFAULT_TIMESTAMP_SCALE;
    let mut video: Option<Track> = None;
    let mut cluster_time = 0i64;
    let mut samples = Vec::new();
    let mut position = 0u64;
    while position < end {
        let Some(header) = element_header(file, position, end)? else {
            break;
        };
        let body = position + header.length;
        let body_end = header
            .size
            .map_or(end, |size| body.saturating_add(size).min(end));
        match header.id {
            // Masters: their children follow directly, so enter them.
            SEGMENT | CLUSTER | BLOCK_GROUP => {
                position = body;
                continue;
            }
            INFO => {
                for (id, value) in elements(&read_span(file, body, body_end)?) {
                    if id == TIMESTAMP_SCALE {
                        scale = uint(value).max(1);
                    }
                }
            }
            TRACKS => {
                let tracks = read_span(file, body, body_end)?;
                video = elements(&tracks)
                    .filter(|(id, _)| *id == TRACK_ENTRY)
                    .map(|(_, entry)| parse_track(entry))
                    .find(|track| track.kind == VIDEO_TRACK);
            }
            TIMESTAMP => {
                cluster_time = uint(&read_span(file, body, body_end)?) as i64;
            }
            SIMPLE_BLOCK | BLOCK => {
                if let Some(track) = &video {
                    let head = read_span(file, body, (body + 12).min(body_end))?;
                    if let Some((number, used)) = vint(&head, false)
                        && number == track.number
                        && head.len() >= used + 3
                    {
                        if head[used + 2] & 0x06 != 0 {
                            return Err(io::Error::other("laced blocks are not supported"));
                        }
                        let timecode = i64::from(i16::from_be_bytes([head[used], head[used + 1]]));
                        // A keyframe flag only exists on simple blocks, so block-group frames are
                        // taken as sync frames.
                        let sync = header.id == BLOCK || head[used + 2] & 0x80 != 0;
                        let time = cluster_time + timecode;
                        samples.push(Sample {
                            offset: body + used as u64 + 3,
                            size: (body_end - (body + used as u64 + 3)) as u32,
                            decode_time: time,
                            presentation_time: time,
                            sync,
                        });
                    }
                }
            }
            _ => {}
        }
        position = body_end;
    }

    let Some(track) = video else {
        return Ok(None);
    };
    let (Some(codec), Some(width), Some(height)) = (
        codec_of(&track.codec_id),
        track.pixel_width,
        track.pixel_height,
    ) else {
        return Ok(None);
    };
    let timescale = 1_000_000_000u64 / scale;
    Ok(Some(SampleTrack {
        codec,
        width: width as u32,
        height: height as u32,
        config: track.private,
        timescale,
        samples,
    }))
}

/// The header of the element at `position`, or `None` where the bytes do not parse.
fn element_header<R: Read + Seek>(
    file: &mut R,
    position: u64,
    end: u64,
) -> io::Result<Option<Header>> {
    let bytes = read_span(file, position, (position + 12).min(end))?;
    let Some((id, id_len)) = vint(&bytes, true) else {
        return Ok(None);
    };
    let Some((size, size_len)) = vint(bytes.get(id_len..).unwrap_or(&[]), false) else {
        return Ok(None);
    };
    // An all-ones size means the size is unknown.
    let unknown = size == (1u64 << (7 * size_len)) - 1;
    Ok(Some(Header {
        id,
        length: (id_len + size_len) as u64,
        size: (!unknown).then_some(size),
    }))
}

/// The bytes from `start` to `end`. Only small elements are read this way.
fn read_span<R: Read + Seek>(file: &mut R, start: u64, end: u64) -> io::Result<Vec<u8>> {
    /// The largest element read into memory this way.
    const MAX_SPAN: u64 = 16 * 1024 * 1024;
    let length = end.saturating_sub(start);
    if length > MAX_SPAN {
        return Err(io::Error::other(
            "a Matroska element is too large to read into memory",
        ));
    }
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = vec![0u8; length as usize];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}
