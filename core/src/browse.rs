//! Queries that group the library for browsing: albums (audio) and movies (video).

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use rusqlite::params;
use rusqlite::types::Value;

use crate::{COLUMNS, Library, MediaItem, MediaKind, Result, row_to_item};

/// How the album grid is ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlbumSort {
    /// By album name, untagged albums last.
    #[default]
    Name,
    /// Oldest first, untagged years last.
    Year,
    /// Most recently added first.
    Added,
}

/// Which albums the grid keeps. An album passes a genre if any of its tracks has that genre.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AlbumFilter {
    pub genre: Option<String>,
    pub year: Option<i64>,
}

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
        self.albums_by(AlbumSort::Name, &AlbumFilter::default())
    }

    /// Albums in `sort` order, keeping only those that pass `filter`.
    pub fn albums_by(&self, sort: AlbumSort, filter: &AlbumFilter) -> Result<Vec<AlbumSummary>> {
        let mut values: Vec<Value> = Vec::new();
        let mut where_sql = String::from("kind = 'audio'");
        if let Some(genre) = &filter.genre {
            values.push(Value::Text(genre.clone()));
            where_sql.push_str(&format!(
                " AND COALESCE(album, '') IN \
                  (SELECT COALESCE(album, '') FROM media WHERE kind = 'audio' AND genre = ?{})",
                values.len()
            ));
        }
        let mut having_sql = String::new();
        if let Some(year) = filter.year {
            values.push(Value::Integer(year));
            having_sql = format!(" HAVING MAX(year) = ?{}", values.len());
        }
        let order_sql = match sort {
            AlbumSort::Name => "COALESCE(album, '') = '', COALESCE(album, '') COLLATE NOCASE",
            AlbumSort::Year => "MAX(year) IS NULL, MAX(year), COALESCE(album, '') COLLATE NOCASE",
            AlbumSort::Added => "MIN(added) DESC, COALESCE(album, '') COLLATE NOCASE",
        };
        let sql = format!(
            "SELECT COALESCE(album, ''), MAX(year), COUNT(*) \
             FROM media WHERE {where_sql} \
             GROUP BY COALESCE(album, '') {having_sql} \
             ORDER BY {order_sql}"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(values.iter()), |r| {
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

    /// Every genre tagged on an audio file, sorted.
    pub fn genres(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT genre FROM media \
             WHERE kind = 'audio' AND genre IS NOT NULL AND genre <> '' \
             ORDER BY genre COLLATE NOCASE",
        )?;
        Ok(stmt
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?)
    }

    /// Every year tagged on an audio file, newest first.
    pub fn years(&self) -> Result<Vec<i64>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT year FROM media WHERE kind = 'audio' AND year IS NOT NULL \
             ORDER BY year DESC",
        )?;
        Ok(stmt
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?)
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

    /// Embedded cover art for an album. Tries the first few tracks, since not all of them carry one.
    pub fn album_cover(&self, album: Option<&str>) -> Result<Option<Vec<u8>>> {
        let mut stmt = self.conn.prepare(
            "SELECT path FROM media WHERE kind = 'audio' AND COALESCE(album, '') = ?1 \
             ORDER BY track IS NULL, track, path LIMIT 5",
        )?;
        let paths: Vec<String> = stmt
            .query_map([album.unwrap_or("")], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(paths
            .iter()
            .find_map(|p| crate::cover::read_cover(Path::new(p))))
    }

    /// Audio tracks whose title contains `text` (case-insensitive), sorted by title.
    pub fn songs_matching(&self, text: &str, limit: usize) -> Result<Vec<MediaItem>> {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("%{escaped}%");
        let sql = format!(
            "SELECT {COLUMNS} FROM media \
             WHERE kind = 'audio' AND title LIKE ?1 ESCAPE '\\' \
             ORDER BY title COLLATE NOCASE \
             LIMIT ?2"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![pattern, limit as i64], row_to_item)?;
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
