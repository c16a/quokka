//! What a file says about itself: every tag, and its lyrics.

use std::path::Path;

use lofty::file::TaggedFileExt;
use lofty::tag::ItemKey;

/// Everything in a file's tags, for the track info panel.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackDetails {
    /// Each tag item as `(name, value)`, in file order. Binary items such as pictures are left out.
    pub tags: Vec<(String, String)>,
    /// Unsynchronised lyrics, if the file has any.
    pub lyrics: Option<String>,
    /// Number of embedded pictures.
    pub pictures: usize,
}

/// Read the tags of the file at `path`. `None` when the file can't be read.
pub fn track_details(path: &Path) -> Option<TrackDetails> {
    let tagged = lofty::read_from_path(path).ok()?;
    let mut details = TrackDetails::default();
    for tag in tagged.tags() {
        for item in tag.items() {
            if let Some(text) = item.value().text() {
                let text = text.trim();
                if !text.is_empty() {
                    details
                        .tags
                        .push((format!("{:?}", item.key()), text.to_owned()));
                }
            }
        }
        details.pictures += tag.pictures().len();
    }
    let primary = tagged.primary_tag().or_else(|| tagged.first_tag());
    details.lyrics = primary
        .and_then(|tag| {
            tag.get_string(ItemKey::UnsyncLyrics)
                .or_else(|| tag.get_string(ItemKey::Lyrics))
        })
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty());
    Some(details)
}
