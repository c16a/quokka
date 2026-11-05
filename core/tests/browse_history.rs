use std::fs;
use std::path::Path;

use player_core::{AlbumFilter, AlbumSort, Library, Recent, SongQuery};

/// A library scanned from placeholder files, which carry no tags.
fn library_with(dir: &Path, names: &[&str]) -> Library {
    for name in names {
        fs::write(dir.join(name), b"not really audio").unwrap();
    }
    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir).unwrap();
    lib
}

#[test]
fn recent_lists_follow_additions_and_plays() {
    let dir = tempfile::tempdir().unwrap();
    let lib = library_with(dir.path(), &["a.mp3", "b.mp3"]);
    // Untagged files all belong to the album named "".
    assert_eq!(
        lib.recent_albums(Recent::Added, 10).unwrap(),
        vec![String::new()]
    );
    assert!(lib.recent_albums(Recent::Played, 10).unwrap().is_empty());

    let track = lib.list(1, 0).unwrap()[0].id;
    lib.record_play(track).unwrap();
    assert_eq!(
        lib.recent_albums(Recent::Played, 10).unwrap(),
        vec![String::new()]
    );
}

#[test]
fn album_sorting_and_filters_run_without_tags() {
    let dir = tempfile::tempdir().unwrap();
    let lib = library_with(dir.path(), &["a.mp3"]);
    for sort in [AlbumSort::Name, AlbumSort::Year, AlbumSort::Added] {
        assert_eq!(
            lib.albums_by(sort, &AlbumFilter::default()).unwrap().len(),
            1
        );
    }
    let by_year = AlbumFilter {
        genre: None,
        year: Some(2005),
    };
    assert!(lib.albums_by(AlbumSort::Name, &by_year).unwrap().is_empty());
    assert!(lib.genres().unwrap().is_empty());
    assert!(lib.years().unwrap().is_empty());
    let untagged = lib
        .search_songs(&SongQuery::parse("format:mp3"), 50)
        .unwrap();
    assert!(
        untagged.is_empty(),
        "no codec is read from placeholder files"
    );
}
