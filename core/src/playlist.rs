//! User playlists: named, ordered lists of tracks, kept in the library database. A track that
//! is removed from the library leaves its playlists too.

use rusqlite::{OptionalExtension, params};

use crate::{Library, M_COLUMNS, MediaItem, Result, row_to_item};

/// A playlist and how many tracks it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistSummary {
    pub id: i64,
    pub name: String,
    pub track_count: i64,
}

impl Library {
    /// All playlists, sorted by name.
    pub fn playlists(&self) -> Result<Vec<PlaylistSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.name, COUNT(pt.media_id) \
             FROM playlist p LEFT JOIN playlist_track pt ON pt.playlist_id = p.id \
             GROUP BY p.id ORDER BY p.name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(PlaylistSummary {
                id: r.get(0)?,
                name: r.get(1)?,
                track_count: r.get(2)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Create an empty playlist and return its id.
    pub fn create_playlist(&self, name: &str) -> Result<i64> {
        self.conn
            .execute("INSERT INTO playlist (name) VALUES (?1)", [name.trim()])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn rename_playlist(&self, id: i64, name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE playlist SET name = ?2 WHERE id = ?1",
            params![id, name.trim()],
        )?;
        Ok(())
    }

    /// Delete a playlist. Its tracks stay in the library.
    pub fn delete_playlist(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM playlist WHERE id = ?1", [id])?;
        Ok(())
    }

    /// The tracks of a playlist, in playlist order.
    pub fn playlist_tracks(&self, id: i64) -> Result<Vec<MediaItem>> {
        let sql = format!(
            "SELECT {M_COLUMNS} FROM playlist_track pt \
             JOIN media m ON m.id = pt.media_id \
             WHERE pt.playlist_id = ?1 ORDER BY pt.position"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([id], row_to_item)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Append tracks to a playlist. Adding the same track twice keeps both entries.
    pub fn add_to_playlist(&self, id: i64, media_ids: &[i64]) -> Result<()> {
        for media_id in media_ids {
            self.conn.execute(
                "INSERT INTO playlist_track (playlist_id, position, media_id) \
                 VALUES (?1, \
                         (SELECT COALESCE(MAX(position) + 1, 0) FROM playlist_track \
                          WHERE playlist_id = ?1), \
                         ?2)",
                params![id, media_id],
            )?;
        }
        Ok(())
    }

    /// Remove the entry at `position` (as stored, not as shown) from a playlist.
    pub fn remove_from_playlist(&self, id: i64, position: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM playlist_track WHERE playlist_id = ?1 AND position = ?2",
            params![id, position],
        )?;
        Ok(())
    }

    /// The stored positions of a playlist's entries, in playlist order. Pairs with
    /// [`Library::playlist_tracks`] when the UI needs to remove a row.
    pub fn playlist_positions(&self, id: i64) -> Result<Vec<i64>> {
        let mut stmt = self.conn.prepare(
            "SELECT position FROM playlist_track WHERE playlist_id = ?1 ORDER BY position",
        )?;
        let rows = stmt.query_map([id], |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Find the playlist with this name, if any.
    pub fn playlist_id(&self, name: &str) -> Result<Option<i64>> {
        Ok(self
            .conn
            .query_row(
                "SELECT id FROM playlist WHERE name = ?1 COLLATE NOCASE",
                [name.trim()],
                |r| r.get(0),
            )
            .optional()?)
    }
}
