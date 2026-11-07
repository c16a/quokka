//! Saved positions: they are kept per file, overwritten by the next save, and cleared on request.

use std::path::Path;

use player_core::{Library, Progress, finished};

#[test]
fn a_saved_position_is_kept_and_overwritten() {
    let library = Library::open_in_memory().expect("open");
    let path = Path::new("/movies/a.mkv");
    assert_eq!(library.progress(path).expect("read"), None);

    library
        .save_progress(path, 16.1, Some(6000.0))
        .expect("save");
    let saved: Progress = library.progress(path).expect("read").expect("saved");
    assert_eq!(saved.position_secs, 16.1);
    assert_eq!(saved.duration_secs, Some(6000.0));

    library.save_progress(path, 42.0, None).expect("save again");
    let saved = library.progress(path).expect("read").expect("saved");
    assert_eq!(saved.position_secs, 42.0);
    assert_eq!(saved.duration_secs, None);
}

#[test]
fn clearing_forgets_the_position() {
    let library = Library::open_in_memory().expect("open");
    let path = Path::new("/movies/b.mkv");
    library
        .save_progress(path, 90.0, Some(6000.0))
        .expect("save");
    library.clear_progress(path).expect("clear");
    assert_eq!(library.progress(path).expect("read"), None);
}

#[test]
fn positions_are_kept_per_file() {
    let library = Library::open_in_memory().expect("open");
    library
        .save_progress(Path::new("/a.mp3"), 10.0, None)
        .expect("save");
    library
        .save_progress(Path::new("/b.mp3"), 20.0, None)
        .expect("save");
    assert_eq!(
        library
            .progress(Path::new("/a.mp3"))
            .expect("read")
            .unwrap()
            .position_secs,
        10.0
    );
    assert_eq!(
        library
            .progress(Path::new("/b.mp3"))
            .expect("read")
            .unwrap()
            .position_secs,
        20.0
    );
}

#[test]
fn finished_means_near_the_end() {
    assert!(finished(6000.0, Some(6000.0)));
    assert!(!finished(16.0, Some(6000.0)));
}
