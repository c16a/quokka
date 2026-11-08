//! H.264 and HEVC decode through Media Foundation on Windows. H.264 is decoded by the decoder that ships
//! with Windows, so those files must come out whole. HEVC needs the HEVC Video Extensions: when they are
//! missing, the file must be refused with a message that names them, not fail silently.

#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use quokka_playback::decode::player::{FrameError, FramePlayer};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../core/tests/fixtures/video")
        .join(name)
}

/// Decode a whole file. Returns the frames that came out and the number the container states.
fn decode_all(path: &Path) -> Result<(usize, usize), FrameError> {
    let mut player = FramePlayer::open(path)?;
    let expected = player.frame_count;
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut count = 0;
    loop {
        match player.next_frame().expect("no decode error") {
            Some(_) => count += 1,
            None if player.finished() => break,
            None => {
                assert!(Instant::now() < deadline, "decoding did not finish");
                sleep(Duration::from_millis(1));
            }
        }
    }
    Ok((count, expected))
}

#[test]
fn h264_in_mp4_keeps_every_frame() {
    let (count, expected) = decode_all(&fixture("h264_8_420.mp4")).expect("H.264 opens");
    assert!(count > 0, "no frames came out");
    assert_eq!(count, expected, "frames out of h264_8_420.mp4");
}

#[test]
fn h264_in_mkv_keeps_every_frame() {
    let (count, expected) = decode_all(&fixture("h264_8_420.mkv")).expect("H.264 opens");
    assert_eq!(count, expected, "frames out of h264_8_420.mkv");
}

#[test]
fn hevc_decodes_or_names_the_missing_extension() {
    match decode_all(&fixture("hevc_10_420.mp4")) {
        Ok((count, expected)) => {
            assert!(count > 0, "no frames came out");
            assert_eq!(count, expected, "frames out of hevc_10_420.mp4");
        }
        Err(FrameError::NoDecoder(reason)) => assert!(
            reason.contains("HEVC Video Extensions"),
            "the refusal does not say what is missing: {reason}"
        ),
        Err(other) => panic!("unexpected error: {other}"),
    }
}
