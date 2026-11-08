//! Disk cache of TMDb posters, so a poster is downloaded once and not on every launch.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::tmdb::Tmdb;
use crate::{Error, Result};

/// The bytes of the TMDb image `path` at size `size`, read from `dir` when cached and otherwise
/// downloaded and saved there. `path` is a TMDb file path such as `/abc.jpg`.
pub fn load_or_fetch(dir: &Path, client: &Tmdb, size: &str, path: &str) -> Result<Vec<u8>> {
    let name = valid_name(path).ok_or_else(|| Error::Tmdb("invalid image path".into()))?;
    let file = dir.join(format!("{size}-{name}"));
    match std::fs::read(&file) {
        Ok(bytes) => return Ok(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let bytes = client.image_bytes(size, path)?;
    std::fs::create_dir_all(dir)?;
    // Write under a temporary name and rename, so an interrupted write never leaves a cache file
    // that is cut short.
    // Each write gets its own temporary name, so two threads fetching the same poster cannot
    // interleave their bytes in one file.
    static NEXT_PART: AtomicU64 = AtomicU64::new(0);
    let serial = NEXT_PART.fetch_add(1, Ordering::Relaxed);
    let part = dir.join(format!(
        "{size}-{name}.{}.{serial}.part",
        std::process::id()
    ));
    std::fs::write(&part, &bytes)?;
    std::fs::rename(&part, &file)?;
    Ok(bytes)
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
    use super::valid_name;

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
