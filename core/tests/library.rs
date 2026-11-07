use std::fs;

use quokka_core::{Library, MediaKind};

#[test]
fn scan_indexes_media_skips_other_and_hidden_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("moonlight.mp3"), b"not really audio").unwrap();
    fs::write(dir.path().join("sub/clip.mkv"), b"not really video").unwrap();
    fs::write(dir.path().join("notes.txt"), b"text").unwrap();
    fs::write(dir.path().join(".hidden.mp3"), b"hidden").unwrap();

    let mut lib = Library::open_in_memory().unwrap();
    let stats = lib.scan(dir.path()).unwrap();
    assert_eq!(stats.indexed, 2);
    assert_eq!(lib.count().unwrap(), 2);
}

#[test]
fn rescan_skips_unchanged_and_removes_deleted() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.mp3"), b"a").unwrap();
    fs::write(dir.path().join("b.mp3"), b"b").unwrap();

    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();

    let again = lib.scan(dir.path()).unwrap();
    assert_eq!(again.indexed, 0);
    assert_eq!(again.unchanged, 2);

    fs::remove_file(dir.path().join("a.mp3")).unwrap();
    let after = lib.scan(dir.path()).unwrap();
    assert_eq!(after.removed, 1);
    assert_eq!(lib.count().unwrap(), 1);
}

#[test]
fn a_movie_dropped_into_a_subfolder_after_the_first_scan_is_indexed() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("Films")).unwrap();
    fs::write(dir.path().join("Films/first.mkv"), b"first").unwrap();

    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();
    assert_eq!(lib.movies().unwrap().len(), 1);

    fs::create_dir(dir.path().join("Films/2024")).unwrap();
    fs::write(dir.path().join("Films/2024/Dropped.MKV"), b"dropped").unwrap();
    let stats = lib.scan(dir.path()).unwrap();

    assert_eq!(stats.indexed, 1);
    assert_eq!(stats.unchanged, 1);
    let movies = lib.movies().unwrap();
    assert_eq!(movies.len(), 2);
    assert!(movies.iter().all(|m| m.kind == MediaKind::Video));
}

#[test]
fn disc_and_raw_video_extensions_are_indexed_as_video() {
    let dir = tempfile::tempdir().unwrap();
    let names = [
        "a.m2ts", "b.MTS", "c.vob", "d.m2v", "e.3gp", "f.ogv", "g.rmvb", "h.obu",
    ];
    for name in names {
        fs::write(dir.path().join(name), b"x").unwrap();
    }

    let mut lib = Library::open_in_memory().unwrap();
    let stats = lib.scan(dir.path()).unwrap();

    assert_eq!(stats.indexed, names.len());
    let movies = lib.movies().unwrap();
    assert_eq!(movies.len(), names.len());
    assert!(movies.iter().all(|m| m.kind == MediaKind::Video));
}

#[test]
fn search_matches_prefix_and_ignores_empty_query() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("moonlight.mp3"), b"x").unwrap();
    fs::write(dir.path().join("sunrise.mp3"), b"x").unwrap();

    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();

    let hits = lib.search("moonli", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].path.ends_with("moonlight.mp3"));
    assert_eq!(hits[0].kind, MediaKind::Audio);
    assert!(lib.search("   ", 10).unwrap().is_empty());
}

#[test]
fn tags_are_normalised_and_removable() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("song.flac"), b"x").unwrap();

    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();
    let id = lib.list(10, 0).unwrap()[0].id;

    lib.add_tag(id, "  Chill ").unwrap();
    lib.add_tag(id, "chill").unwrap();
    assert_eq!(lib.tags_of(id).unwrap(), vec!["chill".to_string()]);

    lib.remove_tag(id, "CHILL").unwrap();
    assert!(lib.tags_of(id).unwrap().is_empty());
    assert!(lib.add_tag(id, "   ").is_err());
}
