//! Library layer: SQLite index of audio and video files, scanning, metadata,
//! full-text search and tags. Pure Rust, no UI and no playback dependencies.

mod artist_photo;
mod browse;
mod cover;

pub use artist_photo::{ArtistPhoto, fetch_artist_photo};
pub use cover::read_cover;
mod db;
mod details;
mod favorites;
mod history;
mod lyrics;
mod metadata;
mod playlist;
mod radio;
mod replaygain;
mod scan;
mod search;
mod session;
mod settings;
mod tag_write;

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

pub use browse::{AlbumFilter, AlbumSort, AlbumSummary};
pub use details::{TagLine, TrackDetails, track_details};
pub use favorites::FavoriteKind;
pub use history::Recent;
pub use lyrics::{
    LyricLine, Lyrics, LyricsQuery, RemoteLyrics, best_lyrics, fetch_lyrics, lyrics_from_text,
    parse_lrc, sidecar_lyrics,
};
pub use playlist::PlaylistSummary;
pub use radio::{Station, fetch_all_stations, parse_stations};
pub use replaygain::{loudness_factor, replay_gain_factor, tag_gain};
pub use scan::ScanStats;
pub use search::SongQuery;
pub use session::{RepeatMode, Session};
pub use settings::{ReplayGainMode, Settings, Theme};

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
    #[error("the file's tags changed since they were read")]
    TagsChanged,
    #[error("the tag {0} can't be edited here")]
    TagNotEditable(String),
    #[error("could not write the tags: {0}")]
    Tag(String),
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
            "mp3" | "flac" | "m4a" | "aac" | "ogg" | "oga" | "opus" | "wav" | "aif" | "aiff"
            | "mka" => Some(Self::Audio),
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

    /// Remove every file under `root` from the library. Nothing on disk is touched.
    pub fn forget_root(&self, root: &Path) -> Result<usize> {
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let prefix = root.to_string_lossy().into_owned();
        let upper_bound = format!("{prefix}\u{10FFFF}");
        Ok(self.conn.execute(
            "DELETE FROM media WHERE path >= ?1 AND path < ?2",
            params![prefix, upper_bound],
        )?)
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

    /// Look up one item by its file path.
    pub fn find_path(&self, path: &Path) -> Result<Option<MediaItem>> {
        let sql = format!("SELECT {COLUMNS} FROM media WHERE path = ?1");
        let key = path.to_string_lossy();
        Ok(self
            .conn
            .query_row(&sql, [key.as_ref()], row_to_item)
            .optional()?)
    }

    /// Look up one item by id.
    pub fn get(&self, id: i64) -> Result<Option<MediaItem>> {
        let sql = format!("SELECT {COLUMNS} FROM media WHERE id = ?1");
        Ok(self.conn.query_row(&sql, [id], row_to_item).optional()?)
    }

    /// The measured loudness of the file at `path`, as `(LUFS, peak)`. `None` when it hasn't been
    /// measured, or the file has changed since.
    pub fn loudness(&self, path: &Path) -> Result<Option<(f64, f64)>> {
        let key = path.to_string_lossy().into_owned();
        let row: Option<(f64, f64, i64)> = self
            .conn
            .query_row(
                "SELECT lufs, peak, mtime FROM loudness WHERE path = ?1",
                [&key],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        Ok(row.and_then(|(lufs, peak, mtime)| {
            (file_mtime(path) == Some(mtime)).then_some((lufs, peak))
        }))
    }

    /// Store the measured loudness of the file at `path`, as of its current modification time.
    pub fn set_loudness(&self, path: &Path, lufs: f64, peak: f64) -> Result<()> {
        let key = path.to_string_lossy().into_owned();
        let mtime = file_mtime(path).unwrap_or(0);
        self.conn.execute(
            "INSERT INTO loudness(path, lufs, peak, mtime) VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(path) DO UPDATE SET lufs = excluded.lufs, peak = excluded.peak, \
             mtime = excluded.mtime",
            params![key, lufs, peak, mtime],
        )?;
        Ok(())
    }

    /// Write new text tags into the file at `path`. `expected` is the tag lines as they were when
    /// the user started editing, and `values` is the new value of each. Before the first edit of a
    /// file, its tags are kept, so `revert_tags` can put them back.
    pub fn save_tags(&self, path: &Path, expected: &[TagLine], values: &[String]) -> Result<()> {
        let key = path.to_string_lossy().into_owned();
        if !self.has_tag_snapshot(path)? {
            let json = serde_json::to_string(expected)?;
            self.conn.execute(
                "INSERT INTO tag_snapshot(path, tags) VALUES (?1, ?2)",
                params![key, json],
            )?;
        }
        tag_write::write_tags(path, expected, values)?;
        self.refresh_media(path)
    }

    /// Put back the tags the file had before its first edit. Returns `false` when there is nothing
    /// to put back.
    pub fn revert_tags(&self, path: &Path) -> Result<bool> {
        let key = path.to_string_lossy().into_owned();
        let json: Option<String> = self
            .conn
            .query_row(
                "SELECT tags FROM tag_snapshot WHERE path = ?1",
                [&key],
                |row| row.get(0),
            )
            .optional()?;
        let Some(json) = json else {
            return Ok(false);
        };
        let original: Vec<TagLine> = serde_json::from_str(&json)?;
        let values: Vec<String> = original.iter().map(|line| line.value.clone()).collect();
        let current = details::track_details(path).ok_or(Error::TagsChanged)?.tags;
        tag_write::write_tags(path, &current, &values)?;
        self.conn
            .execute("DELETE FROM tag_snapshot WHERE path = ?1", [&key])?;
        self.refresh_media(path)?;
        Ok(true)
    }

    /// Whether the file at `path` has tags to revert to.
    pub fn has_tag_snapshot(&self, path: &Path) -> Result<bool> {
        let key = path.to_string_lossy().into_owned();
        let found: Option<i64> = self
            .conn
            .query_row(
                "SELECT 1 FROM tag_snapshot WHERE path = ?1",
                [&key],
                |row| row.get(0),
            )
            .optional()?;
        Ok(found.is_some())
    }

    /// Re-read a file's tags into its row, so the library shows what was just written.
    fn refresh_media(&self, path: &Path) -> Result<()> {
        let key = path.to_string_lossy().into_owned();
        let meta = std::fs::metadata(path)?;
        let size = meta.len() as i64;
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let info = metadata::read(path, size);
        self.conn.execute(
            "UPDATE media SET size = ?2, mtime = ?3, duration_ms = ?4, title = ?5, artist = ?6, \
               album = ?7, album_artist = ?8, genre = ?9, composer = ?10, track = ?11, year = ?12 \
             WHERE path = ?1",
            params![
                key,
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
                info.year
            ],
        )?;
        Ok(())
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

/// The modification time of the file at `path`, in whole seconds since the epoch.
fn file_mtime(path: &Path) -> Option<i64> {
    std::fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}
