use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rusqlite::{Connection, params};
use walkdir::{DirEntry, WalkDir};

use crate::{Error, MediaKind, Result, metadata};

/// Outcome counters for one scan.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScanStats {
    /// Files newly added or re-read because size or mtime changed.
    pub indexed: usize,
    /// Files whose size and mtime matched the index, so they were not re-read.
    pub unchanged: usize,
    /// Index rows removed because the file no longer exists under the root.
    pub removed: usize,
    /// Entries that could not be read (permissions, broken symlinks, ...).
    pub errors: usize,
}

struct Known {
    id: i64,
    size: i64,
    mtime: i64,
}

pub(crate) fn scan(conn: &mut Connection, root: &Path) -> Result<ScanStats> {
    if !root.is_dir() {
        return Err(Error::NotADirectory(root.to_path_buf()));
    }
    let root = root.canonicalize()?;
    let root_prefix = root.to_string_lossy().into_owned();

    let tx = conn.transaction()?;
    let mut stats = ScanStats::default();

    // Everything we already know about under this root.
    let mut known: HashMap<String, Known> = HashMap::new();
    {
        let mut stmt = tx.prepare(
            "SELECT id, path, size, mtime FROM media \
             WHERE substr(path, 1, length(?1)) = ?1",
        )?;
        let rows = stmt.query_map([&root_prefix], |r| {
            Ok((
                r.get::<_, String>(1)?,
                Known {
                    id: r.get(0)?,
                    size: r.get(2)?,
                    mtime: r.get(3)?,
                },
            ))
        })?;
        for row in rows {
            let (path, k) = row?;
            known.insert(path, k);
        }
    }

    let mut seen: HashSet<PathBuf> = HashSet::new();
    let walker = WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_hidden(e));

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                stats.errors += 1;
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Some(kind) = MediaKind::from_path(path) else {
            continue;
        };
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => {
                stats.errors += 1;
                continue;
            }
        };
        let size = meta.len() as i64;
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let key = path.to_string_lossy().into_owned();
        seen.insert(path.to_path_buf());

        if let Some(k) = known.get(&key)
            && k.size == size
            && k.mtime == mtime
        {
            stats.unchanged += 1;
            continue;
        }

        let info = metadata::read(path);
        tx.execute(
            "INSERT INTO media \
               (path, kind, size, mtime, duration_ms, title, artist, album, album_artist, \
                genre, composer, track, year) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13) \
             ON CONFLICT(path) DO UPDATE SET \
               kind = excluded.kind, size = excluded.size, mtime = excluded.mtime, \
               duration_ms = excluded.duration_ms, title = excluded.title, \
               artist = excluded.artist, album = excluded.album, \
               album_artist = excluded.album_artist, genre = excluded.genre, \
               composer = excluded.composer, track = excluded.track, year = excluded.year",
            params![
                key,
                kind.as_str(),
                size,
                mtime,
                info.duration_ms,
                info.title,
                info.artist,
                info.album,
                info.album_artist,
                info.genre,
                info.composer,
                info.track,
                info.year,
            ],
        )?;
        stats.indexed += 1;
    }

    // Anything we knew about that was not seen this time has been deleted or moved.
    for (path, k) in &known {
        if !seen.contains(Path::new(path)) {
            tx.execute("DELETE FROM media WHERE id = ?1", [k.id])?;
            stats.removed += 1;
        }
    }

    tx.commit()?;
    Ok(stats)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.depth() > 0 && entry.file_name().to_string_lossy().starts_with('.')
}
