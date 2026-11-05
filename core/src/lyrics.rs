//! Lyrics. A `.lrc` file beside a track always wins. Otherwise synced lyrics are preferred to
//! plain ones, from the file's own tags or from a lookup on LRCLIB (https://lrclib.net), which
//! is cached per track in the library.

use std::fs;
use std::path::Path;
use std::time::Duration;

use rusqlite::OptionalExtension;
use serde::Deserialize;

use crate::{Library, Result};

/// One timed line of synced lyrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricLine {
    pub at: Duration,
    pub text: String,
}

/// Lyrics for one track.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lyrics {
    /// Lines with the time each one starts, in order.
    Synced(Vec<LyricLine>),
    /// Text without timing.
    Plain(String),
}

impl Lyrics {
    pub fn is_synced(&self) -> bool {
        matches!(self, Self::Synced(_))
    }
}

/// Parse LRC text: lines that start with one or more `[mm:ss.xx]` times. Metadata tags such as
/// `[ti:…]` are skipped. Returns `None` when no line has a time.
pub fn parse_lrc(text: &str) -> Option<Vec<LyricLine>> {
    let mut lines = Vec::new();
    for raw in text.lines() {
        let mut rest = raw.trim();
        let mut times = Vec::new();
        while let Some(after_bracket) = rest.strip_prefix('[') {
            let Some(end) = after_bracket.find(']') else {
                break;
            };
            let Some(at) = parse_time(&after_bracket[..end]) else {
                break;
            };
            times.push(at);
            rest = after_bracket[end + 1..].trim_start();
        }
        for at in times {
            lines.push(LyricLine {
                at,
                text: rest.trim().to_owned(),
            });
        }
    }
    if lines.is_empty() {
        return None;
    }
    // Stable, so lines that share a time keep their order.
    lines.sort_by_key(|line| line.at);
    Some(lines)
}

/// `mm:ss`, `mm:ss.x`, `mm:ss.xx` or `mm:ss.xxx`.
fn parse_time(tag: &str) -> Option<Duration> {
    let (minutes, rest) = tag.split_once(':')?;
    let minutes: u64 = minutes.trim().parse().ok()?;
    let (seconds, fraction) = match rest.split_once('.') {
        Some((seconds, fraction)) => (seconds, Some(fraction)),
        None => (rest, None),
    };
    let seconds: u64 = seconds.trim().parse().ok()?;
    let millis = match fraction {
        None => 0,
        Some(digits) => {
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let used: String = digits.chars().take(3).collect();
            let value: u64 = used.parse().ok()?;
            value * 10u64.pow(3 - used.len() as u32)
        }
    };
    Some(Duration::from_millis(
        minutes * 60_000 + seconds * 1000 + millis,
    ))
}

/// Lyrics from text: synced when it has LRC times, otherwise plain. `None` when it is blank.
pub fn lyrics_from_text(text: &str) -> Option<Lyrics> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    Some(match parse_lrc(text) {
        Some(lines) => Lyrics::Synced(lines),
        None => Lyrics::Plain(text.to_owned()),
    })
}

/// The `.lrc` file beside a track, if there is one. It overrides every other source.
pub fn sidecar_lyrics(track: &Path) -> Option<Lyrics> {
    let bytes = fs::read(track.with_extension("lrc")).ok()?;
    lyrics_from_text(&String::from_utf8_lossy(&bytes))
}

/// The best of several sources: the first synced one, otherwise the first plain one.
pub fn best_lyrics(candidates: impl IntoIterator<Item = Lyrics>) -> Option<Lyrics> {
    let mut plain = None;
    for candidate in candidates {
        if candidate.is_synced() {
            return Some(candidate);
        }
        plain.get_or_insert(candidate);
    }
    plain
}

/// What to look up: the track's title, artist, album and length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricsQuery {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration: Option<Duration>,
}

/// Lyrics found online for a track. Either or both may be present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteLyrics {
    pub synced: Option<String>,
    pub plain: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    track_name: String,
    artist_name: String,
    duration: f64,
    #[serde(default)]
    synced_lyrics: Option<String>,
    #[serde(default)]
    plain_lyrics: Option<String>,
}

const API: &str = "https://lrclib.net/api";
const USER_AGENT: &str = concat!("player/", env!("CARGO_PKG_VERSION"));
/// How far a result's length may differ from the file's, in seconds.
const LENGTH_TOLERANCE: f64 = 2.0;

/// Look up lyrics on LRCLIB. A result is used only when its title and artist match the track
/// (ignoring case, punctuation and bracketed suffixes such as "(Remastered)") and its length is
/// within two seconds of the file's. Tracks without a known length are never matched.
///
/// `Ok(None)` means LRCLIB has nothing that matches. `Err` means the lookup itself failed, and
/// should not be cached. This blocks, so call it off the UI thread.
pub fn fetch_lyrics(query: &LyricsQuery) -> std::result::Result<Option<RemoteLyrics>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent(USER_AGENT)
        .build()
        .into();

    let mut candidates = Vec::new();
    // An exact lookup first, since it names the recording precisely.
    let mut exact = agent
        .get(format!("{API}/get"))
        .query("track_name", &query.title)
        .query("artist_name", &query.artist);
    if let Some(album) = &query.album {
        exact = exact.query("album_name", album);
    }
    if let Some(duration) = query.duration {
        exact = exact.query("duration", duration.as_secs_f64().round().to_string());
    }
    let mut response = exact.call().map_err(|e| e.to_string())?;
    match response.status().as_u16() {
        200 => {
            let text = response
                .body_mut()
                .read_to_string()
                .map_err(|e| e.to_string())?;
            candidates.push(serde_json::from_str::<Record>(&text).map_err(|e| e.to_string())?);
        }
        404 => {}
        other => return Err(format!("LRCLIB answered with status {other}")),
    }

    // Otherwise, the search by title and artist, from which the length is checked.
    if !candidates.iter().any(|record| matches(record, query)) {
        let mut response = agent
            .get(format!("{API}/search"))
            .query("track_name", &query.title)
            .query("artist_name", &query.artist)
            .call()
            .map_err(|e| e.to_string())?;
        if response.status().as_u16() != 200 {
            return Err(format!(
                "LRCLIB search answered with status {}",
                response.status().as_u16()
            ));
        }
        let text = response
            .body_mut()
            .read_to_string()
            .map_err(|e| e.to_string())?;
        candidates.extend(serde_json::from_str::<Vec<Record>>(&text).map_err(|e| e.to_string())?);
    }

    let verified: Vec<&Record> = candidates
        .iter()
        .filter(|record| matches(record, query))
        .collect();
    let synced = verified
        .iter()
        .find_map(|record| non_empty(record.synced_lyrics.as_deref()));
    let plain = verified
        .iter()
        .find_map(|record| non_empty(record.plain_lyrics.as_deref()));
    if synced.is_none() && plain.is_none() {
        return Ok(None);
    }
    Ok(Some(RemoteLyrics {
        synced: synced.map(str::to_owned),
        plain: plain.map(str::to_owned),
    }))
}

fn non_empty(text: Option<&str>) -> Option<&str> {
    text.map(str::trim).filter(|text| !text.is_empty())
}

/// Whether a record describes the same recording as the query.
fn matches(record: &Record, query: &LyricsQuery) -> bool {
    let Some(duration) = query.duration else {
        return false;
    };
    normalise(&record.track_name) == normalise(&query.title)
        && normalise(&record.artist_name) == normalise(&query.artist)
        && (record.duration - duration.as_secs_f64()).abs() <= LENGTH_TOLERANCE
}

/// Lower-cased letters and digits only, with bracketed text such as "(Remastered)" removed.
fn normalise(text: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;
    for c in text.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => {
                out.extend(c.to_lowercase().filter(|l| l.is_alphanumeric()));
            }
            _ => {}
        }
    }
    out
}

impl Library {
    /// The cached lookup for a track. `None`: never looked up. `Some(None)`: looked up, nothing
    /// found. `Some(Some(lyrics))`: found.
    pub fn cached_lyrics(&self, media_id: i64) -> Result<Option<Option<Lyrics>>> {
        let row: Option<Option<String>> = self
            .conn
            .query_row(
                "SELECT text FROM lyrics WHERE media_id = ?1",
                [media_id],
                |r| r.get(0),
            )
            .optional()?;
        Ok(row.map(|text| text.as_deref().and_then(lyrics_from_text)))
    }

    /// Cache the result of a lookup. `None` records that nothing was found.
    pub fn store_lyrics(&self, media_id: i64, text: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO lyrics (media_id, text) VALUES (?1, ?2) \
             ON CONFLICT(media_id) DO UPDATE SET text = excluded.text, \
             fetched = strftime('%s', 'now')",
            rusqlite::params![media_id, text],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lrc_times_are_read_and_sorted() {
        let lines = parse_lrc("[ti:Song]\n[00:12.50]second\n[00:01.2][00:05]repeated\n").unwrap();
        let texts: Vec<_> = lines
            .iter()
            .map(|l| (l.at.as_millis(), l.text.as_str()))
            .collect();
        assert_eq!(
            texts,
            vec![(1200, "repeated"), (5000, "repeated"), (12500, "second")]
        );
        assert!(parse_lrc("just words\nno times").is_none());
    }

    #[test]
    fn text_without_times_is_plain() {
        assert_eq!(
            lyrics_from_text("  hello\nworld "),
            Some(Lyrics::Plain("hello\nworld".into()))
        );
        assert_eq!(lyrics_from_text("   "), None);
    }

    #[test]
    fn synced_beats_plain_whatever_the_order() {
        let plain = Lyrics::Plain("a".into());
        let synced = Lyrics::Synced(vec![]);
        assert_eq!(best_lyrics([plain.clone(), synced.clone()]), Some(synced));
        assert_eq!(best_lyrics([plain.clone()]), Some(plain));
        assert_eq!(best_lyrics([]), None);
    }

    #[test]
    fn titles_match_across_case_punctuation_and_brackets() {
        assert_eq!(
            normalise("Love Story (Taylor's Version)"),
            normalise("love story")
        );
        assert_eq!(normalise("Lamhe [DJ Mix]"), normalise("LAMHE"));
        assert_ne!(normalise("Love Story"), normalise("Love Stories"));
    }

    #[test]
    fn a_result_must_match_title_artist_and_length() {
        let query = LyricsQuery {
            title: "Love Story".into(),
            artist: "Taylor Swift".into(),
            album: None,
            duration: Some(Duration::from_secs(235)),
        };
        let record = |title: &str, artist: &str, duration: f64| Record {
            track_name: title.into(),
            artist_name: artist.into(),
            duration,
            synced_lyrics: None,
            plain_lyrics: None,
        };
        assert!(matches(
            &record("Love Story", "Taylor Swift", 236.0),
            &query
        ));
        assert!(!matches(
            &record("Love Story", "Taylor Swift", 250.0),
            &query
        ));
        assert!(!matches(
            &record("Love Story", "Someone Else", 235.0),
            &query
        ));
        let unknown_length = LyricsQuery {
            duration: None,
            ..query
        };
        assert!(!matches(
            &record("Love Story", "Taylor Swift", 235.0),
            &unknown_length
        ));
    }
}
