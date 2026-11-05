use rusqlite::Connection;

use crate::Result;

/// Bump when the schema changes and add a matching migration step below.
const SCHEMA_VERSION: i64 = 7;

/// Per-connection settings and schema migration.
pub(crate) fn configure(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
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
