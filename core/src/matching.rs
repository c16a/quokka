//! Matching a file name to a movie title from an online database. The name is cleaned into a
//! title and a year. Each candidate title is then scored by its edit distance to that title,
//! with a bonus when the years agree.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use flate2::read::GzDecoder;

use crate::Result;

/// A best match scoring below this is a guess to check by hand.
pub const MATCH_THRESHOLD: f64 = 0.85;

/// A title and an optional year, read from a file name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedName {
    pub title: String,
    pub year: Option<i64>,
}

/// A movie title from a database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// The database's id for the title, such as an IMDb `tt` id.
    pub id: String,
    pub title: String,
    pub year: Option<i64>,
}

/// A candidate with its score against a parsed name. Higher is closer; at most 1.
#[derive(Debug, Clone, Copy)]
pub struct Scored<'a> {
    pub candidate: &'a Candidate,
    pub score: f64,
}

/// Words that end the title in a release name: codec, source, audio and so on.
const TAG_WORDS: &[&str] = &[
    "av1",
    "x264",
    "x265",
    "h264",
    "h265",
    "hevc",
    "avc",
    "vp9",
    "aac",
    "ac3",
    "eac3",
    "dts",
    "truehd",
    "atmos",
    "flac",
    "opus",
    "hdr",
    "hdr10",
    "hdr10plus",
    "dv",
    "dovi",
    "dolby",
    "vision",
    "web",
    "webdl",
    "webrip",
    "bluray",
    "bdrip",
    "brrip",
    "hdtv",
    "dvdrip",
    "remux",
    "proper",
    "repack",
    "internal",
    "multi",
    "yify",
    "yts",
    "rarbg",
    "uhd",
    "sdr",
    "4k",
    "8k",
];

/// Whether a token of a file name is a release tag rather than part of the title: a resolution
/// (`1080p`), a frame size (`3840x2160`), a bit depth (`10bit`), a frame rate (`5994fps`) or a
/// bitrate (`5967kbps`).
fn is_tag(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    if TAG_WORDS.contains(&lower.as_str()) {
        return true;
    }
    if let Some(digits) = lower.strip_suffix('p')
        && all_digits(digits)
    {
        return true;
    }
    if let Some((width, height)) = lower.split_once('x')
        && all_digits(width)
        && all_digits(height)
    {
        return true;
    }
    ["bit", "fps", "kbps", "mbps"]
        .iter()
        .any(|suffix| lower.strip_suffix(suffix).is_some_and(all_digits))
}

fn all_digits(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_digit())
}

/// The year a token names, for years from 1900 to 2099.
fn as_year(token: &str) -> Option<i64> {
    if token.len() != 4 || !all_digits(token) {
        return None;
    }
    let year: i64 = token.parse().ok()?;
    (1900..=2099).contains(&year).then_some(year)
}

/// Clean a file name (without its extension) into a title and a year. The title runs up to the
/// first release tag, or to a year that follows some title words. Separators such as dots,
/// dashes and brackets are dropped, so `The.Matrix.1999.1080p.BluRay` reads as "The Matrix" 1999.
pub fn parse_file_name(name: &str) -> ParsedName {
    // Apostrophes join a word rather than separate it: "Schindler's" is one word.
    let name: String = name.chars().filter(|c| !matches!(c, '\'' | '’')).collect();
    let mut words = Vec::new();
    let mut year = None;
    for token in name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
    {
        if is_tag(token) {
            break;
        }
        if let Some(y) = as_year(token)
            && !words.is_empty()
        {
            year = Some(y);
            break;
        }
        words.push(token);
    }
    ParsedName {
        title: words.join(" "),
        year,
    }
}

/// Lower-case words separated by single spaces, with punctuation removed. Titles are compared in
/// this form.
fn normalise(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The Levenshtein edit distance between two sequences of characters.
fn edit_distance(a: &[char], b: &[char]) -> usize {
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(ca != cb);
            current[j + 1] = substitution.min(previous[j + 1] + 1).min(current[j] + 1);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

/// How alike two titles are, from 0 (nothing alike) to 1 (the same). The titles are normalised
/// first, so case and punctuation don't count.
pub fn similarity(a: &str, b: &str) -> f64 {
    let (a, b) = (normalise(a), normalise(b));
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let longest = a.len().max(b.len());
    if longest == 0 {
        return 0.0;
    }
    1.0 - edit_distance(&a, &b) as f64 / longest as f64
}

/// The score of a candidate against a parsed name. The title similarity is adjusted by the years:
/// a bonus when they match, a small penalty when they are a year apart, and a larger penalty when
/// they differ by more. Without a year on either side, only the title counts.
pub fn score(parsed: &ParsedName, candidate: &Candidate) -> f64 {
    let title = similarity(&parsed.title, &candidate.title);
    let year = match (parsed.year, candidate.year) {
        (Some(a), Some(b)) => match (a - b).abs() {
            0 => 0.1,
            1 => 0.0,
            _ => -0.2,
        },
        _ => 0.0,
    };
    (title + year).clamp(0.0, 1.0)
}

/// The `limit` candidates that score highest against `parsed`, best first.
pub fn best_matches<'a>(
    parsed: &ParsedName,
    candidates: &'a [Candidate],
    limit: usize,
) -> Vec<Scored<'a>> {
    let mut scored: Vec<Scored<'a>> = candidates
        .iter()
        .map(|candidate| Scored {
            candidate,
            score: score(parsed, candidate),
        })
        .collect();
    scored.sort_by(|a, b| b.score.total_cmp(&a.score));
    scored.truncate(limit);
    scored
}

/// Read movie titles from an IMDb `title.basics` file. A `.gz` file is decompressed as it is read.
/// Only movies and TV movies are kept, and adult titles are left out. Each title is kept under its
/// original name as well, when that differs.
pub fn read_imdb_movies(path: &Path) -> Result<Vec<Candidate>> {
    let file = File::open(path)?;
    let gzipped = path.extension().is_some_and(|ext| ext == "gz");
    if gzipped {
        parse_imdb_movies(BufReader::new(GzDecoder::new(file)))
    } else {
        parse_imdb_movies(BufReader::new(file))
    }
}

/// Parse the tab-separated `title.basics` format, which starts with a header line.
pub fn parse_imdb_movies(reader: impl BufRead) -> Result<Vec<Candidate>> {
    let mut candidates = Vec::new();
    for line in reader.lines().skip(1) {
        let line = line?;
        let columns: Vec<&str> = line.split('\t').collect();
        let [id, kind, primary, original, adult, start_year, ..] = columns.as_slice() else {
            continue;
        };
        if !matches!(*kind, "movie" | "tvMovie") || *adult == "1" {
            continue;
        }
        let year = start_year.parse().ok();
        candidates.push(Candidate {
            id: (*id).to_owned(),
            title: (*primary).to_owned(),
            year,
        });
        if original != primary && *original != "\\N" {
            candidates.push(Candidate {
                id: (*id).to_owned(),
                title: (*original).to_owned(),
                year,
            });
        }
    }
    Ok(candidates)
}
