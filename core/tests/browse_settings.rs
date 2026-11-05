use std::fs;

use player_core::{Library, Settings};

#[test]
fn albums_group_tracks_and_movies_list_videos() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("one.mp3"), b"x").unwrap();
    fs::write(dir.path().join("two.mp3"), b"x").unwrap();
    fs::write(dir.path().join("film.mkv"), b"x").unwrap();

    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();

    // Untagged test files: one album group with no artist or album name.
    let albums = lib.albums().unwrap();
    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].track_count, 2);
    assert!(albums[0].artists.is_empty());
    assert!(albums[0].composers.is_empty());

    let tracks = lib.album_tracks(None).unwrap();
    assert_eq!(tracks.len(), 2);

    let movies = lib.movies().unwrap();
    assert_eq!(movies.len(), 1);
    assert!(movies[0].path.ends_with("film.mkv"));
}

#[test]
fn settings_round_trip_and_default_when_missing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config").join("settings.json");

    assert_eq!(Settings::load(&path).unwrap(), Settings::default());

    let settings = Settings {
        library_root: Some(dir.path().join("music")),
    };
    settings.save(&path).unwrap();
    assert_eq!(Settings::load(&path).unwrap(), settings);
}
