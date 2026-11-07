//! VP9 headers, from the `vpcC` box of an MP4 file. Its colour byte gives the bit depth and the
//! chroma subsampling. The picture size and frame rate come from the container.

use super::bits::Bits;
use super::{Chroma, Headers};

/// The headers of a `vpcC` box body. It is a full box: four bytes of version and flags, then the
/// profile, the level, and one byte holding the bit depth, chroma subsampling and range.
pub(crate) fn from_vpcc(record: &[u8]) -> Option<Headers> {
    let colour = *record.get(6)?;
    let bit_depth = colour >> 4;
    let chroma = match (colour >> 1) & 0x07 {
        0 | 1 => Chroma::Yuv420,
        2 => Chroma::Yuv422,
        3 => Chroma::Yuv444,
        _ => return None,
    };
    Some(Headers {
        bit_depth: matches!(bit_depth, 8 | 10 | 12).then_some(bit_depth),
        chroma: Some(chroma),
        ..Headers::default()
    })
}

/// The headers of a VP9 keyframe, from its uncompressed header (VP9 bitstream section 6.2). Matroska
/// files often leave the colour fields out of the track, so the first frame is read instead.
pub(crate) fn from_keyframe(frame: &[u8]) -> Option<Headers> {
    let mut bits = Bits::new(frame);
    // frame_marker, then profile_low_bit and profile_high_bit.
    if bits.bits(2)? != 2 {
        return None;
    }
    let low = bits.bit()?;
    let high = bits.bit()?;
    let profile = (high << 1) | low;
    if profile == 3 {
        bits.skip(1)?;
    }
    // show_existing_frame, then frame_type (0 is a keyframe).
    if bits.flag()? || bits.flag()? {
        return None;
    }
    // show_frame, error_resilient_mode, then the sync code.
    bits.skip(2)?;
    if bits.bits(24)? != 0x49_83_42 {
        return None;
    }
    let bit_depth = if profile >= 2 {
        if bits.flag()? { 12 } else { 10 }
    } else {
        8
    };
    let colour_space = bits.bits(3)?;
    let (subsampling_x, subsampling_y) = if colour_space != 7 {
        // color_range, then the subsampling for profiles 1 and 3.
        bits.skip(1)?;
        if profile == 1 || profile == 3 {
            let x = bits.bit()?;
            let y = bits.bit()?;
            bits.skip(1)?;
            (x, y)
        } else {
            (1, 1)
        }
    } else {
        // sRGB is always 4:4:4 in profiles 1 and 3, and has a reserved bit there.
        if profile == 1 || profile == 3 {
            bits.skip(1)?;
        }
        (0, 0)
    };
    let chroma = match (subsampling_x, subsampling_y) {
        (1, 1) => Chroma::Yuv420,
        (1, 0) => Chroma::Yuv422,
        (0, 0) => Chroma::Yuv444,
        _ => return None,
    };
    // frame_width_minus_1 and frame_height_minus_1.
    let width = bits.bits(16)? + 1;
    let height = bits.bits(16)? + 1;
    Some(Headers {
        width: u32::try_from(width).ok(),
        height: u32::try_from(height).ok(),
        bit_depth: Some(bit_depth),
        chroma: Some(chroma),
        ..Headers::default()
    })
}
