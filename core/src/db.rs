use rusqlite::Connection;

use crate::Result;

/// Bump when the schema changes and add a matching migration step below.
const SCHEMA_VERSION: i64 = 3;

/// Per-connection settings and schema migration.
pub(crate) fn configure(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
    // WAL is not available for in-memory databases; ignore the result there.
    let _ = conn.pragma_update(None, "journal_mode", "WAL");

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
