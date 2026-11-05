use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rayon::prelude::*;
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

/// A file that needs its tags read.
struct Pending {
    path: PathBuf,
    key: String,
    kind: MediaKind,
    size: i64,
    mtime: i64,
}

pub(crate) fn scan(conn: &mut Connection, root: &Path) -> Result<ScanStats> {
    if !root.is_dir() {
        return Err(Error::NotADirectory(root.to_path_buf()));
    }
    let root = root.canonicalize()?;
    let root_prefix = root.to_string_lossy().into_owned();
    // Every path that starts with `root_prefix` sorts between it and this bound,
    // so the known-file lookup is a range query that can use the unique path index.
    let upper_bound = format!("{root_prefix}\u{10FFFF}");

    let tx = conn.transaction()?;
    let mut stats = ScanStats::default();

    let mut known: HashMap<String, Known> = HashMap::new();
    {
        let mut stmt = tx.prepare_cached(
            "SELECT id, path, size, mtime FROM media WHERE path >= ?1 AND path < ?2",
        )?;
        let rows = stmt.query_map(params![root_prefix, upper_bound], |r| {
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

    // Pass 1 (sequential): walk the tree, skip unchanged files, and collect the rest.
    let mut seen: HashSet<String> = HashSet::new();
    let mut pending: Vec<Pending> = Vec::new();
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
        seen.insert(key.clone());

        if let Some(k) = known.get(&key)
            && k.size == size
            && k.mtime == mtime
        {
            stats.unchanged += 1;
            continue;
        }
        pending.push(Pending {
            path: path.to_path_buf(),
            key,
            kind,
            size,
            mtime,
        });
    }

    // Pass 2 (parallel): read tags for the files that changed. Order is preserved.
    let read: Vec<(Pending, metadata::Metadata)> = pending
        .into_par_iter()
        .map(|p| {
            let info = metadata::read(&p.path, p.size);
            (p, info)
        })
        .collect();

    // Pass 3 (sequential): write the results in one transaction, reusing statements.
    {
        let mut upsert = tx.prepare_cached(
            "INSERT INTO media \
               (path, kind, size, mtime, duration_ms, title, artist, album, album_artist, \
                genre, composer, track, year, codec, bit_depth, sample_rate, channels, \
                bitrate_kbps) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18) \
             ON CONFLICT(path) DO UPDATE SET \
               kind = excluded.kind, size = excluded.size, mtime = excluded.mtime, \
               duration_ms = excluded.duration_ms, title = excluded.title, \
               artist = excluded.artist, album = excluded.album, \
               album_artist = excluded.album_artist, genre = excluded.genre, \
               composer = excluded.composer, track = excluded.track, year = excluded.year, \
               codec = excluded.codec, bit_depth = excluded.bit_depth, \
               sample_rate = excluded.sample_rate, channels = excluded.channels, \
               bitrate_kbps = excluded.bitrate_kbps",
        )?;
        for (p, info) in &read {
            upsert.execute(params![
                p.key,
                p.kind.as_str(),
                p.size,
                p.mtime,
                info.duration_ms,
                info.title,
                info.artist,
                info.album,
                info.album_artist,
                info.genre,
                info.composer,
                info.track,
                info.year,
                info.codec,
                info.bit_depth,
                info.sample_rate,
                info.channels,
                info.bitrate_kbps,
            ])?;
            stats.indexed += 1;
        }
    }

    // Anything we knew about that was not seen this time has been deleted or moved.
    {
        let mut delete = tx.prepare_cached("DELETE FROM media WHERE id = ?1")?;
        for (path, k) in &known {
            if !seen.contains(path) {
                delete.execute([k.id])?;
                stats.removed += 1;
            }
        }
    }

    tx.commit()?;
    Ok(stats)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.depth() > 0 && entry.file_name().to_string_lossy().starts_with('.')
}
