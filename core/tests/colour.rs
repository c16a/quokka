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
                ..Default::default()
            },
        ),
        (
            "colour/h264_limited_bt601.mp4",
            Colour {
                full_range: Some(false),
                matrix: Some(Matrix::Bt601),
                ..Default::default()
            },
        ),
        (
            "colour/hevc_full_bt709.mp4",
            Colour {
                full_range: Some(true),
                matrix: Some(Matrix::Bt709),
                ..Default::default()
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

#[test]
fn hdr_is_read_from_the_transfer_characteristics() {
    // A PQ picture with BT.2020 primaries is HDR, and a BT.709 picture is SDR.
    let hdr = inspect(&fixture("colour/hdr_pq_bt2020_hevc.mp4")).unwrap();
    assert_eq!(hdr.colour.transfer, Some(16));
    assert_eq!(hdr.colour.primaries, Some(9));
    assert_eq!(hdr.colour.hdr(), Some(true));

    let sdr = inspect(&fixture("colour/sdr_bt709_hevc.mp4")).unwrap();
    assert_eq!(sdr.colour.transfer, Some(1));
    assert_eq!(sdr.colour.hdr(), Some(false));

    // The H.264 fixture states no transfer characteristics, so it is unknown rather than SDR.
    let unknown = inspect(&fixture("colour/h264_full_bt709.mp4")).unwrap();
    assert_eq!(unknown.colour.hdr(), None);
}
