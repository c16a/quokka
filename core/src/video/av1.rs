//! AV1 headers: the sequence header OBU, which states the picture size, colour depth, chroma
//! subsampling and, when the encoder wrote it, the timing. It comes from a raw OBU stream or from
//! the sequence header inside an `av1C` record.

use super::bits::Bits;
use super::{Chroma, Colour, Headers, Matrix};

/// The OBU type of a sequence header.
const SEQUENCE_HEADER: u8 = 1;

/// The headers of an `av1C` record: its four-byte header, then the configuration OBUs, which
/// hold a sequence header. The record's flags give the colour fields when no header is found.
pub(crate) fn from_av1c(record: &[u8]) -> Option<Headers> {
    // Byte 0: the marker bit and version. Byte 2: the colour flags.
    if record.len() < 4 || record[0] & 0x80 == 0 {
        return None;
    }
    let flags = record[2];
    let fallback = Headers {
        bit_depth: Some(if flags & 0x40 == 0 {
            8
        } else if flags & 0x20 != 0 {
            12
        } else {
            10
        }),
        chroma: Some(if flags & 0x10 != 0 {
            Chroma::Monochrome
        } else {
            match (((flags >> 3) & 1), ((flags >> 2) & 1)) {
                (1, 1) => Chroma::Yuv420,
                (1, 0) => Chroma::Yuv422,
                _ => Chroma::Yuv444,
            }
        }),
        ..Headers::default()
    };
    Some(from_obus(&record[4..]).unwrap_or(fallback))
}

/// The sequence header of a low-overhead OBU stream (section 5.2), in which every OBU carries a
/// size.
pub(crate) fn from_obu_stream(data: &[u8]) -> Option<Headers> {
    from_obus(data)
}

/// The sequence header in a run of OBUs, if one comes before the end.
fn from_obus(data: &[u8]) -> Option<Headers> {
    let mut at = 0;
    while at < data.len() {
        let header = data[at];
        let kind = (header >> 3) & 0x0f;
        let has_extension = header & 0x04 != 0;
        let has_size = header & 0x02 != 0;
        at += 1 + usize::from(has_extension);
        let size = if has_size {
            let (size, used) = leb128(data.get(at..)?)?;
            at += used;
            usize::try_from(size).ok()?
        } else {
            data.len().checked_sub(at)?
        };
        let payload = data.get(at..at.checked_add(size)?)?;
        if kind == SEQUENCE_HEADER {
            return sequence_header(payload);
        }
        at += size;
    }
    None
}

/// An unsigned LEB128 value, and how many bytes it took.
fn leb128(data: &[u8]) -> Option<(u64, usize)> {
    let mut value = 0u64;
    for (index, &byte) in data.iter().take(8).enumerate() {
        value |= u64::from(byte & 0x7f) << (7 * index);
        if byte & 0x80 == 0 {
            return Some((value, index + 1));
        }
    }
    None
}

/// Read a sequence header OBU's payload (section 5.5).
fn sequence_header(payload: &[u8]) -> Option<Headers> {
    let mut bits = Bits::new(payload);
    let profile = bits.bits(3)?;
    // still_picture, then reduced_still_picture_header.
    bits.skip(1)?;
    let reduced = bits.flag()?;

    let mut timing = None;
    if reduced {
        // seq_level_idx[0].
        bits.skip(5)?;
    } else {
        let timing_present = bits.flag()?;
        if timing_present {
            let units = bits.bits(32)?;
            let scale = bits.bits(32)?;
            let equal_interval = bits.flag()?;
            let ticks_per_picture = if equal_interval {
                uvlc(&mut bits)? + 1
            } else {
                0
            };
            if units > 0 && scale > 0 && equal_interval {
                timing = Some((scale, units.saturating_mul(ticks_per_picture)));
            }
            let decoder_model_info = bits.flag()?;
            let mut buffer_delay_length = 0;
            if decoder_model_info {
                buffer_delay_length = bits.bits(5)? as u32 + 1;
                // num_units_in_decoding_tick, buffer_removal_time_length_minus_1,
                // frame_presentation_time_length_minus_1.
                bits.skip(32 + 5 + 5)?;
            }
            let initial_display_delay = bits.flag()?;
            let points = bits.bits(5)? + 1;
            for _ in 0..points {
                // operating_point_idc, then seq_level_idx.
                bits.skip(12)?;
                let level = bits.bits(5)?;
                if level > 7 {
                    bits.skip(1)?;
                }
                if decoder_model_info && bits.flag()? {
                    // decoder_buffer_delay and encoder_buffer_delay, then low_delay_mode_flag.
                    bits.skip(2 * buffer_delay_length)?;
                    bits.skip(1)?;
                }
                if initial_display_delay && bits.flag()? {
                    bits.skip(4)?;
                }
            }
        } else {
            let initial_display_delay = bits.flag()?;
            let points = bits.bits(5)? + 1;
            for _ in 0..points {
                bits.skip(12)?;
                let level = bits.bits(5)?;
                if level > 7 {
                    bits.skip(1)?;
                }
                if initial_display_delay && bits.flag()? {
                    bits.skip(4)?;
                }
            }
        }
    }

    // frame_width_bits_minus_1, frame_height_bits_minus_1, then the maximum size.
    let width_bits = bits.bits(4)? as u32 + 1;
    let height_bits = bits.bits(4)? as u32 + 1;
    let width = bits.bits(width_bits)? + 1;
    let height = bits.bits(height_bits)? + 1;

    if !reduced && bits.flag()? {
        // frame_id_numbers_present_flag: two lengths.
        bits.skip(7)?;
    }
    // use_128x128_superblock, enable_filter_intra, enable_intra_edge_filter.
    bits.skip(3)?;

    if !reduced {
        // enable_interintra_compound, enable_masked_compound, enable_warped_motion,
        // enable_dual_filter.
        bits.skip(4)?;
        let order_hint = bits.flag()?;
        if order_hint {
            // enable_jnt_comp, enable_ref_frame_mvs.
            bits.skip(2)?;
        }
        // seq_choose_screen_content_tools: if 0, a flag gives the value.
        let choose_screen_content = bits.flag()?;
        let force_screen_content = if choose_screen_content {
            2
        } else {
            bits.bits(1)?
        };
        if force_screen_content > 0 {
            let choose_integer_mv = bits.flag()?;
            if !choose_integer_mv {
                bits.skip(1)?;
            }
        }
        if order_hint {
            // order_hint_bits_minus_1.
            bits.skip(3)?;
        }
    }

    // enable_superres, enable_cdef, enable_restoration.
    bits.skip(3)?;
    let (bit_depth, chroma, colour) = color_config(&mut bits, profile)?;
    Some(Headers {
        width: u32::try_from(width).ok(),
        height: u32::try_from(height).ok(),
        bit_depth: Some(bit_depth),
        chroma: Some(chroma),
        colour,
        timing,
        ..Headers::default()
    })
}

/// Read `color_config()` (section 5.5.2): the bit depth and chroma subsampling.
fn color_config(bits: &mut Bits, profile: u64) -> Option<(u8, Chroma, Colour)> {
    let high_bitdepth = bits.flag()?;
    let bit_depth = if profile == 2 && high_bitdepth {
        if bits.flag()? { 12 } else { 10 }
    } else if high_bitdepth {
        10
    } else {
        8
    };
    let mono_chrome = if profile == 1 { false } else { bits.flag()? };

    // color_description_present_flag: primaries, transfer characteristics and matrix.
    let (mut primaries, mut transfer, mut matrix) = (2u64, 2u64, 2u64);
    if bits.flag()? {
        primaries = bits.bits(8)?;
        transfer = bits.bits(8)?;
        matrix = bits.bits(8)?;
    }
    let mut colour = Colour {
        full_range: None,
        matrix: Matrix::from_code(matrix),
    };
    if mono_chrome {
        colour.full_range = Some(bits.flag()?);
        return Some((bit_depth, Chroma::Monochrome, colour));
    }
    // sRGB in 4:4:4 has no colour_range bit, and is full range.
    let srgb = primaries == 1 && transfer == 13 && matrix == 0;
    let (subsampling_x, subsampling_y) = if srgb {
        colour.full_range = Some(true);
        (0, 0)
    } else {
        colour.full_range = Some(bits.flag()?);
        match profile {
            0 => (1, 1),
            1 => (0, 0),
            _ if bit_depth == 12 => {
                let x = bits.bits(1)?;
                let y = if x == 1 { bits.bits(1)? } else { 0 };
                (x, y)
            }
            _ => (1, 0),
        }
    };
    if subsampling_x == 1 && subsampling_y == 1 {
        // chroma_sample_position.
        bits.skip(2)?;
    }
    // separate_uv_delta_q.
    bits.skip(1)?;
    let chroma = match (subsampling_x, subsampling_y) {
        (1, 1) => Chroma::Yuv420,
        (1, 0) => Chroma::Yuv422,
        _ => Chroma::Yuv444,
    };
    Some((bit_depth, chroma, colour))
}

/// A `uvlc()` value (section 4.10.3): leading zeros, then that many bits.
fn uvlc(bits: &mut Bits) -> Option<u64> {
    let mut zeros = 0u32;
    while !bits.flag()? {
        zeros += 1;
    }
    if zeros >= 32 {
        return Some(u64::from(u32::MAX));
    }
    Some(bits.bits(zeros)? + (1u64 << zeros) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leb128_reads_multi_byte_values() {
        assert_eq!(leb128(&[0x05]), Some((5, 1)));
        assert_eq!(leb128(&[0x80, 0x01]), Some((128, 2)));
    }

    #[test]
    fn a_temporal_delimiter_has_no_sequence_header() {
        // A temporal delimiter OBU with a zero size.
        assert_eq!(from_obu_stream(&[0x12, 0x00]), None);
    }
}
