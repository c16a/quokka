//! Raw video streams, which have no container: AV1 OBUs (`.obu`) and H.264 or HEVC Annex B
//! streams, whose NAL units are split by start codes (`.h264`, `.hevc`).

use super::{Codec, Container, Found, VideoFormat, assemble, av1, h264, hevc};

/// How many NAL units are looked at for the parameter set. The set comes first in a stream.
const MAX_NAL_UNITS: usize = 64;

/// The format of a raw stream, given its first bytes.
pub(crate) fn inspect(data: &[u8]) -> VideoFormat {
    annexb(data)
        .or_else(|| obu(data))
        .map(assemble)
        .unwrap_or_default()
}

/// An Annex B stream: a start code, then NAL units. The first NAL unit's header tells H.264 from
/// HEVC, because the two headers cannot be confused for each other.
fn annexb(data: &[u8]) -> Option<Found> {
    if !(data.starts_with(&[0, 0, 1]) || data.starts_with(&[0, 0, 0, 1])) {
        return None;
    }
    let nals = nal_units(data);
    let first = *nals.first()?.first()?;
    let codec = if is_h264_first(first) {
        Codec::H264
    } else if is_hevc_first(first, nals.first()?.get(1).copied()) {
        Codec::Hevc
    } else {
        return None;
    };
    let headers = nals
        .iter()
        .find_map(|nal| match codec {
            Codec::H264 if nal[0] & 0x1f == 7 => h264::from_sps(nal),
            Codec::Hevc if (nal[0] >> 1) & 0x3f == 33 => hevc::from_sps(nal),
            _ => None,
        })
        .unwrap_or_default();
    Some(Found {
        container: Some(Container::AnnexB),
        codec: Some(codec),
        headers,
        ..Found::default()
    })
}

/// H.264 NAL unit types 6 to 9 (SEI, SPS, PPS and access unit delimiter), with the forbidden bit
/// clear. These are the first NAL units a stream starts with.
fn is_h264_first(header: u8) -> bool {
    header & 0x80 == 0 && matches!(header & 0x1f, 6..=9)
}

/// HEVC NAL unit types 32 to 35 and 39 and 40 (VPS, SPS, PPS, delimiter and SEI), with the
/// forbidden bit clear and a temporal layer id.
fn is_hevc_first(header: u8, second: Option<u8>) -> bool {
    header & 0x80 == 0
        && matches!((header >> 1) & 0x3f, 32..=35 | 39 | 40)
        && second.is_some_and(|byte| byte & 0x07 != 0)
}

/// The NAL units of an Annex B stream, without their start codes and trailing zero bytes.
fn nal_units(data: &[u8]) -> Vec<&[u8]> {
    let mut starts = Vec::new();
    let mut at = 0;
    while at + 3 <= data.len() && starts.len() < MAX_NAL_UNITS {
        if data[at] == 0 && data[at + 1] == 0 && data[at + 2] == 1 {
            starts.push(at + 3);
            at += 3;
        } else {
            at += 1;
        }
    }
    starts
        .iter()
        .enumerate()
        .map(|(index, &start)| {
            let mut end = starts.get(index + 1).map_or(data.len(), |&next| next - 3);
            while end > start && data[end - 1] == 0 {
                end -= 1;
            }
            &data[start..end]
        })
        .filter(|nal| !nal.is_empty())
        .collect()
}

/// A raw AV1 stream of OBUs. It starts with a temporal delimiter or a sequence header, and every
/// OBU has a size.
fn obu(data: &[u8]) -> Option<Found> {
    let first = *data.first()?;
    let kind = (first >> 3) & 0x0f;
    let has_size = first & 0x02 != 0;
    if first & 0x80 != 0 || !has_size || !(kind == 1 || kind == 2) {
        return None;
    }
    let headers = av1::from_obu_stream(data)?;
    Some(Found {
        container: Some(Container::RawObu),
        codec: Some(Codec::Av1),
        headers,
        ..Found::default()
    })
}
