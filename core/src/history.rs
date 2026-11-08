//! Play history, and the albums that were added or played most recently.

use rusqlite::params;

use crate::{Library, Result};

/// Which recent list to make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recent {
    /// Albums by when their first file was added, newest first.
    Added,
    /// Albums by when one of their tracks was last played, most recent first.
    Played,
}

impl Library {
    /// Note that a track started playing now.
    pub fn record_play(&self, media_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO play_history (media_id, played_at) VALUES (?1, strftime('%s', 'now'))",
            params![media_id],
        )?;
        Ok(())
    }

    /// Album names for a recent list, at most `limit` of them. An untagged album is `""`.
    pub fn recent_albums(&self, recent: Recent, limit: usize) -> Result<Vec<String>> {
        let sql = match recent {
            Recent::Added => {
                "SELECT COALESCE(album, '') FROM media WHERE kind = 'audio' \
                 GROUP BY COALESCE(album, '') ORDER BY MIN(added) DESC LIMIT ?1"
            }
            Recent::Played => {
                "SELECT COALESCE(m.album, '') FROM play_history p \
                 JOIN media m ON m.id = p.media_id WHERE m.kind = 'audio' \
                 GROUP BY COALESCE(m.album, '') ORDER BY MAX(p.played_at) DESC LIMIT ?1"
            }
        };
        let mut stmt = self.conn.prepare_cached(sql)?;
        let rows = stmt.query_map([limit as i64], |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
