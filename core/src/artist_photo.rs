//! Artist photos from Wikidata. An artist is looked up by name. The first person or group on
//! Wikidata whose description says it makes music, and which has an image (property P18), gives
//! the photo. That image is a Wikimedia Commons file, and a copy scaled to 320 px is downloaded,
//! with its credit: the author and the licence, as Commons gives them.
//!
//! Photos and misses are cached in a folder, so each artist is looked up once. A failed lookup is
//! not cached, so it is tried again later.

use std::fs;
use std::path::Path;
use std::time::Duration;

const USER_AGENT: &str = concat!("player/", env!("CARGO_PKG_VERSION"));
const WIKIDATA: &str = "https://www.wikidata.org/w/api.php";
const COMMONS: &str = "https://commons.wikimedia.org/wiki/Special:FilePath";
const COMMONS_API: &str = "https://commons.wikimedia.org/w/api.php";
/// Width in pixels of the photo to download. Commons scales the original down to this.
const PHOTO_WIDTH: u32 = 320;
/// Words that, in a Wikidata description, say the entity is a musician or a music group.
const MUSICAL_WORDS: [&str; 12] = [
    "singer",
    "musician",
    "rapper",
    "band",
    "group",
    "composer",
    "songwriter",
    "producer",
    "dj",
    "vocalist",
    "pianist",
    "guitarist",
];

/// A photo of an artist, with its credit: "Author, licence, Wikimedia Commons".
pub struct ArtistPhoto {
    pub bytes: Vec<u8>,
    pub credit: String,
}

/// The photo of `artist`: from the cache in `dir` if it is there, otherwise from Wikidata. `None`
/// means there is no photo, or the lookup failed.
pub fn fetch_artist_photo(artist: &str, dir: &Path) -> Option<ArtistPhoto> {
    let key = format!("{:016x}", hash(artist));
    let photo_path = dir.join(format!("{key}.photo"));
    let credit_path = dir.join(format!("{key}.credit"));
    let miss_path = dir.join(format!("{key}.none"));
    if let (Ok(bytes), Ok(credit)) = (fs::read(&photo_path), fs::read_to_string(&credit_path)) {
        return Some(ArtistPhoto { bytes, credit });
    }
    if miss_path.exists() {
        return None;
    }
    match lookup(artist) {
        Ok(Some(photo)) => {
            let _ = fs::create_dir_all(dir);
            let _ = fs::write(&photo_path, &photo.bytes);
            let _ = fs::write(&credit_path, &photo.credit);
            Some(photo)
        }
        Ok(None) => {
            let _ = fs::create_dir_all(dir);
            let _ = fs::write(&miss_path, b"");
            None
        }
        Err(_) => None,
    }
}

/// Ask Wikidata for the artist's photo. `Ok(None)` means Wikidata has none. `Err` means the lookup
/// failed. This blocks, so call it off the UI thread.
fn lookup(artist: &str) -> Result<Option<ArtistPhoto>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent(USER_AGENT)
        .build()
        .into();
    let Some(id) = find_musician(&agent, artist)? else {
        return Ok(None);
    };
    let Some(file) = photo_file(&agent, &id)? else {
        return Ok(None);
    };
    let credit = credit(&agent, &file)?;
    let bytes = download(&agent, &file)?;
    Ok(Some(ArtistPhoto { bytes, credit }))
}

/// The first search result whose description says it is a musician or a music group.
fn find_musician(agent: &ureq::Agent, artist: &str) -> Result<Option<String>, String> {
    let json = get_json(
        agent
            .get(WIKIDATA)
            .query("action", "wbsearchentities")
            .query("search", artist)
            .query("language", "en")
            .query("type", "item")
            .query("limit", "5")
            .query("format", "json"),
    )?;
    let results = json["search"].as_array().cloned().unwrap_or_default();
    Ok(results.iter().find_map(|result| {
        let description = result["description"].as_str()?.to_lowercase();
        let musical = description
            .split(|c: char| !c.is_alphanumeric())
            .any(|word| MUSICAL_WORDS.contains(&word));
        if musical {
            result["id"].as_str().map(str::to_owned)
        } else {
            None
        }
    }))
}

/// The name of the Commons file that is the entity's image (property P18), if it has one.
fn photo_file(agent: &ureq::Agent, id: &str) -> Result<Option<String>, String> {
    let json = get_json(
        agent
            .get(WIKIDATA)
            .query("action", "wbgetclaims")
            .query("entity", id)
            .query("property", "P18")
            .query("format", "json"),
    )?;
    Ok(json["claims"]["P18"][0]["mainsnak"]["datavalue"]["value"]
        .as_str()
        .map(str::to_owned))
}

/// The credit of a Commons file: its author and licence, from the file's metadata.
fn credit(agent: &ureq::Agent, file: &str) -> Result<String, String> {
    let json = get_json(
        agent
            .get(COMMONS_API)
            .query("action", "query")
            .query("titles", format!("File:{file}"))
            .query("prop", "imageinfo")
            .query("iiprop", "extmetadata")
            .query("format", "json"),
    )?;
    let page = json["query"]["pages"]
        .as_object()
        .and_then(|pages| pages.values().next())
        .ok_or("no page for the photo")?;
    let metadata = &page["imageinfo"][0]["extmetadata"];
    let field = |name: &str| {
        metadata[name]["value"]
            .as_str()
            .map(strip_tags)
            .unwrap_or_default()
    };
    let author = field("Artist");
    let author = if author.is_empty() {
        "Unknown author".to_owned()
    } else {
        author
    };
    let licence = field("LicenseShortName");
    Ok(if licence.is_empty() {
        format!("{author}, Wikimedia Commons")
    } else {
        format!("{author}, {licence}, Wikimedia Commons")
    })
}

/// Download a scaled copy of a Commons file.
fn download(agent: &ureq::Agent, file: &str) -> Result<Vec<u8>, String> {
    let url = format!("{COMMONS}/{}?width={PHOTO_WIDTH}", percent_encode(file));
    let mut response = agent.get(url).call().map_err(|e| e.to_string())?;
    if response.status().as_u16() != 200 {
        return Err(format!("status {}", response.status().as_u16()));
    }
    response.body_mut().read_to_vec().map_err(|e| e.to_string())
}

/// Send a request and read its body as JSON.
fn get_json(
    request: ureq::RequestBuilder<ureq::typestate::WithoutBody>,
) -> Result<serde_json::Value, String> {
    let mut response = request.call().map_err(|e| e.to_string())?;
    if response.status().as_u16() != 200 {
        return Err(format!("status {}", response.status().as_u16()));
    }
    let text = response
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// Plain text from a piece of HTML: tags removed, the `&amp;` entity read, spaces tidied.
fn strip_tags(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    text.replace("&amp;", "&")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Escape a file name for a URL path: letters, digits and `-_.~` stay, everything else is `%XX`.
fn percent_encode(name: &str) -> String {
    name.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// A stable hash of the artist's name, ignoring case, for the cache file names.
fn hash(artist: &str) -> u64 {
    artist
        .to_lowercase()
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |h, b| {
            (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
        })
}
