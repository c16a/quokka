//! Library layer: SQLite index of audio and video files, scanning, metadata,
//! full-text search and tags. Pure Rust, no UI and no playback dependencies.

mod browse;
mod db;
mod metadata;
mod scan;
mod settings;

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

pub use browse::AlbumSummary;
pub use scan::ScanStats;
pub use settings::{Settings, Theme};

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("settings file is not valid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a directory: {0}")]
    NotADirectory(PathBuf),
    #[error("invalid tag name")]
    InvalidTag,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Audio,
    Video,
}

impl MediaKind {
    /// Classify a path by extension. `None` means "not a media file we index".
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "mp3" | "flac" | "m4a" | "aac" | "ogg" | "oga" | "opus" | "wav" | "aif" | "aiff" => {
                Some(Self::Audio)
            }
            "mp4" | "m4v" | "mkv" | "webm" | "mov" | "avi" | "wmv" | "flv" | "ts" | "mpg"
            | "mpeg" => Some(Self::Video),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Audio => "audio",
            Self::Video => "video",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "audio" => Some(Self::Audio),
            "video" => Some(Self::Video),
            _ => None,
        }
    }
}

/// One indexed file as the UI sees it.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaItem {
    pub id: i64,
    pub path: PathBuf,
    pub kind: MediaKind,
    pub size: i64,
    pub duration_ms: Option<i64>,
    pub title: Option<String>,
    /// Track artist(s).
    pub artist: Option<String>,
    pub album: Option<String>,
    /// Album artist, if tagged.
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub genre: Option<String>,
    /// Codec label, such as "FLAC".
    pub codec: Option<String>,
    pub bit_depth: Option<i64>,
    pub sample_rate: Option<i64>,
    pub channels: Option<i64>,
    pub bitrate_kbps: Option<i64>,
    pub track: Option<i64>,
    pub year: Option<i64>,
}

/// Handle to the library database.
pub struct Library {
    conn: Connection,
}

impl Library {
    /// Open (or create) a library database file.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        db::configure(&conn)?;
        Ok(Self { conn })
    }

    /// In-memory library, mainly for tests.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        db::configure(&conn)?;
        Ok(Self { conn })
    }

    /// Scan `root` recursively and bring the index up to date.
    /// Files whose size and mtime are unchanged are skipped; rows for files
    /// that no longer exist under `root` are removed.
    pub fn scan(&mut self, root: &Path) -> Result<ScanStats> {
        scan::scan(&mut self.conn, root)
    }

    /// Number of indexed files.
    pub fn count(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM media", [], |r| r.get(0))?)
    }

    /// A page of items in library order, for virtualised lists.
    pub fn list(&self, limit: usize, offset: usize) -> Result<Vec<MediaItem>> {
        let sql = format!(
            "SELECT {COLUMNS} FROM media \
             ORDER BY artist IS NULL, artist, album, track, path \
             LIMIT ?1 OFFSET ?2"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![limit as i64, offset as i64], row_to_item)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Full-text search over title, artist, album and path. Every whitespace
    /// separated word must match, and the last word also matches as a prefix,
    /// so results appear while typing.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<MediaItem>> {
        let Some(fts) = fts_query(query) else {
            return Ok(Vec::new());
        };
        let sql = format!(
            "SELECT {M_COLUMNS} FROM media_fts \
             JOIN media m ON m.id = media_fts.rowid \
             WHERE media_fts MATCH ?1 \
             ORDER BY rank \
             LIMIT ?2"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![fts, limit as i64], row_to_item)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Look up one item by id.
    pub fn get(&self, id: i64) -> Result<Option<MediaItem>> {
        let sql = format!("SELECT {COLUMNS} FROM media WHERE id = ?1");
        Ok(self.conn.query_row(&sql, [id], row_to_item).optional()?)
    }

    /// Attach a tag to an item. Names are trimmed and lower-cased.
    pub fn add_tag(&mut self, media_id: i64, name: &str) -> Result<()> {
        let name = normalise_tag(name)?;
        let tx = self.conn.transaction()?;
        tx.execute("INSERT OR IGNORE INTO tag(name) VALUES (?1)", [&name])?;
        tx.execute(
            "INSERT OR IGNORE INTO media_tag(media_id, tag_id) \
             SELECT ?1, id FROM tag WHERE name = ?2",
            params![media_id, name],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Detach a tag from an item. Does nothing if it was not attached.
    pub fn remove_tag(&mut self, media_id: i64, name: &str) -> Result<()> {
        let name = normalise_tag(name)?;
        self.conn.execute(
            "DELETE FROM media_tag WHERE media_id = ?1 \
             AND tag_id = (SELECT id FROM tag WHERE name = ?2)",
            params![media_id, name],
        )?;
        Ok(())
    }

    /// Tags attached to an item, sorted by name.
    pub fn tags_of(&self, media_id: i64) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name FROM tag t JOIN media_tag mt ON mt.tag_id = t.id \
             WHERE mt.media_id = ?1 ORDER BY t.name",
        )?;
        let rows = stmt.query_map([media_id], |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

const COLUMNS: &str = "id, path, kind, size, duration_ms, title, artist, album, genre, track, year, album_artist, composer, codec, bit_depth, sample_rate, channels, bitrate_kbps";
const M_COLUMNS: &str = "m.id, m.path, m.kind, m.size, m.duration_ms, m.title, m.artist, m.album, m.genre, m.track, m.year, m.album_artist, m.composer, m.codec, m.bit_depth, m.sample_rate, m.channels, m.bitrate_kbps";

pub(crate) fn row_to_item(r: &rusqlite::Row<'_>) -> rusqlite::Result<MediaItem> {
    let kind: String = r.get(2)?;
    Ok(MediaItem {
        id: r.get(0)?,
        path: PathBuf::from(r.get::<_, String>(1)?),
        kind: MediaKind::parse(&kind).unwrap_or(MediaKind::Video),
        size: r.get(3)?,
        duration_ms: r.get(4)?,
        title: r.get(5)?,
        artist: r.get(6)?,
        album: r.get(7)?,
        genre: r.get(8)?,
        track: r.get(9)?,
        year: r.get(10)?,
        album_artist: r.get(11)?,
        composer: r.get(12)?,
        codec: r.get(13)?,
        bit_depth: r.get(14)?,
        sample_rate: r.get(15)?,
        channels: r.get(16)?,
        bitrate_kbps: r.get(17)?,
    })
}

/// Turn free text into a safe FTS5 query: each word quoted, the last one a prefix match.
fn fts_query(input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }
    let last = words.len() - 1;
    let parts: Vec<String> = words
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let quoted = format!("\"{}\"", w.replace('"', "\"\""));
            if i == last {
                format!("{quoted} *")
            } else {
                quoted
            }
        })
        .collect();
    Some(parts.join(" "))
}

fn normalise_tag(name: &str) -> Result<String> {
    let name = name.trim().to_lowercase();
    if name.is_empty() {
        Err(Error::InvalidTag)
    } else {
        Ok(name)
    }
}
