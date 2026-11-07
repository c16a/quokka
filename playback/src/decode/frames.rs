//! Turning decoded pictures into RGBA pixels for display.

use std::borrow::Cow;

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
            b"420v" => (true, 8, false, 1, 1),
            b"422v" => (true, 8, false, 1, 0),
            b"444v" => (true, 8, false, 0, 0),
            b"x420" => (true, 10, true, 1, 1),
            b"x422" => (true, 10, true, 1, 0),
            b"x444" => (true, 10, true, 0, 0),
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

/// The RGBA pixels of a picture, or `None` when its pixel format is not one this converts.
pub fn to_rgba(picture: &Picture) -> Option<Rgba> {
    let layout = Layout::of(&picture.pixel_format)?;
    let samples: Cow<[u8]> = match &picture.surface {
        Surface::Cpu(bytes) => Cow::Borrowed(bytes),
        #[cfg(target_os = "macos")]
        Surface::Platform(buffer) => Cow::Owned(buffer.to_bytes()),
    };
    Some(convert(picture.width, picture.height, layout, &samples))
}

/// Convert Y'CbCr to RGB, with the BT.709 matrix and limited range, which is what HD video uses.
fn convert(width: u32, height: u32, layout: Layout, samples: &[u8]) -> Rgba {
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

    let mut pixels = vec![0u8; w * h * 4];
    for y in 0..h {
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
            let (y_part, cb, cr) = (1.164 * (luma - 16.0), cb - 128.0, cr - 128.0);
            let r = y_part + 1.793 * cr;
            let g = y_part - 0.213 * cb - 0.533 * cr;
            let b = y_part + 2.112 * cb;
            let at = (y * w + x) * 4;
            pixels[at] = clamp(r);
            pixels[at + 1] = clamp(g);
            pixels[at + 2] = clamp(b);
            pixels[at + 3] = 255;
        }
    }
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
        let black = to_rgba(&picture(
            b"I420",
            vec![16, 16, 16, 16, 128, 128, 128, 128, 128],
        ))
        .unwrap();
        assert!(
            black
                .pixels
                .chunks(4)
                .all(|px| px[..3] == [0, 0, 0] && px[3] == 255)
        );
        let white = to_rgba(&picture(
            b"I420",
            vec![235, 235, 235, 235, 128, 128, 128, 128, 128],
        ))
        .unwrap();
        assert!(white.pixels.chunks(4).all(|px| px[..3] == [255, 255, 255]));
    }

    #[test]
    fn the_hardware_layout_matches_the_planar_one() {
        // The same picture, once planar and once biplanar with interleaved chroma.
        let planar = picture(b"I420", vec![100, 150, 60, 200, 128, 128, 128, 128, 128]);
        let biplanar = picture(b"420v", vec![100, 150, 60, 200, 128, 128, 128, 128, 128]);
        assert_eq!(to_rgba(&planar), to_rgba(&biplanar));
    }

    #[test]
    fn unknown_formats_are_not_converted() {
        assert!(to_rgba(&picture(b"????", vec![0; 16])).is_none());
    }
}
