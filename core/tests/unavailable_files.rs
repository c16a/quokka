//! Files under a folder the scanner cannot read stay in the library, marked unavailable, and still
//! turn up in search. They are removed only when a readable folder no longer holds them.
//! Unix only: the test makes a folder unreadable with Unix permission bits, which Windows does not have.

#![cfg(unix)]

use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use quokka_core::Library;
use tempfile::TempDir;

/// Puts a folder's permissions back when dropped, so a failed assertion still lets the temporary
/// folder be removed.
struct Restore(PathBuf);

impl Drop for Restore {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.0, Permissions::from_mode(0o755));
    }
}

fn set_mode(path: &Path, mode: u32) {
    fs::set_permissions(path, Permissions::from_mode(mode)).unwrap();
}

#[test]
fn files_in_an_unreadable_folder_are_kept_marked_and_searchable() {
    let music = TempDir::new().unwrap();
    let db_dir = TempDir::new().unwrap();
    let root = music.path().canonicalize().unwrap();
    let locked = root.join("locked");
    fs::create_dir(root.join("open")).unwrap();
    fs::create_dir(&locked).unwrap();
    fs::write(root.join("open/song-alpha.mp3"), b"not audio").unwrap();
    fs::write(locked.join("zebra.mp3"), b"not audio").unwrap();
    fs::write(locked.join("clip-zebra.mkv"), b"not video").unwrap();

    let mut library = Library::open(&db_dir.path().join("library.db")).unwrap();
    library.scan(&root).unwrap();
    assert_eq!(library.count().unwrap(), 3);

    set_mode(&locked, 0o000);
    let _restore = Restore(locked.clone());

    let stats = library.scan(&root).unwrap();
    assert_eq!(
        stats.removed, 0,
        "rows under an unreadable folder must not be deleted"
    );
    assert_eq!(stats.unavailable, 2);
    assert_eq!(library.count().unwrap(), 3);

    let hits = library.search("zebra", 10).unwrap();
    assert_eq!(
        hits.len(),
        2,
        "unavailable audio and video both match search"
    );
    assert!(hits.iter().all(|item| item.unavailable));
    assert!(
        hits.iter()
            .any(|item| item.kind == quokka_core::MediaKind::Audio)
    );
    assert!(
        hits.iter()
            .any(|item| item.kind == quokka_core::MediaKind::Video)
    );

    let alpha = library
        .find_path(&root.join("open/song-alpha.mp3"))
        .unwrap()
        .unwrap();
    assert!(!alpha.unavailable);

    set_mode(&locked, 0o755);
    let stats = library.scan(&root).unwrap();
    assert_eq!(stats.unavailable, 0);
    let hits = library.search("zebra", 10).unwrap();
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().all(|item| !item.unavailable));

    fs::remove_file(locked.join("zebra.mp3")).unwrap();
    let stats = library.scan(&root).unwrap();
    assert_eq!(
        stats.removed, 1,
        "a readable folder that lost a file still drops its row"
    );
    assert_eq!(library.count().unwrap(), 2);
}
