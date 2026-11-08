//! Turning a semi-planar picture into the planar layout the rest of the decoder uses. Media Foundation
//! returns NV12 (8-bit) and P010 (10-bit): a luma plane, then one plane of interleaved Cb and Cr
//! pairs. The output is I420 or I010: luma, then all the Cb samples, then all the Cr samples. This file
//! is plain byte handling, so it builds and is tested on every platform.

/// Copy the top-left `width` by `height` picture out of an NV12 frame into I420.
///
/// `stride` is the bytes in a row of the decoded frame, which may be wider than the picture.
/// `frame_height` is the decoded frame's height, which is where the chroma plane starts. `None` when
/// the frame is shorter than its dimensions say.
pub(crate) fn nv12_to_i420(
    src: &[u8],
    stride: usize,
    frame_height: usize,
    width: usize,
    height: usize,
) -> Option<Vec<u8>> {
    repack(src, stride, frame_height, width, height, 1, 0)
}

/// Copy the top-left `width` by `height` picture out of a P010 frame into I010. P010 keeps each
/// 10-bit sample in the high bits of a 16-bit word, so each sample moves down six bits to sit in the
/// low bits, which is how I010 stores it.
pub(crate) fn p010_to_i010(
    src: &[u8],
    stride: usize,
    frame_height: usize,
    width: usize,
    height: usize,
) -> Option<Vec<u8>> {
    repack(src, stride, frame_height, width, height, 2, 6)
}

/// Shared by both: `bytes` is 1 for 8-bit samples or 2 for 16-bit ones, and `shift` drops the unused
/// low bits of 16-bit samples.
fn repack(
    src: &[u8],
    stride: usize,
    frame_height: usize,
    width: usize,
    height: usize,
    bytes: usize,
    shift: u32,
) -> Option<Vec<u8>> {
    let chroma_width = width.div_ceil(2);
    let chroma_height = height.div_ceil(2);
    let row = width * bytes;
    let chroma_row = chroma_width * 2 * bytes;
    let chroma_start = stride.checked_mul(frame_height)?;

    let mut out = Vec::with_capacity((width * height + 2 * chroma_width * chroma_height) * bytes);
    for line in 0..height {
        let start = line * stride;
        let luma = src.get(start..start + row)?;
        push_samples(&mut out, luma, bytes, shift);
    }
    let mut cb = Vec::with_capacity(chroma_width * chroma_height * bytes);
    let mut cr = Vec::with_capacity(chroma_width * chroma_height * bytes);
    for line in 0..chroma_height {
        let start = chroma_start + line * stride;
        let pairs = src.get(start..start + chroma_row)?;
        // Each pair is a Cb sample then a Cr sample, `bytes` bytes each.
        for pair in pairs.chunks_exact(2 * bytes) {
            cb.extend_from_slice(&pair[..bytes]);
            cr.extend_from_slice(&pair[bytes..]);
        }
    }
    let mut out_cb = Vec::with_capacity(cb.len());
    push_samples(&mut out_cb, &cb, bytes, shift);
    let mut out_cr = Vec::with_capacity(cr.len());
    push_samples(&mut out_cr, &cr, bytes, shift);
    out.extend_from_slice(&out_cb);
    out.extend_from_slice(&out_cr);
    Some(out)
}

/// Append samples, shifting 16-bit ones down by `shift` and keeping them little-endian.
fn push_samples(out: &mut Vec<u8>, bytes_in: &[u8], bytes: usize, shift: u32) {
    if bytes == 1 {
        out.extend_from_slice(bytes_in);
        return;
    }
    for &word in bytes_in.as_chunks::<2>().0 {
        let value = u16::from_le_bytes(word) >> shift;
        out.extend_from_slice(&value.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nv12_splits_chroma_into_two_planes() {
        // A 4x2 picture, a stride of 4: luma rows, then one row of chroma pairs.
        let frame = [
            1, 2, 3, 4, // luma row 0
            5, 6, 7, 8, // luma row 1
            10, 20, 30, 40, // chroma: Cb 10, Cr 20, Cb 30, Cr 40
        ];
        let out = nv12_to_i420(&frame, 4, 2, 4, 2).expect("the frame holds the picture");
        assert_eq!(out, [1, 2, 3, 4, 5, 6, 7, 8, 10, 30, 20, 40]);
    }

    #[test]
    fn nv12_crops_an_odd_picture() {
        // A 3x1 picture from a 4x2 frame: one chroma pair per row, rounded up.
        let frame = [1, 2, 3, 9, 4, 5, 6, 9, 10, 20, 30, 40];
        let out = nv12_to_i420(&frame, 4, 2, 3, 1).expect("the frame holds the picture");
        assert_eq!(out, [1, 2, 3, 10, 30, 20, 40]);
    }

    #[test]
    fn p010_moves_samples_to_the_low_bits() {
        // A 2x1 picture: luma 0x3FF << 6 and 0x200 << 6, chroma pair 0x100 << 6 and 0x080 << 6.
        let word = |value: u16| (value << 6).to_le_bytes();
        let mut frame = Vec::new();
        frame.extend_from_slice(&word(0x3FF));
        frame.extend_from_slice(&word(0x200));
        frame.extend_from_slice(&[0; 4]);
        frame.extend_from_slice(&word(0x100));
        frame.extend_from_slice(&word(0x080));
        let out = p010_to_i010(&frame, 8, 1, 2, 1).expect("the frame holds the picture");
        let expected: Vec<u8> = [0x3FF_u16, 0x200, 0x100, 0x080]
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();
        assert_eq!(out, expected);
    }

    #[test]
    fn short_frame_is_refused() {
        assert!(nv12_to_i420(&[1, 2, 3], 4, 2, 2, 2).is_none());
    }
}
