use rusqlite::Connection;

use crate::Result;

/// Bump when the schema changes and add a matching migration step below.
const SCHEMA_VERSION: i64 = 21;

/// Per-connection settings and schema migration.
pub(crate) fn configure(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
    // The loudness measurer writes from its own thread, so wait for a busy database.
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    // WAL is not available for in-memory databases; ignore the result there.
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    // With WAL, NORMAL keeps the database consistent after a crash and commits much faster.
    conn.pragma_update(None, "synchronous", "NORMAL")?;

    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version < 1 {
        conn.execute_batch(SCHEMA_V1)?;
    }
    if version < 2 {
        conn.execute_batch(SCHEMA_V2)?;
    }
    if version < 3 {
        conn.execute_batch(SCHEMA_V3)?;
    }
    if version < 4 {
        conn.execute_batch(SCHEMA_V4)?;
    }
    if version < 5 {
        conn.execute_batch(SCHEMA_V5)?;
    }
    if version < 6 {
        conn.execute_batch(SCHEMA_V6)?;
    }
    if version < 7 {
        conn.execute_batch(SCHEMA_V7)?;
    }
    if version < 8 {
        conn.execute_batch(SCHEMA_V8)?;
    }
    if version < 9 {
        conn.execute_batch(SCHEMA_V9)?;
    }
    if version < 10 {
        conn.execute_batch(SCHEMA_V10)?;
    }
    if version < 11 {
        conn.execute_batch(SCHEMA_V11)?;
    }
    if version < 12 {
        conn.execute_batch(SCHEMA_V12)?;
    }
    if version < 13 {
        conn.execute_batch(SCHEMA_V13)?;
    }
    if version < 14 {
        conn.execute_batch(SCHEMA_V14)?;
    }
    if version < 15 {
        conn.execute_batch(SCHEMA_V15)?;
    }
    if version < 16 {
        conn.execute_batch(SCHEMA_V16)?;
    }
    if version < 17 {
        conn.execute_batch(SCHEMA_V17)?;
    }
    if version < 18 {
        conn.execute_batch(SCHEMA_V18)?;
    }
    if version < 19 {
        conn.execute_batch(SCHEMA_V19)?;
    }
    if version < 20 {
        conn.execute_batch(SCHEMA_V20)?;
    }
    if version < 21 {
        conn.execute_batch(SCHEMA_V21)?;
    }
    if version != SCHEMA_VERSION {
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    }
    Ok(())
}

const SCHEMA_V1: &str = "
CREATE TABLE media (
    id          INTEGER PRIMARY KEY,
    path        TEXT NOT NULL UNIQUE,
    kind        TEXT NOT NULL CHECK (kind IN ('audio', 'video')),
    size        INTEGER NOT NULL,
    mtime       INTEGER NOT NULL,
    duration_ms INTEGER,
    title       TEXT,
    artist      TEXT,
    album       TEXT,
    genre       TEXT,
    track       INTEGER,
    year        INTEGER
);

CREATE TABLE tag (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE media_tag (
    media_id INTEGER NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    tag_id   INTEGER NOT NULL REFERENCES tag(id) ON DELETE CASCADE,
    PRIMARY KEY (media_id, tag_id)
);
CREATE INDEX media_tag_by_tag ON media_tag(tag_id);

CREATE VIRTUAL TABLE media_fts USING fts5(
    title, artist, album, path,
    content = 'media',
    content_rowid = 'id',
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TRIGGER media_ai AFTER INSERT ON media BEGIN
    INSERT INTO media_fts(rowid, title, artist, album, path)
    VALUES (new.id, new.title, new.artist, new.album, new.path);
END;

CREATE TRIGGER media_ad AFTER DELETE ON media BEGIN
    INSERT INTO media_fts(media_fts, rowid, title, artist, album, path)
    VALUES ('delete', old.id, old.title, old.artist, old.album, old.path);
END;

CREATE TRIGGER media_au AFTER UPDATE ON media BEGIN
    INSERT INTO media_fts(media_fts, rowid, title, artist, album, path)
    VALUES ('delete', old.id, old.title, old.artist, old.album, old.path);
    INSERT INTO media_fts(rowid, title, artist, album, path)
    VALUES (new.id, new.title, new.artist, new.album, new.path);
END;
";

/// Album artist and composer columns. Existing rows get a stale mtime so the next
/// scan re-reads every file and fills the new columns.
const SCHEMA_V2: &str = "
ALTER TABLE media ADD COLUMN album_artist TEXT;
ALTER TABLE media ADD COLUMN composer TEXT;
UPDATE media SET mtime = -1;
";

/// Stream format columns. Existing rows get a stale mtime so the next scan
/// re-reads every file and fills them in.
const SCHEMA_V3: &str = "
ALTER TABLE media ADD COLUMN codec TEXT;
ALTER TABLE media ADD COLUMN bit_depth INTEGER;
ALTER TABLE media ADD COLUMN sample_rate INTEGER;
ALTER TABLE media ADD COLUMN channels INTEGER;
ALTER TABLE media ADD COLUMN bitrate_kbps INTEGER;
UPDATE media SET mtime = -1;
";

/// Favourite tracks and albums, keyed by kind and key.
const SCHEMA_V4: &str = "
CREATE TABLE favorite (
    kind TEXT NOT NULL,
    key  TEXT NOT NULL,
    PRIMARY KEY (kind, key)
);
";

/// User playlists. Entries are numbered by `position`; removing an entry leaves a gap, which
/// is fine because entries are always read in order.
const SCHEMA_V5: &str = "
CREATE TABLE playlist (
    id      INTEGER PRIMARY KEY,
    name    TEXT NOT NULL,
    created INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE playlist_track (
    playlist_id INTEGER NOT NULL REFERENCES playlist(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    media_id    INTEGER NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    PRIMARY KEY (playlist_id, position)
);
";

/// When each file was first added, and a history of plays. Files indexed before this version
/// count as added at the time of the upgrade.
const SCHEMA_V6: &str = "
ALTER TABLE media ADD COLUMN added INTEGER NOT NULL DEFAULT 0;
UPDATE media SET added = strftime('%s', 'now');

CREATE TABLE play_history (
    id        INTEGER PRIMARY KEY,
    media_id  INTEGER NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    played_at INTEGER NOT NULL
);
CREATE INDEX play_history_by_media ON play_history(media_id);
";

/// Lyrics looked up online, cached per track. `text` is NULL when the lookup found nothing.
const SCHEMA_V7: &str = "
CREATE TABLE lyrics (
    media_id INTEGER PRIMARY KEY REFERENCES media(id) ON DELETE CASCADE,
    text     TEXT,
    fetched  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
";

/// Internet radio: the station index, the tuned-in list (with each station's address, so it plays
/// even if the index drops it), and small app state such as the last sync time.
const SCHEMA_V8: &str = "
CREATE TABLE station (
    uuid     TEXT PRIMARY KEY,
    name     TEXT NOT NULL,
    url      TEXT NOT NULL,
    country  TEXT NOT NULL DEFAULT '',
    codec    TEXT NOT NULL DEFAULT '',
    bitrate  INTEGER NOT NULL DEFAULT 0,
    tags     TEXT NOT NULL DEFAULT '',
    votes    INTEGER NOT NULL DEFAULT 0,
    working  INTEGER NOT NULL DEFAULT 0
);

CREATE VIRTUAL TABLE station_fts USING fts5(
    uuid UNINDEXED, name, tags, country,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TABLE tuned (
    uuid    TEXT PRIMARY KEY,
    name    TEXT NOT NULL,
    url     TEXT NOT NULL,
    country TEXT NOT NULL DEFAULT '',
    codec   TEXT NOT NULL DEFAULT '',
    bitrate INTEGER NOT NULL DEFAULT 0,
    added   INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

/// Two-letter country code of each station, for the flag shown in the radio list. Stations
/// stored before this version get it at the next sync.
const SCHEMA_V9: &str = "
ALTER TABLE station ADD COLUMN countrycode TEXT NOT NULL DEFAULT '';
";

/// The measured loudness of each file, with its modification time, so a changed file is measured
/// again.
const SCHEMA_V11: &str = "
CREATE TABLE loudness (
    path  TEXT PRIMARY KEY,
    lufs  REAL NOT NULL,
    peak  REAL NOT NULL,
    mtime INTEGER NOT NULL
);
";

/// Video stream columns for movies. Existing movie rows get a stale mtime so the next scan
/// re-reads them and fills the new columns.
const SCHEMA_V12: &str = "
ALTER TABLE media ADD COLUMN video_codec TEXT;
ALTER TABLE media ADD COLUMN width INTEGER;
ALTER TABLE media ADD COLUMN height INTEGER;
ALTER TABLE media ADD COLUMN video_bit_depth INTEGER;
UPDATE media SET mtime = -1 WHERE kind = 'video';
";

/// Movie details fetched from TMDb, as JSON, one row per movie. A movie's row goes with the
/// movie.
const SCHEMA_V13: &str = "
CREATE TABLE movie_details (
    media_id INTEGER PRIMARY KEY REFERENCES media(id) ON DELETE CASCADE,
    tmdb_id  INTEGER NOT NULL,
    details  TEXT NOT NULL
);
";

/// Chroma subsampling of video files. Existing movie rows get a stale mtime so the next scan
/// re-reads them and fills it in.
const SCHEMA_V14: &str = "
ALTER TABLE media ADD COLUMN video_chroma TEXT;
UPDATE media SET mtime = -1 WHERE kind = 'video';
";

/// The codec of MP4 files that symphonia does not name, such as H.264. Existing movie rows get a
/// stale mtime so the next scan reads their codec.
const SCHEMA_V15: &str = "
UPDATE media SET mtime = -1 WHERE kind = 'video';
";

/// Whether each file has been through its codec's processing step. A file that changes is
/// processed again.
const SCHEMA_V16: &str = "
ALTER TABLE media ADD COLUMN processed INTEGER NOT NULL DEFAULT 0;
";

/// Whether a video is HDR, from its colour transfer characteristics. Existing videos are processed
/// again to fill it in.
const SCHEMA_V20: &str = "
ALTER TABLE media ADD COLUMN hdr INTEGER;
UPDATE media SET processed = 0 WHERE kind = 'video';
";

/// Frame rate, whether it varies, and display aspect ratio of video files, from the video module.
/// Existing videos are processed again to fill them in.
const SCHEMA_V17: &str = "
ALTER TABLE media ADD COLUMN video_fps REAL;
ALTER TABLE media ADD COLUMN video_fps_variable INTEGER NOT NULL DEFAULT 0;
ALTER TABLE media ADD COLUMN video_aspect REAL;
UPDATE media SET processed = 0 WHERE kind = 'video';
";

/// How far each file was played, so that playing it again can resume from there.
const SCHEMA_V18: &str = "
CREATE TABLE progress (
    path          TEXT PRIMARY KEY,
    position_secs REAL NOT NULL,
    duration_secs REAL,
    updated       INTEGER NOT NULL
);
";

/// Whether a video still needs a TMDb lookup. A file that is new or changed needs one, and every
/// existing video gets one once, on the first scan after this step, when a TMDb key is set.
const SCHEMA_V19: &str = "
ALTER TABLE media ADD COLUMN lookup_pending INTEGER NOT NULL DEFAULT 1;
";

/// The tags each file had before the first edit made here, as JSON, until they are reverted.
const SCHEMA_V10: &str = "
CREATE TABLE tag_snapshot (path TEXT PRIMARY KEY, tags TEXT NOT NULL);
";

/// Lets album lookups use an index, so an album's tracks are found by name without scanning the
/// whole library.
const SCHEMA_V21: &str = "
CREATE INDEX media_audio_album_track ON media(kind, COALESCE(album, ''), track);
";
