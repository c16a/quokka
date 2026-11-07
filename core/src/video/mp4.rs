//! MP4 and MOV files: the first video track, read from the `moov` box.

use std::collections::BTreeMap;
use std::io::{Read, Seek};

use super::bmff::{be, boxes, child, find_top, full_box};
use super::{
    Codec, Colour, Container, Found, FrameRate, Matrix, VideoFormat, assemble, codec_headers,
    specified_code,
};

/// The format of the first video track in an MP4 or MOV file.
pub(crate) fn inspect<R: Read + Seek>(file: &mut R) -> std::io::Result<VideoFormat> {
    let mut found = Found {
        container: Some(Container::Mp4),
        ..Found::default()
    };
    if let Some(moov) = find_top(file, b"moov")?
        && let Some(track) = first_video(&moov)
    {
        found = Found {
            container: Some(Container::Mp4),
            ..track
        };
    }
    Ok(assemble(found))
}

/// The first track whose handler is video, as far as the file states it.
fn first_video(moov: &[u8]) -> Option<Found> {
    let trak = boxes(moov)
        .filter(|(kind, _)| kind == b"trak")
        .map(|(_, body)| body)
        .find(|trak| handler(trak) == Some(*b"vide"))?;
    let mdia = child(trak, b"mdia")?;
    let timescale = timescale(child(mdia, b"mdhd")?)?;
    let stbl = child(child(mdia, b"minf")?, b"stbl")?;
    let (kind, entry) = sample_entry(stbl)?;
    // A visual sample entry: 78 bytes of fixed fields, then the codec's own boxes.
    let codec = codec_of(&kind);
    let children = entry.get(78..).unwrap_or(&[]);

    let mut found = Found {
        codec,
        width: be::<2>(entry, 24).map(|value| value as u32),
        height: be::<2>(entry, 26).map(|value| value as u32),
        ..Found::default()
    };
    if let Some(codec) = codec {
        let config = match codec {
            Codec::H264 => b"avcC",
            Codec::Hevc => b"hvcC",
            Codec::Av1 => b"av1C",
            Codec::Vp9 => b"vpcC",
        };
        if let Some(record) = child(children, config) {
            found.headers = codec_headers(codec, record);
        }
    }
    found.display_aspect = display_aspect(trak);
    found.colour = colour_box(children);
    if found.headers.sar.is_none() {
        found.headers.sar = pixel_aspect(children);
    }
    found.frame_rate = frame_rate(moov, stbl, timescale, track_id(trak));
    Some(found)
}

/// The handler type of a track's media, such as `vide`.
pub(crate) fn handler(trak: &[u8]) -> Option<[u8; 4]> {
    let hdlr = child(child(trak, b"mdia")?, b"hdlr")?;
    let (_, rest) = full_box(hdlr)?;
    rest.get(4..8)?.try_into().ok()
}

/// The time scale of a media header: ticks per second.
pub(crate) fn timescale(mdhd: &[u8]) -> Option<u64> {
    let (version, rest) = full_box(mdhd)?;
    // Version 1 has 64-bit times, so the time scale comes after 16 more bytes.
    let at = if version == 1 { 16 } else { 8 };
    be::<4>(rest, at)
}

/// The first sample entry of a track: its type and body.
pub(crate) fn sample_entry(stbl: &[u8]) -> Option<([u8; 4], &[u8])> {
    let (_, rest) = full_box(child(stbl, b"stsd")?)?;
    boxes(rest.get(4..)?).next()
}

pub(crate) fn codec_of(kind: &[u8; 4]) -> Option<Codec> {
    match kind {
        b"avc1" | b"avc3" => Some(Codec::H264),
        b"hvc1" | b"hev1" => Some(Codec::Hevc),
        b"av01" => Some(Codec::Av1),
        b"vp09" => Some(Codec::Vp9),
        _ => None,
    }
}

/// The track's id, from its header.
fn track_id(trak: &[u8]) -> Option<u64> {
    let (version, rest) = full_box(child(trak, b"tkhd")?)?;
    be::<4>(rest, if version == 1 { 16 } else { 8 })
}

/// The display size from the track header, as a width over a height. The last two 16.16 fixed
/// point numbers of `tkhd` are the display width and height.
fn display_aspect(trak: &[u8]) -> Option<f64> {
    let tkhd = child(trak, b"tkhd")?;
    let end = tkhd.len().checked_sub(8)?;
    // Each is a 16.16 fixed point number, so the fraction is kept: 426.67 is not 426.
    let width = be::<4>(tkhd, end)? as f64 / 65536.0;
    let height = be::<4>(tkhd, end + 4)? as f64 / 65536.0;
    (width > 0.0 && height > 0.0).then(|| width / height)
}

/// The pixel aspect ratio from a `pasp` box, horizontal then vertical spacing.
fn pixel_aspect(children: &[u8]) -> Option<(u32, u32)> {
    let pasp = child(children, b"pasp")?;
    let horizontal = be::<4>(pasp, 0)?;
    let vertical = be::<4>(pasp, 4)?;
    (horizontal > 0 && vertical > 0).then_some((horizontal as u32, vertical as u32))
}

/// The frame rate, from the sample durations of the track. A fragmented file has none in the
/// track, so its default sample duration in `trex` is used.
fn frame_rate(
    moov: &[u8],
    stbl: &[u8],
    timescale: u64,
    track_id: Option<u64>,
) -> Option<(FrameRate, bool)> {
    if let Some(stts) = child(stbl, b"stts")
        && let Some(rate) = stts_rate(stts, timescale)
    {
        return Some(rate);
    }
    let mvex = child(moov, b"mvex")?;
    let duration = boxes(mvex)
        .filter(|(kind, _)| kind == b"trex")
        .map(|(_, body)| body)
        .find(|trex| full_box(trex).is_some_and(|(_, rest)| be::<4>(rest, 0) == track_id))
        .and_then(|trex| full_box(trex).and_then(|(_, rest)| be::<4>(rest, 8)))?;
    FrameRate::new(timescale, duration).map(|rate| (rate, false))
}

/// The frame rate from a `stts` table of (sample count, sample duration) pairs. The rate is the
/// most common duration. A track is variable when that duration covers less than 99% of its frames.
fn stts_rate(stts: &[u8], timescale: u64) -> Option<(FrameRate, bool)> {
    let (_, rest) = full_box(stts)?;
    let entries = be::<4>(rest, 0)? as usize;
    let mut frames = 0u64;
    let mut ticks = 0u64;
    let mut by_duration: BTreeMap<u64, u64> = BTreeMap::new();
    for index in 0..entries.min(rest.len() / 8) {
        let at = 4 + index * 8;
        let count = be::<4>(rest, at)?;
        let duration = be::<4>(rest, at + 4)?;
        frames = frames.saturating_add(count);
        ticks = ticks.saturating_add(count.saturating_mul(duration));
        *by_duration.entry(duration).or_default() += count;
    }
    if frames == 0 || ticks == 0 {
        return None;
    }
    let (&common, &covered) = by_duration.iter().max_by_key(|(_, count)| **count)?;
    let variable = covered * 100 < frames * 99;
    let rate = if variable {
        FrameRate::new(timescale.saturating_mul(frames), ticks)?
    } else {
        FrameRate::new(timescale, common)?
    };
    Some((rate, variable))
}

/// The colour a `colr` box states, when it is an `nclx` box: the matrix and the range.
fn colour_box(children: &[u8]) -> Colour {
    let Some(colr) = child(children, b"colr") else {
        return Colour::default();
    };
    if colr.get(0..4) != Some(b"nclx") || colr.len() < 11 {
        return Colour::default();
    }
    Colour {
        full_range: Some(colr[10] & 0x80 != 0),
        matrix: Matrix::from_code(u64::from(u16::from_be_bytes([colr[8], colr[9]]))),
        primaries: specified_code(u64::from(u16::from_be_bytes([colr[4], colr[5]]))),
        transfer: specified_code(u64::from(u16::from_be_bytes([colr[6], colr[7]]))),
    }
}
