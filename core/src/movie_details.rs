//! Movie details from TMDb, kept in the index beside each movie. A movie is matched to TMDb by its
//! file name. A confident match is saved straight away. A weak match, or a request to change the
//! match, returns up to `CHOICE_LIMIT` candidates for the user to pick from.

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{OptionalExtension, params};

use crate::matching::{self, MATCH_THRESHOLD};
use crate::tmdb::{self, MovieInfo, Tmdb};
use crate::{Error, Library, Result};

/// How many candidates a choice list offers.
pub const CHOICE_LIMIT: usize = 5;

/// Pause after each automatic lookup. TMDb allows about 40 requests every 10 seconds, and a lookup
/// makes at most two, so this keeps a long scan well under that.
const LOOKUP_PAUSE: std::time::Duration = std::time::Duration::from_millis(300);

/// What a lookup found.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchOutcome {
    /// A confident match, now saved.
    Matched(Box<MovieInfo>),
    /// Candidates to choose from, best first. Nothing is saved until one is picked.
    Choices(Vec<MovieInfo>),
    /// No TMDb title for this file name.
    Nothing,
}

impl Library {
    /// The TMDb details saved for the movie `media_id`, if it has been matched.
    pub fn movie_details(&self, media_id: i64) -> Result<Option<MovieInfo>> {
        let json: Option<String> = self
            .conn
            .query_row(
                "SELECT details FROM movie_details WHERE media_id = ?1",
                [media_id],
                |r| r.get(0),
            )
            .optional()?;
        match json {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    /// The saved details of every matched movie, keyed by media id.
    pub fn all_movie_details(&self) -> Result<HashMap<i64, MovieInfo>> {
        let mut stmt = self
            .conn
            .prepare("SELECT media_id, details FROM movie_details")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        let mut out = HashMap::new();
        for row in rows {
            let (media_id, json) = row?;
            out.insert(media_id, serde_json::from_str(&json)?);
        }
        Ok(out)
    }

    /// Look up the movie `media_id` at `path` on TMDb. A confident match is saved and returned as
    /// `Matched`. Otherwise, or when `force_choices` is set, the candidates come back as `Choices`
    /// and nothing is saved.
    pub fn find_movie(
        &self,
        media_id: i64,
        path: &Path,
        tmdb: &Tmdb,
        force_choices: bool,
    ) -> Result<MatchOutcome> {
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            return Ok(MatchOutcome::Nothing);
        };
        let parsed = matching::parse_file_name(stem);
        let mut hits = tmdb.search(&parsed.title, parsed.year)?;
        if hits.is_empty() && parsed.year.is_some() {
            hits = tmdb.search(&parsed.title, None)?;
        }
        let candidates = tmdb::candidates(&hits);
        let ranked = matching::best_matches(&parsed, &candidates, candidates.len());
        let Some(best) = ranked.first() else {
            return Ok(MatchOutcome::Nothing);
        };

        // A name without a year can't be told apart from another film with the same title, so
        // it always offers choices. Only a name with a matching year is saved on its own.
        if !force_choices && parsed.year.is_some() && best.score >= MATCH_THRESHOLD {
            let info = tmdb.movie(parse_id(&best.candidate.id)?)?;
            self.save_movie_details(media_id, &info)?;
            return Ok(MatchOutcome::Matched(Box::new(info)));
        }

        // Candidates in score order, each title once.
        let mut ids: Vec<u64> = Vec::new();
        for hit in &ranked {
            let id = parse_id(&hit.candidate.id)?;
            if !ids.contains(&id) {
                ids.push(id);
            }
            if ids.len() == CHOICE_LIMIT {
                break;
            }
        }
        let infos = ids
            .into_iter()
            .map(|id| tmdb.movie(id))
            .collect::<Result<Vec<_>>>()?;
        Ok(MatchOutcome::Choices(infos))
    }

    /// Look up the movie `media_id` at `path` on TMDb, and save its details only when there is one
    /// confident match with a year. Returns whether it was saved. Anything less certain is left for
    /// the user, who picks from the choices on the movie's page.
    pub fn match_confident(&self, media_id: i64, path: &Path, tmdb: &Tmdb) -> Result<bool> {
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            return Ok(false);
        };
        let parsed = matching::parse_file_name(stem);
        if parsed.year.is_none() {
            return Ok(false);
        }
        let mut hits = tmdb.search(&parsed.title, parsed.year)?;
        if hits.is_empty() {
            hits = tmdb.search(&parsed.title, None)?;
        }
        let candidates = tmdb::candidates(&hits);
        let ranked = matching::best_matches(&parsed, &candidates, candidates.len());
        let Some(best) = ranked.first() else {
            return Ok(false);
        };
        if best.score < MATCH_THRESHOLD {
            return Ok(false);
        }
        let info = tmdb.movie(parse_id(&best.candidate.id)?)?;
        self.save_movie_details(media_id, &info)?;
        Ok(true)
    }

    /// Run `lookup` on every video whose TMDb lookup is pending, and mark each one done once it has
    /// been tried. `lookup` returns whether it saved details. A failed lookup stops the pass and
    /// stays pending, so the next scan tries it again. Returns how many were saved.
    pub fn match_pending<F>(&self, mut lookup: F) -> Result<usize>
    where
        F: FnMut(i64, &Path) -> Result<bool>,
    {
        let pending: Vec<(i64, String)> = {
            let mut stmt = self.conn.prepare(
                "SELECT id, path FROM media WHERE kind = 'video' AND lookup_pending = 1 ORDER BY id",
            )?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            rows.collect::<rusqlite::Result<_>>()?
        };
        let mut saved = 0;
        for (media_id, path) in pending {
            if lookup(media_id, Path::new(&path))? {
                saved += 1;
            }
            self.conn.execute(
                "UPDATE media SET lookup_pending = 0 WHERE id = ?1",
                [media_id],
            )?;
        }
        Ok(saved)
    }

    /// Look up every pending video on TMDb and save the confident matches. A video that already has
    /// details is marked done without a request. Returns how many were saved.
    pub fn match_pending_movies(&self, tmdb: &Tmdb) -> Result<usize> {
        self.match_pending(|media_id, path| {
            if self.movie_details(media_id)?.is_some() {
                return Ok(false);
            }
            let saved = self.match_confident(media_id, path, tmdb)?;
            std::thread::sleep(LOOKUP_PAUSE);
            Ok(saved)
        })
    }

    /// Save the details of movie `media_id`, replacing any saved before.
    pub fn save_movie_details(&self, media_id: i64, info: &MovieInfo) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO movie_details (media_id, tmdb_id, details) VALUES (?1, ?2, ?3)",
            params![media_id, info.id as i64, serde_json::to_string(info)?],
        )?;
        Ok(())
    }

    /// Fetch the details of movie `media_id` again, by its TMDb id, and save them. Used to add photos
    /// to details saved before photos were stored.
    pub fn refresh_movie_details(
        &self,
        media_id: i64,
        tmdb_id: u64,
        tmdb: &Tmdb,
    ) -> Result<MovieInfo> {
        let info = tmdb.movie(tmdb_id)?;
        self.save_movie_details(media_id, &info)?;
        Ok(info)
    }

    /// Forget the TMDb details of movie `media_id`. The movie itself is kept.
    pub fn clear_movie_details(&self, media_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM movie_details WHERE media_id = ?1", [media_id])?;
        Ok(())
    }
}

fn parse_id(text: &str) -> Result<u64> {
    text.parse()
        .map_err(|_| Error::Tmdb(format!("bad TMDb id {text}")))
}
