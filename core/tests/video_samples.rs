//! The sample readers for MP4, Matroska and WebM against the video fixtures. Each fixture has
//! 48 frames, and every frame must lie inside the file.

use std::path::{Path, PathBuf};

use quokka_core::video::{Codec, read_samples};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/video")
        .join(name)
}

#[test]
fn every_container_yields_all_frames_of_the_video_track() {
    let cases = [
        ("h264_8_420.mp4", Codec::H264),
        ("h264_8_420.mkv", Codec::H264),
        ("hevc_8_420.mp4", Codec::Hevc),
        ("hevc_8_420.mkv", Codec::Hevc),
        ("av1_8_420.mp4", Codec::Av1),
        ("av1_8_420.mkv", Codec::Av1),
        ("av1_8_420.webm", Codec::Av1),
        ("av1_10_420.mkv", Codec::Av1),
        ("vp9_8_420.mp4", Codec::Vp9),
        ("vp9_8_420.webm", Codec::Vp9),
        ("vp9_8_420.mkv", Codec::Vp9),
        ("vp9_10_420.webm", Codec::Vp9),
    ];
    for (name, codec) in cases {
        let path = fixture(name);
        let track = read_samples(&path)
            .unwrap_or_else(|error| panic!("{name}: {error}"))
            .unwrap_or_else(|| panic!("{name}: no video track"));
        assert_eq!(track.codec, codec, "{name}");
        assert_eq!((track.width, track.height), (320, 180), "{name}");
        assert_eq!(track.samples.len(), 48, "{name}: frame count");
        assert!(
            track.samples[0].sync,
            "{name}: the first frame is a keyframe"
        );
        let file_length = std::fs::metadata(&path).unwrap().len();
        for sample in &track.samples {
            assert!(sample.size > 0, "{name}: empty frame");
            assert!(
                sample.offset + u64::from(sample.size) <= file_length,
                "{name}: frame outside the file"
            );
        }
    }
}

#[test]
fn a_file_that_is_not_video_has_no_samples() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.txt");
    std::fs::write(&path, b"plain text, not a container").unwrap();
    assert!(read_samples(&path).unwrap().is_none());
}
