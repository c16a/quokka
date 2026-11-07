//! TMDb (The Movie Database) lookups: search for a movie by title and year, then fetch its
//! details and credits. Each user supplies their own API key in Settings, or in the `TMDB_API_KEY`
//! environment variable. TMDb's terms require attributing it in the app's About or Credits screen.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::matching::Candidate;
use crate::{Error, Result};

const API: &str = "https://api.themoviedb.org/3";
const IMAGES: &str = "https://image.tmdb.org/t/p";

/// How many cast members a movie's details keep, in billing order.
pub const CAST_LIMIT: usize = 10;

/// One result of a title search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub id: u64,
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastMember {
    pub name: String,
    pub character: String,
    /// Path of the person's photo on TMDb's image server, when TMDb has one.
    #[serde(default)]
    pub profile_path: Option<String>,
}

/// A movie's details, with its credits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovieInfo {
    pub id: u64,
    pub title: String,
    pub year: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub genres: Vec<String>,
    pub tagline: Option<String>,
    pub overview: Option<String>,
    /// Path of the poster on TMDb's image server, such as `/abc.jpg`.
    pub poster_path: Option<String>,
    pub directors: Vec<String>,
    pub writers: Vec<String>,
    /// The first `CAST_LIMIT` cast members, in billing order.
    pub cast: Vec<CastMember>,
    /// Photo paths of the directors and writers, by name.
    #[serde(default)]
    pub photos: BTreeMap<String, String>,
    /// Whether these details were read with their photos. Details saved before photos were added
    /// lack this, so they are fetched again.
    #[serde(default)]
    pub photos_fetched: bool,
}

impl MovieInfo {
    /// Whether a photo is missing for someone in the cast, or for a director or writer, or the
    /// details predate the photos.
    pub fn is_missing_photos(&self) -> bool {
        !self.photos_fetched
            || self.cast.iter().any(|c| c.profile_path.is_none())
            || self
                .directors
                .iter()
                .chain(self.writers.iter())
                .any(|name| !self.photos.contains_key(name))
    }
}

/// A client for the TMDb v3 API.
pub struct Tmdb {
    key: String,
    agent: ureq::Agent,
}

impl Tmdb {
    pub fn new(key: impl Into<String>) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .into();
        Self {
            key: key.into(),
            agent,
        }
    }

    /// A client using the key in `TMDB_API_KEY`, or an error naming the variable when it is unset.
    pub fn from_env() -> Result<Self> {
        match std::env::var("TMDB_API_KEY") {
            Ok(key) if !key.trim().is_empty() => Ok(Self::new(key.trim())),
            _ => Err(Error::Tmdb("set TMDB_API_KEY to a TMDb API key".into())),
        }
    }

    /// Movies whose title matches `title`. A year narrows the results to that release year.
    pub fn search(&self, title: &str, year: Option<i64>) -> Result<Vec<SearchHit>> {
        let mut query = vec![("query", title.to_owned())];
        if let Some(year) = year {
            query.push(("year", year.to_string()));
        }
        let body = self.get("/search/movie", &query)?;
        parse_search(&body)
    }

    /// The details and credits of the movie with TMDb id `id`.
    pub fn movie(&self, id: u64) -> Result<MovieInfo> {
        let body = self.get(
            &format!("/movie/{id}"),
            &[("append_to_response", "credits".to_owned())],
        )?;
        parse_movie(&body)
    }

    /// The bytes of a poster image, at TMDb's medium size. `path` is a poster path from a search or
    /// movie response, such as `/abc.jpg`. The image server needs no key.
    pub fn poster_bytes(&self, path: &str) -> Result<Vec<u8>> {
        self.image_bytes("w342", path)
    }

    /// The bytes of an image at TMDb size `size`, such as `w185`. The image server needs no key.
    pub fn image_bytes(&self, size: &str, path: &str) -> Result<Vec<u8>> {
        let mut response = self
            .agent
            .get(format!("{IMAGES}/{size}{path}"))
            .call()
            .map_err(|e| Error::Tmdb(e.to_string()))?;
        response
            .body_mut()
            .read_to_vec()
            .map_err(|e| Error::Tmdb(e.to_string()))
    }

    fn get(&self, path: &str, query: &[(&str, String)]) -> Result<String> {
        let mut request = self
            .agent
            .get(format!("{API}{path}"))
            .query("api_key", &self.key)
            .query("language", "en-US");
        for (name, value) in query {
            request = request.query(name, value);
        }
        let mut response = request.call().map_err(|e| Error::Tmdb(e.to_string()))?;
        response
            .body_mut()
            .read_to_string()
            .map_err(|e| Error::Tmdb(e.to_string()))
    }
}

/// The year in a `YYYY-MM-DD` release date.
fn year_of(date: Option<&str>) -> Option<i64> {
    date?.get(..4)?.parse().ok()
}

#[derive(Deserialize)]
struct SearchBody {
    results: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    id: u64,
    title: String,
    original_title: Option<String>,
    release_date: Option<String>,
}

/// Parse a `/search/movie` response.
pub fn parse_search(json: &str) -> Result<Vec<SearchHit>> {
    let body: SearchBody = serde_json::from_str(json)?;
    Ok(body
        .results
        .into_iter()
        .map(|r| SearchHit {
            id: r.id,
            title: r.title,
            original_title: r.original_title,
            year: year_of(r.release_date.as_deref()),
        })
        .collect())
}

/// The titles of search hits as candidates for matching. Each hit contributes its original title
/// too, when that differs.
pub fn candidates(hits: &[SearchHit]) -> Vec<Candidate> {
    let mut out = Vec::new();
    for hit in hits {
        out.push(Candidate {
            id: hit.id.to_string(),
            title: hit.title.clone(),
            year: hit.year,
        });
        if let Some(original) = hit.original_title.as_deref()
            && original != hit.title
        {
            out.push(Candidate {
                id: hit.id.to_string(),
                title: original.to_owned(),
                year: hit.year,
            });
        }
    }
    out
}

#[derive(Deserialize)]
struct MovieBody {
    id: u64,
    title: String,
    release_date: Option<String>,
    runtime: Option<i64>,
    #[serde(default)]
    genres: Vec<Named>,
    tagline: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    credits: Option<CreditsBody>,
}

#[derive(Deserialize)]
struct Named {
    name: String,
}

#[derive(Deserialize)]
struct CreditsBody {
    #[serde(default)]
    cast: Vec<CastBody>,
    #[serde(default)]
    crew: Vec<CrewBody>,
}

#[derive(Deserialize)]
struct CastBody {
    name: String,
    character: Option<String>,
    order: Option<u32>,
    profile_path: Option<String>,
}

#[derive(Deserialize)]
struct CrewBody {
    name: String,
    job: String,
    profile_path: Option<String>,
}

/// Parse a `/movie/{id}?append_to_response=credits` response.
pub fn parse_movie(json: &str) -> Result<MovieInfo> {
    let body: MovieBody = serde_json::from_str(json)?;
    let credits = body.credits.unwrap_or(CreditsBody {
        cast: Vec::new(),
        crew: Vec::new(),
    });
    let mut cast = credits.cast;
    cast.sort_by_key(|c| c.order.unwrap_or(u32::MAX));
    let mut photos = BTreeMap::new();
    for member in &credits.crew {
        if matches!(member.job.as_str(), "Director" | "Writer")
            && let Some(path) = non_empty(member.profile_path.clone())
        {
            photos.entry(member.name.clone()).or_insert(path);
        }
    }
    let names_for = |job: &str| -> Vec<String> {
        credits
            .crew
            .iter()
            .filter(|c| c.job == job)
            .map(|c| c.name.clone())
            .collect()
    };
    Ok(MovieInfo {
        id: body.id,
        title: body.title,
        year: year_of(body.release_date.as_deref()),
        runtime_minutes: body.runtime.filter(|&m| m > 0),
        genres: body.genres.into_iter().map(|g| g.name).collect(),
        tagline: non_empty(body.tagline),
        overview: non_empty(body.overview),
        poster_path: non_empty(body.poster_path),
        directors: names_for("Director"),
        writers: names_for("Writer"),
        cast: cast
            .into_iter()
            .take(CAST_LIMIT)
            .map(|c| CastMember {
                name: c.name,
                character: c.character.unwrap_or_default(),
                profile_path: non_empty(c.profile_path),
            })
            .collect(),
        photos,
        photos_fetched: true,
    })
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty())
}
