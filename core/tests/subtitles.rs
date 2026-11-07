//! Subtitles from SRT files and from the text tracks of MP4 and Matroska files.

use std::fs;
use std::path::{Path, PathBuf};

use quokka_core::video::{parse_srt, read_subtitles};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn texts(cues: &[quokka_core::video::Cue]) -> Vec<&str> {
    cues.iter().map(|cue| cue.text.as_str()).collect()
}

#[test]
fn srt_cues_have_their_times_and_text() {
    let cues = parse_srt(&fs::read_to_string(fixture("subtitles/cues.srt")).unwrap());
    assert_eq!(cues.len(), 3);
    assert!((cues[0].start - 0.2).abs() < 1e-9 && (cues[0].end - 0.9).abs() < 1e-9);
    assert_eq!(cues[0].text, "First line");
    assert_eq!(cues[1].text, "Second line\nwith two rows");
    assert!((cues[2].start - 1.9).abs() < 1e-9);
}

#[test]
fn srt_tags_are_dropped() {
    let cues = parse_srt("1\n00:00:00,000 --> 00:00:01,000\n<i>Italic</i> and <b>bold</b>\n");
    assert_eq!(cues[0].text, "Italic and bold");
}

#[test]
fn an_mp4_text_track_gives_its_cues() {
    let cues = read_subtitles(&fixture("subtitles/with_mp4_text.mp4")).unwrap();
    assert_eq!(
        texts(&cues),
        vec!["First line", "Second line\nwith two rows", "Third"]
    );
    assert!((cues[0].start - 0.2).abs() < 0.01);
}

#[test]
fn a_matroska_text_track_gives_its_cues() {
    let cues = read_subtitles(&fixture("subtitles/with_mkv_text.mkv")).unwrap();
    assert_eq!(
        texts(&cues),
        vec!["First line", "Second line\nwith two rows", "Third"]
    );
    assert!((cues[2].start - 1.9).abs() < 0.01);
}

#[test]
fn an_srt_beside_the_movie_is_used() {
    let dir = tempfile::tempdir().unwrap();
    let movie = dir.path().join("movie.mp4");
    fs::copy(fixture("video/h264_8_420.mp4"), &movie).unwrap();
    fs::copy(fixture("subtitles/cues.srt"), dir.path().join("movie.srt")).unwrap();
    let cues = read_subtitles(&movie).unwrap();
    assert_eq!(cues.len(), 3);
}

#[test]
fn a_file_without_subtitles_has_none() {
    assert!(
        read_subtitles(&fixture("video/h264_8_420.mp4"))
            .unwrap()
            .is_empty()
    );
}
