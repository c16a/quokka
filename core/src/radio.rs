//! Internet radio: the station index from radio-browser.info, the stations the user has tuned
//! in to, and the sync that refreshes the index.

use std::time::Duration;

use rusqlite::{OptionalExtension, params};
use serde_json::Value;

use crate::{Library, Result};

/// The radio-browser.info name that resolves to its servers. Its address is shared by several
/// servers, so a retry can reach a different one.
const SEARCH_URL: &str = "https://all.api.radio-browser.info/json/stations/search";
/// Stations per request. A single request for the whole directory is tens of megabytes and
/// times out, so the list is fetched in pages.
const PAGE_SIZE: usize = 5_000;
const PAGE_ATTEMPTS: u32 = 3;
/// Pages fetched at the same time. Kept small so the directory isn't flooded.
const PARALLEL_PAGES: usize = 4;
const USER_AGENT: &str = concat!("player/", env!("CARGO_PKG_VERSION"));

/// One radio station. `url` is the stream address to play.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Station {
    pub uuid: String,
    pub name: String,
    pub url: String,
    pub country: String,
    pub codec: String,
    pub bitrate: i64,
    pub tags: String,
    pub votes: i64,
    /// Whether the directory's last check found the stream working.
    pub working: bool,
}

/// Download the whole station index. Pages are fetched a few at a time, in parallel, and kept in
/// order. Each page is retried a few times before the download is given up. Blocks, so call it
/// off the UI thread.
pub fn fetch_all_stations() -> std::result::Result<Vec<Station>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(120)))
        .user_agent(USER_AGENT)
        .build()
        .into();
    let mut stations = Vec::new();
    let mut first_offset = 0;
    loop {
        let offsets: Vec<usize> = (0..PARALLEL_PAGES)
            .map(|i| first_offset + i * PAGE_SIZE)
            .collect();
        let pages: Vec<std::result::Result<Vec<Station>, String>> = std::thread::scope(|scope| {
            let handles: Vec<_> = offsets
                .iter()
                .map(|&offset| {
                    let agent = &agent;
                    scope.spawn(move || fetch_page(agent, offset))
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .unwrap_or_else(|_| Err("a download thread stopped".to_owned()))
                })
                .collect()
        });
        // A short page is the last one. Anything fetched after it is empty, so it is dropped.
        for page in pages {
            let page = page?;
            let count = page.len();
            stations.extend(page);
            if count < PAGE_SIZE {
                return Ok(stations);
            }
        }
        first_offset += PARALLEL_PAGES * PAGE_SIZE;
    }
}

fn fetch_page(agent: &ureq::Agent, offset: usize) -> std::result::Result<Vec<Station>, String> {
    let mut last_error = String::new();
    for attempt in 1..=PAGE_ATTEMPTS {
        let result = agent
            .get(SEARCH_URL)
            .query("limit", PAGE_SIZE.to_string())
            .query("offset", offset.to_string())
            .query("hidebroken", "true")
            .query("order", "stationuuid")
            .call()
            .map_err(|e| e.to_string())
            .and_then(|mut response| {
                response
                    .body_mut()
                    .read_to_string()
                    .map_err(|e| e.to_string())
            })
            .and_then(|json| parse_stations(&json));
        match result {
            Ok(page) => return Ok(page),
            Err(e) => {
                last_error = e;
                if attempt < PAGE_ATTEMPTS {
                    std::thread::sleep(Duration::from_secs(2 * u64::from(attempt)));
                }
            }
        }
    }
    Err(format!(
        "the station list download failed at {offset}: {last_error}"
    ))
}

/// Read the station list. Fields are read leniently, because the directory has sent numbers as
/// text and flags as numbers or booleans. Entries without a uuid or a stream address are skipped.
pub fn parse_stations(json: &str) -> std::result::Result<Vec<Station>, String> {
    let entries: Vec<Value> = serde_json::from_str(json).map_err(|e| e.to_string())?;
    Ok(entries.iter().filter_map(station_from).collect())
}

fn station_from(entry: &Value) -> Option<Station> {
    let uuid = text(entry, "stationuuid");
    let resolved = text(entry, "url_resolved");
    let url = if resolved.is_empty() {
        text(entry, "url")
    } else {
        resolved
    };
    if uuid.is_empty() || url.is_empty() {
        return None;
    }
    Some(Station {
        uuid,
        name: text(entry, "name"),
        url,
        country: text(entry, "country"),
        codec: text(entry, "codec"),
        bitrate: number(entry, "bitrate"),
        tags: text(entry, "tags"),
        votes: number(entry, "votes"),
        working: number(entry, "lastcheckok") != 0,
    })
}

fn text(entry: &Value, key: &str) -> String {
    match entry.get(key) {
        Some(Value::String(s)) => s.trim().to_owned(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

fn number(entry: &Value, key: &str) -> i64 {
    match entry.get(key) {
        Some(Value::Number(n)) => n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f as i64))
            .unwrap_or(0),
        Some(Value::String(s)) => s.trim().parse().unwrap_or(0),
        Some(Value::Bool(b)) => i64::from(*b),
        _ => 0,
    }
}

/// A station the user has tuned in to. Its stream address is kept here, so it plays even when
/// the station is missing from the index.
fn tuned_from(row: &rusqlite::Row<'_>) -> rusqlite::Result<Station> {
    Ok(Station {
        uuid: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        country: row.get(3)?,
        codec: row.get(4)?,
        bitrate: row.get(5)?,
        tags: String::new(),
        votes: 0,
        working: true,
    })
}

/// Words typed by the user, as an FTS query: each word must match, and the last one may match
/// as a prefix. Quotes are escaped. `None` when there are no words.
fn fts_query(text: &str) -> Option<String> {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|word| format!("\"{}\"", word.replace('"', "\"\"")))
        .collect();
    if words.is_empty() {
        return None;
    }
    let last = words.len() - 1;
    Some(
        words
            .into_iter()
            .enumerate()
            .map(|(i, word)| if i == last { format!("{word}*") } else { word })
            .collect::<Vec<_>>()
            .join(" "),
    )
}

impl Library {
    /// Replace the station index with `stations`. Returns how many were stored.
    pub fn replace_stations(&mut self, stations: &[Station]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM station_fts", [])?;
        tx.execute("DELETE FROM station", [])?;
        for s in stations {
            tx.execute(
                "INSERT OR REPLACE INTO station \
                 (uuid, name, url, country, codec, bitrate, tags, votes, working) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    s.uuid,
                    s.name,
                    s.url,
                    s.country,
                    s.codec,
                    s.bitrate,
                    s.tags,
                    s.votes,
                    i64::from(s.working)
                ],
            )?;
            tx.execute(
                "INSERT INTO station_fts (uuid, name, tags, country) VALUES (?1, ?2, ?3, ?4)",
                params![s.uuid, s.name, s.tags, s.country],
            )?;
        }
        // Tuned-in stations take the fresh name, address and details from the new index. Ones
        // that have left the index keep the copy they were tuned in with.
        tx.execute(
            "UPDATE tuned SET \
               name    = (SELECT s.name FROM station s WHERE s.uuid = tuned.uuid), \
               url     = (SELECT s.url FROM station s WHERE s.uuid = tuned.uuid), \
               country = (SELECT s.country FROM station s WHERE s.uuid = tuned.uuid), \
               codec   = (SELECT s.codec FROM station s WHERE s.uuid = tuned.uuid), \
               bitrate = (SELECT s.bitrate FROM station s WHERE s.uuid = tuned.uuid) \
             WHERE uuid IN (SELECT uuid FROM station)",
            [],
        )?;
        tx.commit()?;
        Ok(stations.len())
    }

    /// Working stations matching `query`, most popular first. An empty query lists the most
    /// popular stations.
    pub fn search_stations(&self, query: &str, limit: usize) -> Result<Vec<Station>> {
        let columns = "s.uuid, s.name, s.url, s.country, s.codec, s.bitrate, s.tags, s.votes";
        let rows: Vec<Station> = match fts_query(query) {
            Some(fts) => {
                let sql = format!(
                    "SELECT {columns}, s.working FROM station_fts f \
                     JOIN station s ON s.uuid = f.uuid \
                     WHERE station_fts MATCH ?1 AND s.working = 1 \
                     ORDER BY s.votes DESC LIMIT ?2"
                );
                let mut stmt = self.conn.prepare(&sql)?;
                let rows = stmt.query_map(params![fts, limit as i64], station_row)?;
                rows.collect::<rusqlite::Result<_>>()?
            }
            None => {
                let sql = format!(
                    "SELECT {columns}, s.working FROM station s \
                     WHERE s.working = 1 ORDER BY s.votes DESC LIMIT ?1"
                );
                let mut stmt = self.conn.prepare(&sql)?;
                let rows = stmt.query_map([limit as i64], station_row)?;
                rows.collect::<rusqlite::Result<_>>()?
            }
        };
        Ok(rows)
    }

    /// How many stations are in the index.
    pub fn station_count(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM station", [], |r| r.get(0))?)
    }

    /// Add a station to the tuned-in list. Tuning in again keeps its place.
    pub fn tune_in(&self, station: &Station) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tuned (uuid, name, url, country, codec, bitrate) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                station.uuid,
                station.name,
                station.url,
                station.country,
                station.codec,
                station.bitrate
            ],
        )?;
        Ok(())
    }

    pub fn tune_out(&self, uuid: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM tuned WHERE uuid = ?1", [uuid])?;
        Ok(())
    }

    /// Stations the user has tuned in to, in the order they were added.
    pub fn tuned_stations(&self) -> Result<Vec<Station>> {
        let mut stmt = self.conn.prepare(
            "SELECT uuid, name, url, country, codec, bitrate FROM tuned ORDER BY added, name",
        )?;
        let rows = stmt.query_map([], tuned_from)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Whether a station is in the tuned-in list.
    pub fn is_tuned(&self, uuid: &str) -> Result<bool> {
        let found: Option<i64> = self
            .conn
            .query_row("SELECT 1 FROM tuned WHERE uuid = ?1", [uuid], |r| r.get(0))
            .optional()?;
        Ok(found.is_some())
    }

    /// Store a small piece of app state, such as when the station index was last synced.
    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn meta(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
            .optional()?)
    }
}

fn station_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Station> {
    Ok(Station {
        uuid: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        country: row.get(3)?,
        codec: row.get(4)?,
        bitrate: row.get(5)?,
        tags: row.get(6)?,
        votes: row.get(7)?,
        working: row.get::<_, i64>(8)? != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"[
        {"stationuuid":"a1","name":"Radio One","url":"http://old/1","url_resolved":"http://one/stream",
         "country":"India","codec":"MP3","bitrate":128,"tags":"hindi,film","votes":900,"lastcheckok":1},
        {"stationuuid":"b2","name":"Sur Tamil","url":"http://two/","url_resolved":"",
         "country":"India","codec":"AAC","bitrate":"64","tags":"tamil","votes":"40","lastcheckok":true},
        {"stationuuid":"c3","name":"Dead Station","url":"http://dead/","url_resolved":"http://dead/s",
         "country":"Kenya","codec":"MP3","bitrate":32,"tags":"","votes":500,"lastcheckok":0},
        {"name":"No uuid","url":"http://nouuid/"}
    ]"#;

    #[test]
    fn stations_are_read_leniently_and_incomplete_entries_skipped() {
        let stations = parse_stations(SAMPLE).unwrap();
        assert_eq!(stations.len(), 3);
        assert_eq!(
            stations[0].url, "http://one/stream",
            "resolved address is preferred"
        );
        assert_eq!(
            stations[1].url, "http://two/",
            "falls back to the plain address"
        );
        assert_eq!(stations[1].bitrate, 64);
        assert!(stations[1].working);
        assert!(!stations[2].working);
    }

    #[test]
    fn search_finds_working_stations_and_tuning_round_trips() {
        let mut lib = Library::open_in_memory().unwrap();
        let stations = parse_stations(SAMPLE).unwrap();
        assert_eq!(lib.replace_stations(&stations).unwrap(), 3);
        assert_eq!(lib.station_count().unwrap(), 3);

        let found = lib.search_stations("tam", 10).unwrap();
        assert_eq!(
            found.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            vec!["Sur Tamil"]
        );
        assert!(
            lib.search_stations("kenya", 10).unwrap().is_empty(),
            "dead stations are hidden"
        );
        let popular = lib.search_stations("", 10).unwrap();
        assert_eq!(
            popular[0].name, "Radio One",
            "most votes first among working stations"
        );

        lib.tune_in(&stations[0]).unwrap();
        lib.tune_in(&stations[1]).unwrap();
        lib.tune_in(&stations[0]).unwrap();
        assert_eq!(
            lib.tuned_stations().unwrap().len(),
            2,
            "tuning in twice keeps one entry"
        );
        assert!(lib.is_tuned("a1").unwrap());
        lib.tune_out("a1").unwrap();
        assert!(!lib.is_tuned("a1").unwrap());
    }

    #[test]
    fn tuned_stations_keep_playing_after_leaving_the_index() {
        let mut lib = Library::open_in_memory().unwrap();
        let stations = parse_stations(SAMPLE).unwrap();
        lib.replace_stations(&stations).unwrap();
        lib.tune_in(&stations[0]).unwrap();
        lib.replace_stations(&[]).unwrap();
        let tuned = lib.tuned_stations().unwrap();
        assert_eq!(tuned[0].url, "http://one/stream");
    }

    #[test]
    fn a_sync_updates_the_address_of_tuned_stations() {
        let mut lib = Library::open_in_memory().unwrap();
        let stations = parse_stations(SAMPLE).unwrap();
        lib.replace_stations(&stations).unwrap();
        lib.tune_in(&stations[0]).unwrap();

        let mut moved = stations.clone();
        moved[0].url = "http://new-address/stream".into();
        moved[0].name = "Radio One HD".into();
        lib.replace_stations(&moved).unwrap();

        let tuned = lib.tuned_stations().unwrap();
        assert_eq!(tuned[0].url, "http://new-address/stream");
        assert_eq!(tuned[0].name, "Radio One HD");
    }

    #[test]
    fn sync_time_is_stored_as_meta() {
        let lib = Library::open_in_memory().unwrap();
        assert_eq!(lib.meta("stations_synced").unwrap(), None);
        lib.set_meta("stations_synced", "1700000000").unwrap();
        assert_eq!(
            lib.meta("stations_synced").unwrap().as_deref(),
            Some("1700000000")
        );
    }
}
