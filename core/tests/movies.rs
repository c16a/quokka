use std::fs;
use std::path::{Path, PathBuf};

use quokka_core::Library;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn an_mp4_movie_is_indexed_with_its_video_details() {
    let dir = tempfile::tempdir().unwrap();
    fs::copy(fixture("av1_8bit.mp4"), dir.path().join("eight.mp4")).unwrap();
    fs::copy(fixture("av1_10bit.mp4"), dir.path().join("ten.mp4")).unwrap();

    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();
    let movies = lib.movies().unwrap();
    assert_eq!(movies.len(), 2);

    for movie in &movies {
        assert_eq!(movie.video_codec.as_deref(), Some("AV1"));
        assert_eq!(movie.width, Some(320));
        assert!(movie.height.is_some_and(|h| h >= 180));
        let duration = movie.duration_ms.expect("the container gives a length");
        assert!(
            (1_500..=2_500).contains(&duration),
            "duration {duration} ms"
        );
    }

    let depth = |name: &str| {
        movies
            .iter()
            .find(|m| m.path.ends_with(name))
            .and_then(|m| m.video_bit_depth)
    };
    assert_eq!(depth("eight.mp4"), Some(8));
    assert_eq!(depth("ten.mp4"), Some(10));
}

#[test]
fn a_raw_obu_movie_is_indexed_without_video_details() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("raw.obu"), b"not a container").unwrap();

    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();
    let movies = lib.movies().unwrap();

    assert_eq!(movies.len(), 1);
    assert_eq!(movies[0].video_codec, None);
    assert_eq!(movies[0].width, None);
    assert_eq!(movies[0].duration_ms, None);
}

#[test]
fn an_h264_mp4_is_named_as_h264_with_its_colour_format() {
    let dir = tempfile::tempdir().unwrap();
    fs::copy(fixture("h264_test.mp4"), dir.path().join("clip.mp4")).unwrap();
    fs::copy(fixture("h264_10bit.mp4"), dir.path().join("ten.mp4")).unwrap();

    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();
    let movies = lib.movies().unwrap();
    assert_eq!(movies.len(), 2);

    for movie in &movies {
        assert_eq!(movie.video_codec.as_deref(), Some("H.264"));
        assert_eq!(movie.video_chroma.as_deref(), Some("4:2:0"));
    }
    let depth = |name: &str| {
        movies
            .iter()
            .find(|m| m.path.ends_with(name))
            .and_then(|m| m.video_bit_depth)
    };
    assert_eq!(depth("clip.mp4"), Some(8));
    assert_eq!(depth("ten.mp4"), Some(10));
}

#[test]
fn an_hevc_mp4_is_named_with_its_colour_format() {
    let dir = tempfile::tempdir().unwrap();
    fs::copy(fixture("hevc_test.mp4"), dir.path().join("eight.mp4")).unwrap();
    fs::copy(fixture("hevc_10bit.mp4"), dir.path().join("ten.mp4")).unwrap();

    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();
    let movies = lib.movies().unwrap();
    assert_eq!(movies.len(), 2);

    for movie in &movies {
        assert_eq!(movie.video_codec.as_deref(), Some("HEVC"));
        assert_eq!(movie.video_chroma.as_deref(), Some("4:2:0"));
    }
    let depth = |name: &str| {
        movies
            .iter()
            .find(|m| m.path.ends_with(name))
            .and_then(|m| m.video_bit_depth)
    };
    assert_eq!(depth("eight.mp4"), Some(8));
    assert_eq!(depth("ten.mp4"), Some(10));
}

#[test]
fn a_changed_movie_is_processed_again_on_the_next_scan() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("clip.mp4");
    fs::copy(fixture("h264_test.mp4"), &path).unwrap();

    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();
    assert_eq!(lib.movies().unwrap()[0].video_bit_depth, Some(8));

    // A different file at the same path: the scan clears the colour format and the processing
    // step fills it in again from the new file.
    fs::copy(fixture("h264_10bit.mp4"), &path).unwrap();
    lib.scan(dir.path()).unwrap();
    let movies = lib.movies().unwrap();
    assert_eq!(movies[0].video_bit_depth, Some(10));
    assert_eq!(movies[0].video_chroma.as_deref(), Some("4:2:0"));
}
