use std::path::Path;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, ItemKey};

/// Metadata read from a file. Every field is optional: files that lofty cannot
/// parse (common for video) are still indexed, just without tags.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Metadata {
    pub duration_ms: Option<i64>,
    pub title: Option<String>,
    /// Track artist(s), the performer of this recording.
    pub artist: Option<String>,
    /// Album artist, the artist credited for the whole album.
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub composer: Option<String>,
    pub genre: Option<String>,
    pub track: Option<i64>,
    pub year: Option<i64>,
}

pub(crate) fn read(path: &Path) -> Metadata {
    let Ok(tagged) = lofty::read_from_path(path) else {
        return Metadata::default();
    };

    let mut meta = Metadata {
        duration_ms: i64::try_from(tagged.properties().duration().as_millis()).ok(),
        ..Metadata::default()
    };

    if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
        meta.title = clean(tag.title().as_deref());
        meta.artist = clean(tag.artist().as_deref());
        meta.album = clean(tag.album().as_deref());
        meta.genre = clean(tag.genre().as_deref());
        meta.album_artist = clean(tag.get_string(ItemKey::AlbumArtist));
        meta.composer = clean(tag.get_string(ItemKey::Composer));
        meta.track = tag.track().map(i64::from);
        meta.year = tag.date().map(|t| i64::from(t.year));
    }

    meta
}

/// Trim a tag value and drop it if nothing is left.
fn clean(value: Option<&str>) -> Option<String> {
    value.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
}
