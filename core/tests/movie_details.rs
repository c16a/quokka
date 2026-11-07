use std::fs;
use std::path::{Path, PathBuf};

use player_core::Library;
use player_core::Settings;
use player_core::tmdb::{CastMember, MovieInfo};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn matrix() -> MovieInfo {
    MovieInfo {
        id: 603,
        title: "The Matrix".into(),
        year: Some(1999),
        runtime_minutes: Some(136),
        genres: vec!["Action".into(), "Science Fiction".into()],
        tagline: Some("Welcome to the Real World.".into()),
        overview: Some("A computer hacker learns the truth.".into()),
        poster_path: None,
        directors: vec!["Lana Wachowski".into()],
        writers: vec!["Lana Wachowski".into()],
        cast: vec![CastMember {
            name: "Keanu Reeves".into(),
            character: "Neo".into(),
            profile_path: Some("/keanu.jpg".into()),
        }],
        photos: Default::default(),
        photos_fetched: true,
    }
}

#[test]
fn saved_details_are_read_back_for_their_movie() {
    let dir = tempfile::tempdir().unwrap();
    fs::copy(fixture("av1_8bit.mp4"), dir.path().join("The.Matrix.mp4")).unwrap();
    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();
    let id = lib.movies().unwrap()[0].id;

    assert_eq!(lib.movie_details(id).unwrap(), None);
    lib.save_movie_details(id, &matrix()).unwrap();
    assert_eq!(lib.movie_details(id).unwrap(), Some(matrix()));
    assert_eq!(lib.all_movie_details().unwrap().get(&id), Some(&matrix()));
}

#[test]
fn details_go_when_their_movie_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("The.Matrix.mp4");
    fs::copy(fixture("av1_8bit.mp4"), &file).unwrap();
    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();
    let id = lib.movies().unwrap()[0].id;
    lib.save_movie_details(id, &matrix()).unwrap();

    fs::remove_file(&file).unwrap();
    lib.scan(dir.path()).unwrap();

    assert!(lib.movies().unwrap().is_empty());
    assert!(lib.all_movie_details().unwrap().is_empty());
}

#[test]
fn an_unset_key_is_not_written_to_the_settings_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    Settings::default().save(&path).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(!text.contains("tmdb_api_key"));
}

#[test]
fn a_saved_key_round_trips_and_the_file_is_owner_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let settings = Settings {
        tmdb_api_key: Some("test-key-123".into()),
        ..Settings::default()
    };
    settings.save(&path).unwrap();
    assert_eq!(
        Settings::load(&path).unwrap().tmdb_api_key.as_deref(),
        Some("test-key-123")
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}

#[test]
fn cleared_details_are_gone_and_the_movie_stays() {
    let dir = tempfile::tempdir().unwrap();
    fs::copy(fixture("av1_8bit.mp4"), dir.path().join("The.Matrix.mp4")).unwrap();
    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();
    let id = lib.movies().unwrap()[0].id;
    lib.save_movie_details(id, &matrix()).unwrap();

    lib.clear_movie_details(id).unwrap();

    assert_eq!(lib.movie_details(id).unwrap(), None);
    assert_eq!(lib.movies().unwrap().len(), 1);
}
