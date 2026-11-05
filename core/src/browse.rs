//! Queries that group the library for browsing: albums (audio) and movies (video).

use std::collections::{BTreeSet, HashMap};

use crate::{COLUMNS, Library, MediaItem, MediaKind, Result, row_to_item};

/// One album: all audio items sharing an album tag. Tracks with different track
/// or album artists are merged into one album, so the same album name appears once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumSummary {
    /// `None` when the files carry no album tag.
    pub album: Option<String>,
    /// Album artists across the album, sorted. Falls back to track artists when
    /// the album artist tag is missing.
    pub artists: Vec<String>,
    /// Composers across the album, sorted. Empty when no composer tag is present.
    pub composers: Vec<String>,
    pub year: Option<i64>,
    pub track_count: i64,
}

impl Library {
    /// All albums, sorted by name. Untagged albums sort last.
    pub fn albums(&self) -> Result<Vec<AlbumSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(album, ''), MAX(year), COUNT(*) \
             FROM media WHERE kind = 'audio' \
             GROUP BY COALESCE(album, '') \
             ORDER BY COALESCE(album, '') = '', COALESCE(album, '') COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |r| {
            let album: String = r.get(0)?;
            Ok((album, r.get::<_, Option<i64>>(1)?, r.get::<_, i64>(2)?))
        })?;
        let rows: Vec<(String, Option<i64>, i64)> = rows.collect::<rusqlite::Result<_>>()?;

        // Artists and composers per album name, from every distinct combination.
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT COALESCE(album, ''), COALESCE(album_artist, artist, ''), \
                    COALESCE(composer, '') \
             FROM media WHERE kind = 'audio'",
        )?;
        let mut people: HashMap<String, (BTreeSet<String>, BTreeSet<String>)> = HashMap::new();
        let triples = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        for triple in triples {
            let (album, artist, composer) = triple?;
            let entry = people.entry(album).or_default();
            if !artist.is_empty() {
                entry.0.insert(artist);
            }
            if !composer.is_empty() {
                entry.1.insert(composer);
            }
        }

        Ok(rows
            .into_iter()
            .map(|(album, year, track_count)| {
                let (artists, composers) = people.remove(&album).unwrap_or_default();
                AlbumSummary {
                    album: non_empty(album),
                    artists: artists.into_iter().collect(),
                    composers: composers.into_iter().collect(),
                    year,
                    track_count,
                }
            })
            .collect())
    }

    /// Tracks of one album in track-number order. `None` matches untagged albums.
    pub fn album_tracks(&self, album: Option<&str>) -> Result<Vec<MediaItem>> {
        let sql = format!(
            "SELECT {COLUMNS} FROM media \
             WHERE kind = 'audio' AND COALESCE(album, '') = ?1 \
             ORDER BY track IS NULL, track, path"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([album.unwrap_or("")], row_to_item)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// All video items, sorted by title (falling back to file name).
    pub fn movies(&self) -> Result<Vec<MediaItem>> {
        let sql = format!(
            "SELECT {COLUMNS} FROM media WHERE kind = ?1 \
             ORDER BY COALESCE(title, path) COLLATE NOCASE"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([MediaKind::Video.as_str()], row_to_item)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

fn non_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}
