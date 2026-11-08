//! Favourite tracks and albums. Tracks are keyed by their file path, albums by album name.

use std::collections::HashSet;

use rusqlite::params;

use crate::{Library, Result};

/// What a favourite refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FavoriteKind {
    /// A track, keyed by its file path.
    Track,
    /// An album, keyed by its album name (empty for albums without one).
    Album,
}

impl FavoriteKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Track => "track",
            Self::Album => "album",
        }
    }
}

impl Library {
    /// Mark `key` as a favourite of `kind`, or unmark it. Marking twice is harmless.
    pub fn set_favorite(&self, kind: FavoriteKind, key: &str, on: bool) -> Result<()> {
        if on {
            self.conn.execute(
                "INSERT OR IGNORE INTO favorite (kind, key) VALUES (?1, ?2)",
                params![kind.as_str(), key],
            )?;
        } else {
            self.conn.execute(
                "DELETE FROM favorite WHERE kind = ?1 AND key = ?2",
                params![kind.as_str(), key],
            )?;
        }
        Ok(())
    }

    /// The keys of every favourite of `kind`.
    pub fn favorites(&self, kind: FavoriteKind) -> Result<HashSet<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT key FROM favorite WHERE kind = ?1")?;
        let keys = stmt
            .query_map([kind.as_str()], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<HashSet<_>>>()?;
        Ok(keys)
    }
}
