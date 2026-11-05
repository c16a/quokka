use player_core::{FavoriteKind, Library};

#[test]
fn favourites_round_trip_and_stay_separate_by_kind() {
    let lib = Library::open_in_memory().unwrap();
    assert!(lib.favorites(FavoriteKind::Track).unwrap().is_empty());

    lib.set_favorite(FavoriteKind::Track, "/music/a.flac", true)
        .unwrap();
    // Marking twice is harmless.
    lib.set_favorite(FavoriteKind::Track, "/music/a.flac", true)
        .unwrap();
    lib.set_favorite(FavoriteKind::Album, "Red", true).unwrap();

    let tracks = lib.favorites(FavoriteKind::Track).unwrap();
    assert_eq!(tracks.len(), 1);
    assert!(tracks.contains("/music/a.flac"));
    assert!(lib.favorites(FavoriteKind::Album).unwrap().contains("Red"));
    // An album name is not a track path.
    assert!(!tracks.contains("Red"));

    lib.set_favorite(FavoriteKind::Track, "/music/a.flac", false)
        .unwrap();
    assert!(lib.favorites(FavoriteKind::Track).unwrap().is_empty());
}
