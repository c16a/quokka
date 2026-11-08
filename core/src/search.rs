//! Song search with filters. Free words match the title. `artist:`, `album:` and `genre:` match
//! part of the name; `year:` matches exactly; `format:` (or `codec:`) matches the codec, such as
//! `flac`. A value with spaces goes in quotes: `artist:"Roop Kumar Rathod"`.

use rusqlite::types::Value;

use crate::{COLUMNS, Library, MediaItem, Result, fts_query, row_to_item};

/// A parsed search. Empty fields don't restrict the results.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SongQuery {
    /// Words that must start a word in the title.
    pub text: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i64>,
    /// Lower-cased codec label, such as `flac` or `mp3`.
    pub format: Option<String>,
}

impl SongQuery {
    /// Read a query as typed. Unknown `key:value` words, and a year that isn't a number, stay as
    /// title words.
    pub fn parse(input: &str) -> Self {
        let mut query = Self::default();
        let mut words = Vec::new();
        for word in tokens(input) {
            let Some((key, value)) = word.split_once(':') else {
                words.push(word);
                continue;
            };
            let value = value.trim();
            if value.is_empty() {
                words.push(word);
                continue;
            }
            match key.to_ascii_lowercase().as_str() {
                "artist" => query.artist = Some(value.to_owned()),
                "album" => query.album = Some(value.to_owned()),
                "genre" => query.genre = Some(value.to_owned()),
                "year" => match value.parse() {
                    Ok(year) => query.year = Some(year),
                    Err(_) => words.push(word),
                },
                "format" | "codec" => query.format = Some(value.to_ascii_lowercase()),
                _ => words.push(word),
            }
        }
        query.text = words.join(" ");
        query
    }

    /// Whether the query has nothing to match.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
            && self.artist.is_none()
            && self.album.is_none()
            && self.genre.is_none()
            && self.year.is_none()
            && self.format.is_none()
    }

    /// Whether the query uses a filter that album names can't answer (genre or format).
    pub fn has_track_filters(&self) -> bool {
        self.genre.is_some() || self.format.is_some()
    }
}

/// Split on whitespace, except inside double quotes. The quotes are dropped.
fn tokens(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in input.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// A LIKE pattern that matches `text` anywhere, case-insensitively.
fn like(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

impl Library {
    /// Audio tracks matching `query`, sorted by title. Placeholders are bound in the order the
    /// conditions are added, so each condition pushes its values right after its SQL.
    pub fn search_songs(&self, query: &SongQuery, limit: usize) -> Result<Vec<MediaItem>> {
        let mut conditions = vec!["kind = 'audio'".to_string()];
        let mut values: Vec<Value> = Vec::new();
        if let Some(matching) = fts_query(&query.text, Some("title")) {
            conditions
                .push("rowid IN (SELECT rowid FROM media_fts WHERE media_fts MATCH ?)".into());
            values.push(Value::Text(matching));
        }
        if let Some(artist) = &query.artist {
            conditions.push(
                "(COALESCE(artist, '') LIKE ? ESCAPE '\\' OR COALESCE(album_artist, '') LIKE ? ESCAPE '\\')"
                    .into(),
            );
            values.push(Value::Text(like(artist)));
            values.push(Value::Text(like(artist)));
        }
        if let Some(album) = &query.album {
            conditions.push("COALESCE(album, '') LIKE ? ESCAPE '\\'".into());
            values.push(Value::Text(like(album)));
        }
        if let Some(genre) = &query.genre {
            conditions.push("COALESCE(genre, '') LIKE ? ESCAPE '\\'".into());
            values.push(Value::Text(like(genre)));
        }
        if let Some(year) = query.year {
            conditions.push("year = ?".into());
            values.push(Value::Integer(year));
        }
        if let Some(format) = &query.format {
            conditions.push("LOWER(COALESCE(codec, '')) = ?".into());
            values.push(Value::Text(format.clone()));
        }
        values.push(Value::Integer(limit as i64));
        let sql = format!(
            "SELECT {COLUMNS} FROM media WHERE {} ORDER BY title COLLATE NOCASE LIMIT ?",
            conditions.join(" AND ")
        );
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(values.iter()), row_to_item)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::params;

    use super::SongQuery;
    use crate::Library;

    fn insert_song(lib: &Library, path: &str, title: &str) {
        lib.conn
            .execute(
                "INSERT INTO media (path, kind, size, mtime, title) VALUES (?1, 'audio', 1, 1, ?2)",
                params![path, title],
            )
            .unwrap();
    }

    fn titles(lib: &Library, input: &str) -> Vec<String> {
        lib.search_songs(&SongQuery::parse(input), 50)
            .unwrap()
            .into_iter()
            .filter_map(|item| item.title)
            .collect()
    }

    #[test]
    fn title_words_match_by_prefix() {
        let lib = Library::open_in_memory().unwrap();
        insert_song(&lib, "/m/a.mp3", "Love Story");
        insert_song(&lib, "/m/b.mp3", "Glove");
        insert_song(&lib, "/m/c.mp3", "Something Else");

        assert_eq!(titles(&lib, "lov"), vec!["Love Story"]);
        assert_eq!(titles(&lib, "sto"), vec!["Love Story"]);
        assert_eq!(titles(&lib, "love sto"), vec!["Love Story"]);
        assert!(titles(&lib, "ve").is_empty());
    }

    #[test]
    fn title_search_ignores_artist_and_album_words() {
        let lib = Library::open_in_memory().unwrap();
        lib.conn
            .execute(
                "INSERT INTO media (path, kind, size, mtime, title, artist) \
                 VALUES ('/m/a.mp3', 'audio', 1, 1, 'Dawn', 'Love Band')",
                [],
            )
            .unwrap();

        assert!(titles(&lib, "love").is_empty());
    }

    #[test]
    fn filters_are_read_and_other_words_stay_as_title_text() {
        let q = SongQuery::parse(r#"love artist:"Roop Kumar Rathod" year:2005 format:FLAC"#);
        assert_eq!(q.text, "love");
        assert_eq!(q.artist.as_deref(), Some("Roop Kumar Rathod"));
        assert_eq!(q.year, Some(2005));
        assert_eq!(q.format.as_deref(), Some("flac"));
        assert!(q.album.is_none());
    }

    #[test]
    fn unknown_keys_and_bad_years_are_title_words() {
        let q = SongQuery::parse("mood:sad year:soon 10:30");
        assert_eq!(q.text, "mood:sad year:soon 10:30");
        assert!(q.year.is_none());
        assert!(SongQuery::parse("   ").is_empty());
    }
}
