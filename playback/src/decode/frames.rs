//! Turning decoded pictures into pixels for display. A picture's samples are kept as they are, as
//! Y'CbCr planes, so the display can convert them on the GPU. The conversion here is the reference
//! that the GPU conversion is checked against.

use std::num::NonZeroUsize;
use std::thread;

use quokka_core::video::{Colour, Matrix};

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
pub struct Layout {
    /// Chroma planes are interleaved in one plane, rather than one plane each.
    pub biplanar: bool,
    /// Bits per sample: 8, 10 or 12.
    pub bits: u8,
    /// Samples above 8 bits sit in the high bits of their 16-bit words, not the low bits.
    pub high_aligned: bool,
    /// Chroma is halved horizontally and vertically: 0 or 1 for each.
    pub shift_x: u32,
    pub shift_y: u32,
}

impl Layout {
    /// The layout a four-character code names. Planar codes come from the bundled decoders,
    /// biplanar ones from VideoToolbox.
    pub fn of(code: &[u8; 4]) -> Option<Layout> {
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
    pub fn bytes(self) -> usize {
        if self.bits > 8 { 2 } else { 1 }
    }

    /// The width and height of each chroma plane, in samples.
    pub fn chroma_size(self, width: usize, height: usize) -> (usize, usize) {
        (
            (width + self.shift_x as usize) >> self.shift_x,
            (height + self.shift_y as usize) >> self.shift_y,
        )
    }

    /// The bits to drop from a sample to get the 8-bit value the conversion works on.
    pub fn sample_shift(self) -> u32 {
        if self.bits == 8 {
            0
        } else if self.high_aligned {
            16 - 8
        } else {
            u32::from(self.bits) - 8
        }
    }

    /// The bytes a picture of this layout takes: the luma plane, then both chroma planes.
    pub fn total_bytes(self, width: u32, height: u32) -> usize {
        let (w, h) = (width as usize, height as usize);
        let (cw, ch) = self.chroma_size(w, h);
        (w * h + 2 * cw * ch) * self.bytes()
    }
}

/// The numbers that turn Y'CbCr into RGB, for one matrix and range. Each sample is first scaled to
/// 8 bits: luma by `luma_scale` after subtracting `luma_offset`, and chroma by `chroma_scale` after
/// centring on 128. Each colour then adds its share of the samples, with green subtracting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coefficients {
    pub luma_offset: f32,
    pub luma_scale: f32,
    pub chroma_scale: f32,
    pub r_from_cr: f32,
    pub g_from_cb: f32,
    pub g_from_cr: f32,
    pub b_from_cb: f32,
}

/// The conversion numbers for a matrix and range.
pub fn coefficients(matrix: Matrix, full_range: bool) -> Coefficients {
    let (kr, kb) = match matrix {
        Matrix::Bt601 => (0.299, 0.114),
        Matrix::Bt709 => (0.2126, 0.0722),
        Matrix::Bt2020 => (0.2627, 0.0593),
    };
    let kg = 1.0 - kr - kb;
    // Full range spans 0 to 255; limited range spans 16 to 235 for luma and 16 to 240 for chroma.
    let (luma_offset, luma_scale, chroma_scale) = if full_range {
        (0.0, 1.0 / 255.0, 1.0 / 255.0)
    } else {
        (16.0, 1.0 / 219.0, 1.0 / 224.0)
    };
    Coefficients {
        luma_offset,
        luma_scale,
        chroma_scale,
        r_from_cr: 2.0 * (1.0 - kr),
        g_from_cb: 2.0 * kb * (1.0 - kb) / kg,
        g_from_cr: 2.0 * kr * (1.0 - kr) / kg,
        b_from_cb: 2.0 * (1.0 - kb),
    }
}

/// A picture's samples as Y'CbCr planes, with the colour the stream states.
#[derive(Debug, Clone)]
pub struct Yuv {
    pub width: u32,
    pub height: u32,
    pub layout: Layout,
    pub matrix: Matrix,
    pub full_range: bool,
    /// The planes one after another: luma, then the Cb and Cr planes or one interleaved CbCr plane.
    /// Samples above 8 bits are 16-bit little-endian.
    pub samples: Vec<u8>,
}

impl Yuv {
    /// The picture's samples and colour. `None` when its pixel format is not one this handles, or
    /// its samples are shorter than the format needs. Where the stream does not state the colour,
    /// the picture is taken as limited range, in BT.709 when it is high definition and BT.601 when
    /// it is not.
    pub fn from_picture(picture: Picture, colour: Colour) -> Option<Yuv> {
        let layout = Layout::of(&picture.pixel_format)?;
        // Only macOS decoders hand over a platform buffer; elsewhere every picture is CPU bytes.
        #[cfg(target_os = "macos")]
        let samples = match picture.surface {
            Surface::Cpu(bytes) => bytes,
            Surface::Platform(buffer) => buffer.to_bytes(),
        };
        #[cfg(not(target_os = "macos"))]
        let Surface::Cpu(samples) = picture.surface;
        if samples.len() < layout.total_bytes(picture.width, picture.height) {
            return None;
        }
        let matrix = colour.matrix.unwrap_or(if picture.height > 576 {
            Matrix::Bt709
        } else {
            Matrix::Bt601
        });
        Some(Yuv {
            width: picture.width,
            height: picture.height,
            layout,
            matrix,
            full_range: colour.full_range.unwrap_or(false),
            samples,
        })
    }

    /// The picture converted to RGBA on the CPU.
    pub fn to_rgba(&self) -> Rgba {
        convert(
            self.width,
            self.height,
            self.layout,
            &self.samples,
            self.matrix,
            self.full_range,
        )
    }
}

/// The RGBA pixels of a picture, converted on the CPU, or `None` as for [`Yuv::from_picture`].
pub fn to_rgba(picture: &Picture, colour: Colour) -> Option<Rgba> {
    let surface = match &picture.surface {
        Surface::Cpu(bytes) => Surface::Cpu(bytes.clone()),
        #[cfg(target_os = "macos")]
        Surface::Platform(buffer) => Surface::Cpu(buffer.to_bytes()),
    };
    let copy = Picture {
        width: picture.width,
        height: picture.height,
        presentation_time: picture.presentation_time,
        pixel_format: picture.pixel_format,
        surface,
    };
    Yuv::from_picture(copy, colour).map(|yuv| yuv.to_rgba())
}

/// Convert Y'CbCr samples to RGBA, with the given matrix and range.
fn convert(
    width: u32,
    height: u32,
    layout: Layout,
    samples: &[u8],
    matrix: Matrix,
    full_range: bool,
) -> Rgba {
    let c = coefficients(matrix, full_range);
    let (w, h) = (width as usize, height as usize);
    if w == 0 || h == 0 {
        return Rgba {
            width,
            height,
            pixels: Vec::new(),
        };
    }
    let (cw, ch) = layout.chroma_size(w, h);
    let bytes = layout.bytes();
    let luma_bytes = w * h * bytes;
    let chroma_bytes = cw * ch * bytes;
    let shift = layout.sample_shift();

    // Reads one sample: the byte offset is where the sample starts, and the value is scaled to 8 bits.
    let read = |offset: usize| -> u32 {
        let raw = match bytes {
            2 => u32::from(u16::from_le_bytes([
                samples.get(offset).copied().unwrap_or(0),
                samples.get(offset + 1).copied().unwrap_or(0),
            ])),
            _ => u32::from(samples.get(offset).copied().unwrap_or(0)),
        };
        raw >> shift
    };

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
            let y_part = (luma - c.luma_offset) * c.luma_scale;
            let cb = (cb - 128.0) * c.chroma_scale;
            let cr = (cr - 128.0) * c.chroma_scale;
            let r = 255.0 * (y_part + c.r_from_cr * cr);
            let g = 255.0 * (y_part - c.g_from_cb * cb - c.g_from_cr * cr);
            let b = 255.0 * (y_part + c.b_from_cb * cb);
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

    #[test]
    fn short_samples_are_refused() {
        // A 2x2 4:2:0 picture needs 4 luma and 2 chroma bytes: 6 bytes, not 4.
        let short = picture(b"I420", vec![0; 4]);
        assert!(Yuv::from_picture(short, Colour::default()).is_none());
    }
}
