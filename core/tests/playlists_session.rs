use std::fs;
use std::path::Path;

use player_core::{Library, MediaItem, RepeatMode, Session};

/// A library scanned from a folder of placeholder audio files named `names`.
fn library_with(dir: &Path, names: &[&str]) -> (Library, Vec<MediaItem>) {
    for name in names {
        fs::write(dir.join(name), b"not really audio").unwrap();
    }
    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir).unwrap();
    let mut items = lib.list(100, 0).unwrap();
    items.sort_by(|a, b| a.path.cmp(&b.path));
    (lib, items)
}

fn ids(items: &[MediaItem]) -> Vec<i64> {
    items.iter().map(|i| i.id).collect()
}

#[test]
fn playlists_keep_order_and_repeated_tracks() {
    let dir = tempfile::tempdir().unwrap();
    let (lib, items) = library_with(dir.path(), &["a.mp3", "b.mp3"]);
    let (a, b) = (items[0].id, items[1].id);

    let id = lib.create_playlist("  Mix  ").unwrap();
    lib.add_to_playlist(id, &[a, b, a]).unwrap();
    assert_eq!(ids(&lib.playlist_tracks(id).unwrap()), vec![a, b, a]);

    let positions = lib.playlist_positions(id).unwrap();
    lib.remove_from_playlist(id, positions[1]).unwrap();
    assert_eq!(ids(&lib.playlist_tracks(id).unwrap()), vec![a, a]);

    let summary = lib.playlists().unwrap();
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].name, "Mix");
    assert_eq!(summary[0].track_count, 2);

    lib.rename_playlist(id, "Evening").unwrap();
    assert_eq!(lib.playlist_id("evening").unwrap(), Some(id));
    lib.delete_playlist(id).unwrap();
    assert!(lib.playlists().unwrap().is_empty());
    assert_eq!(
        lib.count().unwrap(),
        2,
        "deleting a playlist keeps its tracks"
    );
}

#[test]
fn a_track_removed_from_the_library_leaves_its_playlists() {
    let dir = tempfile::tempdir().unwrap();
    let (mut lib, items) = library_with(dir.path(), &["a.mp3", "b.mp3"]);
    let id = lib.create_playlist("Both").unwrap();
    lib.add_to_playlist(id, &[items[0].id, items[1].id])
        .unwrap();

    fs::remove_file(dir.path().join("b.mp3")).unwrap();
    lib.scan(dir.path()).unwrap();
    assert_eq!(ids(&lib.playlist_tracks(id).unwrap()), vec![items[0].id]);
}

#[test]
fn session_round_trips_and_missing_file_starts_fresh() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session.json");
    assert_eq!(Session::load(&path), None);

    let session = Session {
        queue: vec!["/music/a.flac".into(), "/music/b.flac".into()],
        index: 1,
        position_secs: 42.5,
        shuffle: true,
        repeat: RepeatMode::Queue,
    };
    session.save(&path).unwrap();
    assert_eq!(Session::load(&path), Some(session));
}

#[test]
fn repeat_cycles_off_queue_track() {
    assert_eq!(RepeatMode::Off.cycle(), RepeatMode::Queue);
    assert_eq!(RepeatMode::Queue.cycle(), RepeatMode::Track);
    assert_eq!(RepeatMode::Track.cycle(), RepeatMode::Off);
}
