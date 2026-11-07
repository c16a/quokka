//! The colour range and matrix each file states, against the tags ffprobe reads from the same files.

use std::path::{Path, PathBuf};

use quokka_core::video::{Colour, Matrix, inspect};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn colour_is_read_from_the_stream() {
    let cases = [
        (
            "colour/h264_full_bt709.mp4",
            Colour {
                full_range: Some(true),
                matrix: Some(Matrix::Bt709),
            },
        ),
        (
            "colour/h264_limited_bt601.mp4",
            Colour {
                full_range: Some(false),
                matrix: Some(Matrix::Bt601),
            },
        ),
        (
            "colour/hevc_full_bt709.mp4",
            Colour {
                full_range: Some(true),
                matrix: Some(Matrix::Bt709),
            },
        ),
    ];
    for (name, expected) in cases {
        let format = inspect(&fixture(name)).unwrap();
        assert_eq!(format.colour, expected, "{name}");
    }
}

#[test]
fn a_file_that_states_no_colour_says_nothing() {
    let format = inspect(&fixture("video/h264_8_420.mp4")).unwrap();
    assert_eq!(format.colour, Colour::default());
}
