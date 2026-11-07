//! H.264 headers: the sequence parameter set (SPS), which states the picture size, colour depth,
//! chroma subsampling, pixel aspect ratio and timing. It comes from an MP4 file's `avcC` box or
//! directly from an Annex B stream.

use super::bits::{Bits, unescape};
use super::{ASPECT_RATIOS, Chroma, Headers, read_video_signal};

/// Profiles whose SPS carries the chroma format and bit depth fields (H.264 section 7.3.2.1.1).
const HIGH_PROFILES: [u64; 13] = [100, 110, 122, 244, 44, 83, 86, 118, 128, 138, 139, 134, 135];

/// The headers of the first SPS in an `avcC` record.
pub(crate) fn from_avcc(record: &[u8]) -> Option<Headers> {
    // Byte 0 is the configuration version, byte 5 the number of SPS entries (low five bits).
    if record.len() < 7 || record[0] != 1 {
        return None;
    }
    let mut at = 6;
    let sps = (0..(record[5] & 0x1f))
        .filter_map(|_| {
            let len = usize::from(u16::from_be_bytes([*record.get(at)?, *record.get(at + 1)?]));
            let nal = record.get(at + 2..at + 2 + len)?;
            at += 2 + len;
            Some(nal)
        })
        .find(|nal| nal.first().is_some_and(|header| header & 0x1f == 7))?;
    from_sps(sps)
}

/// The headers of one SPS NAL unit, header byte included.
pub(crate) fn from_sps(nal: &[u8]) -> Option<Headers> {
    let rbsp = unescape(nal.get(1..)?);
    let mut bits = Bits::new(&rbsp);
    let profile = bits.bits(8)?;
    // Constraint flags, then level.
    bits.skip(16)?;
    // seq_parameter_set_id.
    bits.ue()?;

    let (chroma_format, separate_planes, bit_depth) = if HIGH_PROFILES.contains(&profile) {
        let chroma_format = bits.ue()?;
        let separate_planes = chroma_format == 3 && bits.flag()?;
        let luma = bits.ue()?;
        let _chroma_depth = bits.ue()?;
        if luma > 6 {
            return None;
        }
        // qpprime_y_zero_transform_bypass_flag.
        bits.skip(1)?;
        if bits.flag()? {
            // Scaling lists: 8 of them, or 12 for 4:4:4.
            let count = if chroma_format == 3 { 12 } else { 8 };
            for index in 0..count {
                if bits.flag()? {
                    skip_scaling_list(&mut bits, if index < 6 { 16 } else { 64 })?;
                }
            }
        }
        (chroma_format, separate_planes, 8 + luma as u8)
    } else {
        (1, false, 8)
    };

    // log2_max_frame_num_minus4.
    bits.ue()?;
    match bits.ue()? {
        // pic_order_cnt_type 0: log2_max_pic_order_cnt_lsb_minus4.
        0 => {
            bits.ue()?;
        }
        // pic_order_cnt_type 1: the offsets of the reference frame cycle.
        1 => {
            bits.skip(1)?;
            bits.se()?;
            bits.se()?;
            let cycle = bits.ue()?;
            for _ in 0..cycle {
                bits.se()?;
            }
        }
        _ => {}
    }
    // max_num_ref_frames, gaps_in_frame_num_value_allowed_flag.
    bits.ue()?;
    bits.skip(1)?;
    let width_mbs = bits.ue()? + 1;
    let height_map_units = bits.ue()? + 1;
    let frame_mbs_only = bits.flag()?;
    if !frame_mbs_only {
        bits.skip(1)?;
    }
    // direct_8x8_inference_flag.
    bits.skip(1)?;

    // Frame cropping offsets, in crop units.
    let (mut left, mut right, mut top, mut bottom) = (0u64, 0u64, 0u64, 0u64);
    if bits.flag()? {
        left = bits.ue()?;
        right = bits.ue()?;
        top = bits.ue()?;
        bottom = bits.ue()?;
    }

    let frame_height_mbs = (if frame_mbs_only { 1 } else { 2 }) * height_map_units;
    let (sub_width, sub_height) = match chroma_format {
        1 => (2, 2),
        2 => (2, 1),
        _ => (1, 1),
    };
    let array_type = if separate_planes { 0 } else { chroma_format };
    let (crop_x, crop_y) = if array_type == 0 {
        (1, if frame_mbs_only { 1 } else { 2 })
    } else {
        (sub_width, sub_height * if frame_mbs_only { 1 } else { 2 })
    };
    let width = (width_mbs * 16).checked_sub(crop_x * (left + right))?;
    let height = (frame_height_mbs * 16).checked_sub(crop_y * (top + bottom))?;

    let chroma = match chroma_format {
        0 => Chroma::Monochrome,
        1 => Chroma::Yuv420,
        2 => Chroma::Yuv422,
        _ => Chroma::Yuv444,
    };

    let mut headers = Headers {
        width: u32::try_from(width).ok(),
        height: u32::try_from(height).ok(),
        bit_depth: Some(bit_depth),
        chroma: Some(chroma),
        ..Headers::default()
    };
    if bits.flag()? {
        read_vui(&mut bits, &mut headers);
    }
    Some(headers)
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
        bits.skip(1);
    }
    read_video_signal(bits, &mut headers.colour);
    // chroma_loc_info_present_flag: two Exp-Golomb values.
    if bits.flag().unwrap_or(false) {
        bits.ue();
        bits.ue();
    }
    // timing_info_present_flag: the timing follows.
    if bits.flag().unwrap_or(false) {
        let units = bits.bits(32);
        let scale = bits.bits(32);
        if let (Some(units), Some(scale)) = (units, scale)
            && units > 0
        {
            // A frame is two fields, so it takes two clock ticks.
            headers.timing = Some((scale, 2 * units));
        }
    }
}

/// Skip one `scaling_list()` of `size` entries (section 7.3.2.1.1.1).
fn skip_scaling_list(bits: &mut Bits, size: u32) -> Option<()> {
    let mut last = 8i64;
    let mut next = 8i64;
    for _ in 0..size {
        if next != 0 {
            next = (last + bits.se()? + 256).rem_euclid(256);
        }
        if next != 0 {
            last = next;
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_baseline_sps_is_8_bit_4_2_0() {
        // profile 66, constraint flags 0, level 30; then seq_parameter_set_id 0 (1 bit) and the
        // remaining fields for a 16x16 picture: ue 0 for each of the fields up to the size.
        // The bits are: 1 (sps id 0), 1 (log2_max_frame_num 0), 1 (poc type 0), 1 (poc lsb 0),
        // 1 (max refs 0), 0 (gaps), 1 (width 1 MB), 1 (height 1 MB), 1 (frame_mbs_only),
        // 1 (direct 8x8), 0 (no crop), 0 (no vui).
        let nal = [0x67, 66, 0, 30, 0b1111_1011, 0b1100_0000];
        let headers = from_sps(&nal).expect("parses");
        assert_eq!(headers.width, Some(16));
        assert_eq!(headers.height, Some(16));
        assert_eq!(headers.bit_depth, Some(8));
        assert_eq!(headers.chroma, Some(Chroma::Yuv420));
    }
}
