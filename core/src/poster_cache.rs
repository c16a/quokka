//! Disk cache of TMDb posters, so a poster is downloaded once and not on every launch.

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use crate::tmdb::Tmdb;
use crate::{Error, Result};

/// The most the cache folder may hold. Older posters are removed after a new one is saved.
const CACHE_LIMIT_BYTES: u64 = 256 * 1024 * 1024;

/// The bytes of the TMDb image `path` at size `size`, read from `dir` when cached and otherwise
/// downloaded and saved there. `path` is a TMDb file path such as `/abc.jpg`.
pub fn load_or_fetch(dir: &Path, client: &Tmdb, size: &str, path: &str) -> Result<Vec<u8>> {
    let name = valid_name(path).ok_or_else(|| Error::Tmdb("invalid image path".into()))?;
    let file = dir.join(format!("{size}-{name}"));
    match std::fs::read(&file) {
        Ok(bytes) => {
            // Reading counts as use, so a poster that is shown often is not the first one pruned.
            touch(&file);
            return Ok(bytes);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let bytes = client.image_bytes(size, path)?;
    std::fs::create_dir_all(dir)?;
    // Write under a temporary name and rename, so an interrupted write never leaves a cache file
    // that is cut short. Each write gets its own temporary name, so two threads fetching the same
    // poster cannot interleave their bytes in one file.
    static NEXT_PART: AtomicU64 = AtomicU64::new(0);
    let serial = NEXT_PART.fetch_add(1, Ordering::Relaxed);
    let part = dir.join(format!(
        "{size}-{name}.{}.{serial}.part",
        std::process::id()
    ));
    std::fs::write(&part, &bytes)?;
    std::fs::rename(&part, &file)?;
    prune(dir, &file, CACHE_LIMIT_BYTES);
    Ok(bytes)
}

/// Mark a cached file as recently used. Failure only means it may be pruned sooner.
fn touch(file: &Path) {
    if let Ok(handle) = OpenOptions::new().write(true).open(file) {
        let _ = handle.set_modified(SystemTime::now());
    }
}

/// Remove the least recently modified posters until the folder holds at most `limit` bytes. The
/// file `keep` is never removed, and neither are partial downloads still being written. A file
/// that cannot be removed is left in place and counted in the total.
fn prune(dir: &Path, keep: &Path, limit: u64) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut total: u64 = 0;
    let mut candidates: Vec<(SystemTime, u64, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().ends_with(".part") {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        total += meta.len();
        let path = entry.path();
        if path != keep {
            let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            candidates.push((modified, meta.len(), path));
        }
    }
    if total <= limit {
        return;
    }
    candidates.sort_by_key(|(modified, _, _)| *modified);
    for (_, len, path) in candidates {
        if total <= limit {
            break;
        }
        if std::fs::remove_file(&path).is_ok() {
            total -= len;
        }
    }
}

/// The file name in a TMDb path, without its leading `/`. `None` when nothing is left, or when
/// the name would leave the cache directory: it has a separator or `..`.
fn valid_name(path: &str) -> Option<&str> {
    let name = path.strip_prefix('/').unwrap_or(path);
    if name.is_empty() || name.contains(['/', '\\']) || name.contains("..") {
        return None;
    }
    Some(name)
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    use super::{prune, valid_name};

    /// Write a file of `len` bytes whose modification time is `age` seconds after a fixed base.
    fn write_aged(dir: &std::path::Path, name: &str, len: usize, age: u64) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, vec![0u8; len]).unwrap();
        let base = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let handle = OpenOptions::new().write(true).open(&path).unwrap();
        handle
            .set_modified(base + Duration::from_secs(age))
            .unwrap();
        path
    }

    #[test]
    fn prune_removes_oldest_until_under_limit() {
        let dir = tempfile::tempdir().unwrap();
        let old = write_aged(dir.path(), "w500-old.jpg", 100, 0);
        let mid = write_aged(dir.path(), "w500-mid.jpg", 100, 1);
        let new = write_aged(dir.path(), "w500-new.jpg", 100, 2);
        let part = write_aged(dir.path(), "w500-x.jpg.1.2.part", 100, 0);

        prune(dir.path(), &new, 250);

        assert!(!old.exists());
        assert!(mid.exists());
        assert!(new.exists());
        assert!(part.exists());
    }

    #[test]
    fn prune_never_removes_the_kept_file() {
        let dir = tempfile::tempdir().unwrap();
        let keep = write_aged(dir.path(), "w500-keep.jpg", 100, 0);
        let mid = write_aged(dir.path(), "w500-mid.jpg", 100, 1);
        let new = write_aged(dir.path(), "w500-new.jpg", 100, 2);

        prune(dir.path(), &keep, 150);

        assert!(keep.exists());
        assert!(!mid.exists());
        assert!(!new.exists());
    }

    #[test]
    fn accepts_a_plain_name_with_leading_slash() {
        assert_eq!(valid_name("/abc.jpg"), Some("abc.jpg"));
    }

    #[test]
    fn rejects_a_leading_slash_alone() {
        assert_eq!(valid_name("/"), None);
        assert_eq!(valid_name(""), None);
    }

    #[test]
    fn rejects_parent_directory_references() {
        assert_eq!(valid_name("/../secret.jpg"), None);
        assert_eq!(valid_name("/a..b.jpg"), None);
    }

    #[test]
    fn rejects_nested_paths() {
        assert_eq!(valid_name("/a/b.jpg"), None);
        assert_eq!(valid_name("/a\\b.jpg"), None);
    }
}
