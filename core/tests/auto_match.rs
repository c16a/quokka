use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};

use quokka_core::{Error, Library};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn a_pending_lookup_is_tried_once_until_the_file_changes() {
    let dir = tempfile::tempdir().unwrap();
    let movie = dir.path().join("The.Matrix.1999.mp4");
    fs::copy(fixture("av1_8bit.mp4"), &movie).unwrap();
    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();

    // A new file is pending, so the lookup runs for it once.
    let calls = Cell::new(0);
    let saved = lib
        .match_pending(|_, _| {
            calls.set(calls.get() + 1);
            Ok(false)
        })
        .unwrap();
    assert_eq!((saved, calls.get()), (0, 1));

    // Scanning again with nothing changed leaves nothing pending.
    lib.scan(dir.path()).unwrap();
    lib.match_pending(|_, _| {
        calls.set(calls.get() + 1);
        Ok(false)
    })
    .unwrap();
    assert_eq!(calls.get(), 1);

    // A changed file is pending again.
    fs::write(&movie, b"a different length, so the scan sees a change").unwrap();
    lib.scan(dir.path()).unwrap();
    lib.match_pending(|_, _| {
        calls.set(calls.get() + 1);
        Ok(false)
    })
    .unwrap();
    assert_eq!(calls.get(), 2);
}

#[test]
fn a_failed_lookup_stays_pending_for_the_next_scan() {
    let dir = tempfile::tempdir().unwrap();
    fs::copy(fixture("av1_8bit.mp4"), dir.path().join("Heat.1995.mp4")).unwrap();
    let mut lib = Library::open_in_memory().unwrap();
    lib.scan(dir.path()).unwrap();

    let failed = lib.match_pending(|_, _| Err(Error::Tmdb("offline".into())));
    assert!(failed.is_err());

    // The lookup that failed is still pending, so it runs again.
    let calls = Cell::new(0);
    lib.match_pending(|_, _| {
        calls.set(calls.get() + 1);
        Ok(true)
    })
    .unwrap();
    assert_eq!(calls.get(), 1);
}
