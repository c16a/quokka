//! Saving the video frame on screen as a PNG, in a Quokka folder under the user's Pictures.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, RgbaImage};
use quokka_playback::decode::frames::Yuv;

/// Where screenshots are kept: `~/Pictures/Quokka`. Built from `HOME` rather than the platform's
/// picture folder lookup, so it follows the same home folder as the rest of the app's files.
pub fn folder() -> PathBuf {
    let home = std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join("Pictures").join("Quokka")
}

/// Writes the frame as a PNG named after the title and the position, and returns its path. An
/// existing file is never overwritten: the name gets a number before the extension instead.
pub fn save(frame: &Yuv, title: &str, position_secs: f64) -> Result<PathBuf, String> {
    let rgba = frame.to_rgba();
    let (width, height) = (rgba.width, rgba.height);
    let image = RgbaImage::from_raw(width, height, rgba.pixels).ok_or_else(|| {
        format!("The frame is {width} by {height}, but its pixels do not fill that size")
    })?;

    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(image.as_raw(), width, height, ExtendedColorType::Rgba8)
        .map_err(|error| format!("Could not encode the frame as PNG: {error}"))?;

    let folder = folder();
    fs::create_dir_all(&folder)
        .map_err(|error| format!("Could not create {}: {error}", folder.display()))?;
    write_new(&folder, &file_name(title, position_secs), &bytes)
}

/// The file name for a frame: the sanitised title, the position, and the extension.
fn file_name(title: &str, position_secs: f64) -> String {
    let title = title.trim();
    let title: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let title = if title.is_empty() {
        "Frame"
    } else {
        title.as_str()
    };
    format!("{title} {}.png", timestamp(position_secs))
}

/// The position as `h.mm.ss.mmm`. A negative or NaN position counts as zero.
fn timestamp(position_secs: f64) -> String {
    // The cast saturates, so a huge position cannot panic here.
    let millis = (position_secs.max(0.0) * 1000.0).round() as u64;
    let hours = millis / 3_600_000;
    let minutes = millis / 60_000 % 60;
    let seconds = millis / 1000 % 60;
    let millis = millis % 1000;
    format!("{hours}.{minutes:02}.{seconds:02}.{millis:03}")
}

/// Creates `name` in `folder` and writes `bytes` to it. If the name is taken, it tries
/// ` 2`, ` 3` and so on before the extension. The file is opened with `create_new`, so the
/// check and the write cannot race with another save into the same name.
fn write_new(folder: &Path, name: &str, bytes: &[u8]) -> Result<PathBuf, String> {
    let stem = name.strip_suffix(".png").unwrap_or(name);
    let mut number = 1;
    loop {
        let candidate = if number == 1 {
            name.to_owned()
        } else {
            format!("{stem} {number}.png")
        };
        let path = folder.join(candidate);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(bytes)
                    .map_err(|error| format!("Could not write {}: {error}", path.display()))?;
                return Ok(path);
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => number += 1,
            Err(error) => return Err(format!("Could not create {}: {error}", path.display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_name_has_title_and_timestamp() {
        assert_eq!(file_name("Film", 83.456), "Film 0.01.23.456.png");
        assert_eq!(file_name("Film", 3725.5), "Film 1.02.05.500.png");
    }

    #[test]
    fn file_name_replaces_unsafe_characters() {
        assert_eq!(
            file_name("Film: Part/2", 0.0),
            "Film_ Part_2 0.00.00.000.png"
        );
        assert_eq!(file_name("Café", 0.0), "Café 0.00.00.000.png");
    }

    #[test]
    fn file_name_uses_frame_for_empty_title() {
        assert_eq!(file_name("", 0.0), "Frame 0.00.00.000.png");
        assert_eq!(file_name("   ", 0.0), "Frame 0.00.00.000.png");
    }

    #[test]
    fn negative_position_counts_as_zero() {
        assert_eq!(file_name("Film", -1.0), "Film 0.00.00.000.png");
    }
}
