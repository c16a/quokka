//! Converting a stream's packets into the Annex B byte stream that Media Foundation reads. A container
//! stores H.264 and HEVC as length-prefixed NAL units, with the parameter sets in a configuration
//! record (`avcC` or `hvcC`). Annex B instead marks each NAL unit with a start code, and the decoder
//! takes the parameter sets from the sequence header. This file is plain byte handling, so it builds
//! and is tested on every platform.

use quokka_core::video::Codec;

/// The start code that precedes each NAL unit in an Annex B stream.
const START_CODE: [u8; 4] = [0, 0, 0, 1];

/// What the packets of a stream need: how long each length prefix is, and the parameter sets as
/// Annex B NAL units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NalFormat {
    /// The length prefix in bytes, from 1 to 4.
    pub length_size: usize,
    /// The sequence and picture parameter sets, each behind a start code.
    pub parameter_sets: Vec<u8>,
}

/// Read a configuration record. `None` when the record is too short or malformed, or the codec is
/// not H.264 or HEVC.
pub(crate) fn parse_record(codec: Codec, record: &[u8]) -> Option<NalFormat> {
    match codec {
        Codec::H264 => avc(record),
        Codec::Hevc => hevc(record),
        _ => None,
    }
}

/// An `avcC` record: a fixed head, then the SPS and PPS lists.
fn avc(record: &[u8]) -> Option<NalFormat> {
    let length_size = usize::from(*record.get(4)? & 0x03) + 1;
    let sps_count = usize::from(*record.get(5)? & 0x1F);
    let mut at = 6;
    let mut parameter_sets = Vec::new();
    for _ in 0..sps_count {
        at = push_nal(record, at, &mut parameter_sets)?;
    }
    let pps_count = usize::from(*record.get(at)?);
    at += 1;
    for _ in 0..pps_count {
        at = push_nal(record, at, &mut parameter_sets)?;
    }
    Some(NalFormat {
        length_size,
        parameter_sets,
    })
}

/// An `hvcC` record: 22 bytes of fixed fields, then arrays of NAL units tagged by type. Only the
/// VPS, SPS and PPS arrays are kept, since they are the parameter sets.
fn hevc(record: &[u8]) -> Option<NalFormat> {
    const VPS: u8 = 32;
    const SPS: u8 = 33;
    const PPS: u8 = 34;

    let length_size = usize::from(*record.get(21)? & 0x03) + 1;
    let array_count = usize::from(*record.get(22)?);
    let mut at = 23;
    let mut parameter_sets = Vec::new();
    for _ in 0..array_count {
        // The first byte holds the completeness flag and the NAL unit type.
        let nal_type = *record.get(at)? & 0x3F;
        let count = usize::from(u16::from_be_bytes([
            *record.get(at + 1)?,
            *record.get(at + 2)?,
        ]));
        at += 3;
        for _ in 0..count {
            if matches!(nal_type, VPS | SPS | PPS) {
                at = push_nal(record, at, &mut parameter_sets)?;
            } else {
                at = skip_nal(record, at)?;
            }
        }
    }
    Some(NalFormat {
        length_size,
        parameter_sets,
    })
}

/// Append the NAL unit that starts at `at` (a 16-bit length, then its bytes) with a start code.
/// Returns the offset after it.
fn push_nal(record: &[u8], at: usize, out: &mut Vec<u8>) -> Option<usize> {
    let end = nal_end(record, at)?;
    out.extend_from_slice(&START_CODE);
    out.extend_from_slice(&record[at + 2..end]);
    Some(end)
}

/// The offset after the NAL unit that starts at `at`, without copying it.
fn skip_nal(record: &[u8], at: usize) -> Option<usize> {
    nal_end(record, at)
}

/// Where the NAL unit that starts at `at` ends. `None` when its length runs past the record.
fn nal_end(record: &[u8], at: usize) -> Option<usize> {
    let length = usize::from(u16::from_be_bytes([*record.get(at)?, *record.get(at + 1)?]));
    let end = (at + 2).checked_add(length)?;
    (end <= record.len()).then_some(end)
}

/// Rewrite one packet's length-prefixed NAL units as an Annex B byte stream. `None` when a length
/// runs past the end of the packet, or the prefix is not 1 to 4 bytes.
pub(crate) fn sample(data: &[u8], length_size: usize) -> Option<Vec<u8>> {
    if !(1..=4).contains(&length_size) {
        return None;
    }
    let mut out = Vec::with_capacity(data.len() + 16);
    let mut at = 0;
    while at < data.len() {
        let prefix = data.get(at..at + length_size)?;
        let length = prefix
            .iter()
            .fold(0usize, |acc, &byte| (acc << 8) | usize::from(byte));
        at += length_size;
        let end = at.checked_add(length)?;
        let nal = data.get(at..end)?;
        out.extend_from_slice(&START_CODE);
        out.extend_from_slice(nal);
        at = end;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An `avcC` record with one SPS and one PPS, and four-byte length prefixes.
    fn avc_record() -> Vec<u8> {
        let mut record = vec![1, 0x64, 0, 0x1F, 0xFF, 0xE1];
        record.extend_from_slice(&[0, 3, 0x67, 0xAA, 0xBB]); // SPS
        record.push(1);
        record.extend_from_slice(&[0, 2, 0x68, 0xCC]); // PPS
        record
    }

    #[test]
    fn avc_record_gives_start_coded_parameter_sets() {
        let format = parse_record(Codec::H264, &avc_record()).expect("a valid record");
        assert_eq!(format.length_size, 4);
        assert_eq!(
            format.parameter_sets,
            [0, 0, 0, 1, 0x67, 0xAA, 0xBB, 0, 0, 0, 1, 0x68, 0xCC]
        );
    }

    #[test]
    fn hevc_record_keeps_only_parameter_set_arrays() {
        let mut record = vec![0u8; 21];
        record.push(0xFD); // lengthSizeMinusOne = 1, so two-byte prefixes
        record.push(2); // two arrays
        // VPS array with one NAL unit.
        record.extend_from_slice(&[0x20, 0, 1, 0, 2, 0x40, 0x01]);
        // SEI array (type 39), which is skipped.
        record.extend_from_slice(&[0x27, 0, 1, 0, 1, 0x4E]);
        let format = parse_record(Codec::Hevc, &record).expect("a valid record");
        assert_eq!(format.length_size, 2);
        assert_eq!(format.parameter_sets, [0, 0, 0, 1, 0x40, 0x01]);
    }

    #[test]
    fn truncated_record_is_refused() {
        let record = avc_record();
        assert!(parse_record(Codec::H264, &record[..record.len() - 1]).is_none());
        assert!(parse_record(Codec::H264, &[]).is_none());
        assert!(parse_record(Codec::Av1, &avc_record()).is_none());
    }

    #[test]
    fn packet_lengths_become_start_codes() {
        let packet = [0, 0, 0, 2, 0x65, 0x88, 0, 0, 0, 1, 0x41];
        let out = sample(&packet, 4).expect("a valid packet");
        assert_eq!(out, [0, 0, 0, 1, 0x65, 0x88, 0, 0, 0, 1, 0x41]);
    }

    #[test]
    fn packet_with_a_length_past_its_end_is_refused() {
        assert!(sample(&[0, 0, 0, 9, 0x65], 4).is_none());
        assert!(sample(&[0, 0, 0], 4).is_none());
        assert!(sample(&[0, 1, 0x65], 0).is_none());
    }
}
