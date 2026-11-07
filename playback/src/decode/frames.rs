//! Turning decoded pictures into RGBA pixels for display.

use std::borrow::Cow;
use std::num::NonZeroUsize;
use std::thread;

use player_core::video::{Colour, Matrix};

use super::{Picture, Surface};

/// A picture as tight RGBA rows, four bytes a pixel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// How a picture's samples are laid out in memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Layout {
    /// Chroma planes are interleaved in one plane, rather than one plane each.
    biplanar: bool,
    /// Bits per sample: 8, 10 or 12.
    bits: u8,
    /// Samples above 8 bits sit in the high bits of their 16-bit words, not the low bits.
    high_aligned: bool,
    /// Chroma is halved horizontally and vertically: 0 or 1 for each.
    shift_x: u32,
    shift_y: u32,
}

impl Layout {
    /// The layout a four-character code names. Planar codes come from the bundled decoders,
    /// biplanar ones from VideoToolbox.
    fn of(code: &[u8; 4]) -> Option<Layout> {
        let (biplanar, bits, high_aligned, shift_x, shift_y) = match code {
            b"I420" => (false, 8, false, 1, 1),
            b"I422" => (false, 8, false, 1, 0),
            b"I444" => (false, 8, false, 0, 0),
            b"I010" => (false, 10, false, 1, 1),
            b"I210" => (false, 10, false, 1, 0),
            b"I410" => (false, 10, false, 0, 0),
            b"I012" => (false, 12, false, 1, 1),
            b"I212" => (false, 12, false, 1, 0),
            b"I412" => (false, 12, false, 0, 0),
            b"420v" | b"420f" => (true, 8, false, 1, 1),
            b"422v" | b"422f" => (true, 8, false, 1, 0),
            b"444v" | b"444f" => (true, 8, false, 0, 0),
            b"x420" | b"xf20" => (true, 10, true, 1, 1),
            b"x422" | b"xf22" => (true, 10, true, 1, 0),
            b"x444" | b"xf44" => (true, 10, true, 0, 0),
            _ => return None,
        };
        Some(Layout {
            biplanar,
            bits,
            high_aligned,
            shift_x,
            shift_y,
        })
    }

    /// Bytes per sample in memory.
    fn bytes(self) -> usize {
        if self.bits > 8 { 2 } else { 1 }
    }
}

/// The RGBA pixels of a picture, or `None` when its pixel format is not one this converts. The
/// colour is what the stream states. Where it does not, the picture is taken as limited range, in
/// BT.709 when it is high definition and BT.601 when it is not.
pub fn to_rgba(picture: &Picture, colour: Colour) -> Option<Rgba> {
    let layout = Layout::of(&picture.pixel_format)?;
    let samples: Cow<[u8]> = match &picture.surface {
        Surface::Cpu(bytes) => Cow::Borrowed(bytes),
        #[cfg(target_os = "macos")]
        Surface::Platform(buffer) => Cow::Owned(buffer.to_bytes()),
    };
    let matrix = colour.matrix.unwrap_or(if picture.height > 576 {
        Matrix::Bt709
    } else {
        Matrix::Bt601
    });
    Some(convert(
        picture.width,
        picture.height,
        layout,
        &samples,
        matrix,
        colour.full_range.unwrap_or(false),
    ))
}

/// The luma weights of a matrix, `(Kr, Kb)`. The green weights follow from them.
fn luma_weights(matrix: Matrix) -> (f32, f32) {
    match matrix {
        Matrix::Bt601 => (0.299, 0.114),
        Matrix::Bt709 => (0.2126, 0.0722),
        Matrix::Bt2020 => (0.2627, 0.0593),
    }
}

/// Convert Y'CbCr to RGB with the given matrix, and full or limited range.
fn convert(
    width: u32,
    height: u32,
    layout: Layout,
    samples: &[u8],
    matrix: Matrix,
    full_range: bool,
) -> Rgba {
    let (kr, kb) = luma_weights(matrix);
    let kg = 1.0 - kr - kb;
    // How much each chroma sample adds to each colour (green subtracts).
    let r_from_cr = 2.0 * (1.0 - kr);
    let g_from_cb = 2.0 * kb * (1.0 - kb) / kg;
    let g_from_cr = 2.0 * kr * (1.0 - kr) / kg;
    let b_from_cb = 2.0 * (1.0 - kb);
    // Full range spans 0 to 255; limited range spans 16 to 235 for luma and 16 to 240 for chroma.
    let (luma_offset, luma_scale, chroma_scale) = if full_range {
        (0.0, 1.0 / 255.0, 1.0 / 255.0)
    } else {
        (16.0, 1.0 / 219.0, 1.0 / 224.0)
    };
    let (w, h) = (width as usize, height as usize);
    let (cw, ch) = (
        (w + layout.shift_x as usize) >> layout.shift_x,
        (h + layout.shift_y as usize) >> layout.shift_y,
    );
    let bytes = layout.bytes();
    let luma_bytes = w * h * bytes;
    let chroma_bytes = cw * ch * bytes;

    // Reads one sample: the byte offset is where the sample starts, and the value is scaled to 8 bits.
    let read = |offset: usize| -> u32 {
        let raw = match bytes {
            2 => u32::from(u16::from_le_bytes([
                samples.get(offset).copied().unwrap_or(0),
                samples.get(offset + 1).copied().unwrap_or(0),
            ])),
            _ => u32::from(samples.get(offset).copied().unwrap_or(0)),
        };
        if layout.bits == 8 {
            raw
        } else if layout.high_aligned {
            raw >> (16 - 8)
        } else {
            raw >> (layout.bits - 8)
        }
    };

    if w == 0 || h == 0 {
        return Rgba {
            width,
            height,
            pixels: Vec::new(),
        };
    }
    // Converts one row into `out`, four bytes a pixel. Rows do not depend on each other.
    let convert_row = |y: usize, out: &mut [u8]| {
        for x in 0..w {
            let luma = read((y * w + x) * bytes) as f32;
            let chroma = (y >> layout.shift_y) * cw + (x >> layout.shift_x);
            let (cb, cr) = if layout.biplanar {
                let at = luma_bytes + chroma * 2 * bytes;
                (read(at) as f32, read(at + bytes) as f32)
            } else {
                (
                    read(luma_bytes + chroma * bytes) as f32,
                    read(luma_bytes + chroma_bytes + chroma * bytes) as f32,
                )
            };
            let y_part = (luma - luma_offset) * luma_scale;
            let cb = (cb - 128.0) * chroma_scale;
            let cr = (cr - 128.0) * chroma_scale;
            let r = 255.0 * (y_part + r_from_cr * cr);
            let g = 255.0 * (y_part - g_from_cb * cb - g_from_cr * cr);
            let b = 255.0 * (y_part + b_from_cb * cb);
            let at = x * 4;
            out[at] = clamp(r);
            out[at + 1] = clamp(g);
            out[at + 2] = clamp(b);
            out[at + 3] = 255;
        }
    };
    // Bands of rows convert on separate threads, one band per available core.
    let mut pixels = vec![0u8; w * h * 4];
    let threads = thread::available_parallelism()
        .map_or(1, NonZeroUsize::get)
        .min(h);
    let rows_per_band = h.div_ceil(threads);
    thread::scope(|scope| {
        for (band, rows) in pixels.chunks_mut(rows_per_band * w * 4).enumerate() {
            let convert_row = &convert_row;
            scope.spawn(move || {
                for (i, row) in rows.chunks_mut(w * 4).enumerate() {
                    convert_row(band * rows_per_band + i, row);
                }
            });
        }
    });
    Rgba {
        width,
        height,
        pixels,
    }
}

fn clamp(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picture(code: &[u8; 4], samples: Vec<u8>) -> Picture {
        Picture {
            width: 2,
            height: 2,
            presentation_time: 0,
            pixel_format: *code,
            surface: Surface::Cpu(samples),
        }
    }

    #[test]
    fn video_black_and_white_convert_to_black_and_white() {
        // 2x2 4:2:0 with neutral chroma: luma 16 is black and 235 is white.
        let black = to_rgba(
            &picture(b"I420", vec![16, 16, 16, 16, 128, 128, 128, 128, 128]),
            Colour::default(),
        )
        .unwrap();
        assert!(
            black
                .pixels
                .chunks(4)
                .all(|px| px[..3] == [0, 0, 0] && px[3] == 255)
        );
        let white = to_rgba(
            &picture(b"I420", vec![235, 235, 235, 235, 128, 128, 128, 128, 128]),
            Colour::default(),
        )
        .unwrap();
        assert!(white.pixels.chunks(4).all(|px| px[..3] == [255, 255, 255]));
    }

    #[test]
    fn the_hardware_layout_matches_the_planar_one() {
        // The same picture, once planar and once biplanar with interleaved chroma.
        let planar = picture(b"I420", vec![100, 150, 60, 200, 128, 128, 128, 128, 128]);
        let biplanar = picture(b"420v", vec![100, 150, 60, 200, 128, 128, 128, 128, 128]);
        assert_eq!(
            to_rgba(&planar, Colour::default()),
            to_rgba(&biplanar, Colour::default())
        );
    }

    #[test]
    fn full_range_uses_the_whole_scale() {
        let full = Colour {
            full_range: Some(true),
            matrix: Some(Matrix::Bt709),
        };
        let white = to_rgba(
            &picture(b"I420", vec![255, 255, 255, 255, 128, 128, 128, 128, 128]),
            full,
        )
        .unwrap();
        assert!(white.pixels.chunks(4).all(|px| px[..3] == [255, 255, 255]));
        let black = to_rgba(
            &picture(b"I420", vec![0, 0, 0, 0, 128, 128, 128, 128, 128]),
            full,
        )
        .unwrap();
        assert!(black.pixels.chunks(4).all(|px| px[..3] == [0, 0, 0]));
    }

    #[test]
    fn limited_range_black_is_black_in_any_matrix() {
        for matrix in [Matrix::Bt601, Matrix::Bt709, Matrix::Bt2020] {
            let colour = Colour {
                full_range: Some(false),
                matrix: Some(matrix),
            };
            let black = to_rgba(
                &picture(b"I420", vec![16, 16, 16, 16, 128, 128, 128, 128, 128]),
                colour,
            )
            .unwrap();
            assert!(
                black.pixels.chunks(4).all(|px| px[..3] == [0, 0, 0]),
                "{matrix:?}"
            );
        }
    }

    #[test]
    fn the_matrix_changes_saturated_colours() {
        // A red chroma sample, limited range: BT.601 and BT.709 give different reds.
        let sample = vec![81, 81, 81, 81, 90, 90, 90, 90, 240];
        let red_601 = to_rgba(
            &picture(b"I420", sample.clone()),
            Colour {
                full_range: Some(false),
                matrix: Some(Matrix::Bt601),
            },
        )
        .unwrap();
        let red_709 = to_rgba(
            &picture(b"I420", sample),
            Colour {
                full_range: Some(false),
                matrix: Some(Matrix::Bt709),
            },
        )
        .unwrap();
        assert_ne!(red_601.pixels, red_709.pixels);
    }

    #[test]
    fn unknown_formats_are_not_converted() {
        assert!(to_rgba(&picture(b"????", vec![0; 16]), Colour::default()).is_none());
    }
}
