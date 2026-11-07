//! HEVC headers: the sequence parameter set (SPS), which states the picture size, colour depth,
//! chroma subsampling, pixel aspect ratio and timing. It comes from an MP4 file's `hvcC` box or
//! directly from an Annex B stream.

use super::bits::{Bits, unescape};
use super::{ASPECT_RATIOS, Chroma, Headers};

/// The NAL unit type of a sequence parameter set.
const SPS: u8 = 33;

/// The headers of the sequence parameter set in an `hvcC` record. The record's own chroma and bit
/// depth fields are the fallback when the record has no SPS.
pub(crate) fn from_hvcc(record: &[u8]) -> Option<Headers> {
    // Byte 0 is the configuration version, byte 16 the chroma format and byte 17 the luma bit depth.
    if record.len() < 23 || record[0] != 1 {
        return None;
    }
    let fallback = Headers {
        bit_depth: Some(8 + (record[17] & 0x07)),
        chroma: chroma_from_idc(u64::from(record[16] & 0x03)),
        ..Headers::default()
    };
    // Byte 22 is the number of NAL unit arrays; each holds NAL units of one type.
    let mut at = 23;
    for _ in 0..record[22] {
        let kind = *record.get(at)? & 0x3f;
        let count = usize::from(u16::from_be_bytes([
            *record.get(at + 1)?,
            *record.get(at + 2)?,
        ]));
        at += 3;
        for _ in 0..count {
            let len = usize::from(u16::from_be_bytes([*record.get(at)?, *record.get(at + 1)?]));
            let Some(nal) = record.get(at + 2..at + 2 + len) else {
                break;
            };
            at += 2 + len;
            if kind == SPS
                && let Some(headers) = from_sps(nal)
            {
                return Some(headers);
            }
        }
    }
    Some(fallback)
}

/// The headers of one SPS NAL unit, its two-byte header included.
pub(crate) fn from_sps(nal: &[u8]) -> Option<Headers> {
    let rbsp = unescape(nal.get(2..)?);
    let mut bits = Bits::new(&rbsp);
    // sps_video_parameter_set_id, sps_max_sub_layers_minus1, sps_temporal_id_nesting_flag.
    bits.skip(4)?;
    let max_sub_layers_minus1 = bits.bits(3)? as u32;
    bits.skip(1)?;
    profile_tier_level(&mut bits, max_sub_layers_minus1)?;
    // sps_seq_parameter_set_id.
    bits.ue()?;

    let chroma_format = bits.ue()?;
    let separate_planes = chroma_format == 3 && bits.flag()?;
    let mut width = bits.ue()?;
    let mut height = bits.ue()?;
    let (sub_width, sub_height) = if separate_planes {
        (1, 1)
    } else {
        match chroma_format {
            1 => (2, 2),
            2 => (2, 1),
            _ => (1, 1),
        }
    };
    if bits.flag()? {
        // The conformance window: offsets in chroma sample units.
        let left = bits.ue()?;
        let right = bits.ue()?;
        let top = bits.ue()?;
        let bottom = bits.ue()?;
        width = width.checked_sub(sub_width * (left + right))?;
        height = height.checked_sub(sub_height * (top + bottom))?;
    }

    let bit_depth = bits.ue()? + 8;
    // bit_depth_chroma_minus8.
    bits.ue()?;
    let log2_max_poc_lsb = bits.ue()? + 4;

    let ordering_for_all = bits.flag()?;
    let first = if ordering_for_all {
        0
    } else {
        max_sub_layers_minus1
    };
    for _ in first..=max_sub_layers_minus1 {
        // max_dec_pic_buffering_minus1, max_num_reorder_pics, max_latency_increase_plus1.
        bits.ue()?;
        bits.ue()?;
        bits.ue()?;
    }

    // Block sizes and transform depths: six Exp-Golomb values.
    for _ in 0..6 {
        bits.ue()?;
    }

    if bits.flag()? && bits.flag()? {
        skip_scaling_list_data(&mut bits)?;
    }
    // amp_enabled_flag, sample_adaptive_offset_enabled_flag.
    bits.skip(2)?;
    if bits.flag()? {
        // pcm_sample_bit_depth_luma_minus1 and chroma (4 bits each), two block sizes, and
        // pcm_loop_filter_disabled_flag.
        bits.skip(8)?;
        bits.ue()?;
        bits.ue()?;
        bits.skip(1)?;
    }

    let sets = bits.ue()?;
    let mut delta_counts = Vec::new();
    for index in 0..sets.min(64) {
        short_term_ref_pic_set(&mut bits, index as usize, &mut delta_counts)?;
    }
    if bits.flag()? {
        let long_term = bits.ue()?;
        for _ in 0..long_term.min(32) {
            bits.skip(log2_max_poc_lsb as u32)?;
            bits.skip(1)?;
        }
    }
    // sps_temporal_mvp_enabled_flag, strong_intra_smoothing_enabled_flag.
    bits.skip(2)?;

    let chroma = chroma_from_idc(chroma_format)?;
    let mut headers = Headers {
        width: u32::try_from(width).ok(),
        height: u32::try_from(height).ok(),
        bit_depth: u8::try_from(bit_depth).ok(),
        chroma: Some(chroma),
        ..Headers::default()
    };
    if bits.flag()? {
        read_vui(&mut bits, &mut headers);
    }
    Some(headers)
}

/// The chroma format for `chroma_format_idc`.
fn chroma_from_idc(idc: u64) -> Option<Chroma> {
    Some(match idc {
        0 => Chroma::Monochrome,
        1 => Chroma::Yuv420,
        2 => Chroma::Yuv422,
        3 => Chroma::Yuv444,
        _ => return None,
    })
}

/// Skip `profile_tier_level(1, max_sub_layers_minus1)` (section 7.3.3).
fn profile_tier_level(bits: &mut Bits, max_sub_layers_minus1: u32) -> Option<()> {
    // General profile (88 bits) and general level (8 bits).
    bits.skip(96)?;
    let mut profile_present = Vec::new();
    let mut level_present = Vec::new();
    for _ in 0..max_sub_layers_minus1 {
        profile_present.push(bits.flag()?);
        level_present.push(bits.flag()?);
    }
    if max_sub_layers_minus1 > 0 {
        for _ in max_sub_layers_minus1..8 {
            bits.skip(2)?;
        }
    }
    for (profile, level) in profile_present.iter().zip(&level_present) {
        if *profile {
            bits.skip(88)?;
        }
        if *level {
            bits.skip(8)?;
        }
    }
    Some(())
}

/// Read one `st_ref_pic_set()` (section 7.3.7) and record how many pictures it names. Only the
/// count is needed to read the rest of the SPS.
fn short_term_ref_pic_set(bits: &mut Bits, index: usize, counts: &mut Vec<u64>) -> Option<()> {
    let predicted = index != 0 && bits.flag()?;
    let count = if predicted {
        // In an SPS the set is predicted from the one before it.
        bits.skip(1)?;
        bits.ue()?;
        let reference = *counts.get(index - 1)?;
        let mut total = 0;
        for _ in 0..=reference {
            // used_by_curr_pic_flag, else use_delta_flag: only read when the first is clear.
            if bits.flag()? || bits.flag()? {
                total += 1;
            }
        }
        total
    } else {
        let negative = bits.ue()?;
        let positive = bits.ue()?;
        for _ in 0..negative + positive {
            bits.ue()?;
            bits.skip(1)?;
        }
        negative + positive
    };
    counts.push(count);
    Some(())
}

/// Skip `scaling_list_data()` (section 7.3.4).
fn skip_scaling_list_data(bits: &mut Bits) -> Option<()> {
    for size_id in 0..4u32 {
        let step = if size_id == 3 { 3 } else { 1 };
        for _matrix in (0..6).step_by(step) {
            if bits.flag()? {
                // The coefficients, with a DC value for the larger sizes.
                let coefficients = 64.min(1u32 << (4 + size_id * 2));
                if size_id > 1 {
                    bits.se()?;
                }
                for _ in 0..coefficients {
                    bits.se()?;
                }
            } else {
                bits.ue()?;
            }
        }
    }
    Some(())
}

/// The VUI fields that give the pixel aspect ratio and the timing. The rest are not needed.
fn read_vui(bits: &mut Bits, headers: &mut Headers) {
    if bits.flag().unwrap_or(false) {
        match bits.bits(8).unwrap_or(0) {
            255 => {
                if let (Some(w), Some(h)) = (bits.bits(16), bits.bits(16)) {
                    headers.sar = Some((w as u32, h as u32));
                }
            }
            index @ 1..=16 => headers.sar = Some(ASPECT_RATIOS[index as usize - 1]),
            _ => {}
        }
    }
    // overscan_info_present_flag, then overscan_appropriate_flag.
    if bits.flag().unwrap_or(false) {
        let _ = bits.skip(1);
    }
    // video_signal_type_present_flag: video_format and full range flag, then the colour description.
    if bits.flag().unwrap_or(false) {
        let _ = bits.skip(4);
        if bits.flag().unwrap_or(false) {
            let _ = bits.skip(24);
        }
    }
    // chroma_loc_info_present_flag: two Exp-Golomb values.
    if bits.flag().unwrap_or(false) {
        let _ = bits.ue();
        let _ = bits.ue();
    }
    // neutral_chroma_indication_flag, field_seq_flag, frame_field_info_present_flag.
    let _ = bits.skip(3);
    // default_display_window_flag: four Exp-Golomb offsets.
    if bits.flag().unwrap_or(false) {
        for _ in 0..4 {
            let _ = bits.ue();
        }
    }
    // vui_timing_info_present_flag: the clock tick and time scale.
    if bits.flag().unwrap_or(false) {
        let units = bits.bits(32);
        let scale = bits.bits(32);
        if let (Some(units), Some(scale)) = (units, scale)
            && units > 0
        {
            headers.timing = Some((scale, units));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hvcc_without_an_sps_falls_back_to_its_own_fields() {
        // Version 1, chroma format 1 (4:2:0) in byte 16, luma bit depth minus 8 of 2 in byte 17,
        // and no arrays.
        let mut record = vec![0u8; 23];
        record[0] = 1;
        record[16] = 0xFC | 1;
        record[17] = 0xF8 | 2;
        let headers = from_hvcc(&record).expect("parses");
        assert_eq!(headers.bit_depth, Some(10));
        assert_eq!(headers.chroma, Some(Chroma::Yuv420));
        assert_eq!(headers.width, None);
    }
}
