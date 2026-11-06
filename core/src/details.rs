//! What a file says about itself: every tag, and its lyrics.

use std::path::Path;

use lofty::file::{TaggedFile, TaggedFileExt};
use lofty::tag::{ItemKey, TagType};
use serde::{Deserialize, Serialize};

/// Everything in a file's tags, for the track info panel.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackDetails {
    /// Each text tag, in file order. Binary items such as pictures are left out.
    pub tags: Vec<TagLine>,
    /// Unsynchronised lyrics, if the file has any.
    pub lyrics: Option<String>,
    /// Number of embedded pictures.
    pub pictures: usize,
}

/// One text tag of a file: its name, its value, and whether the value can be changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagLine {
    /// The tag's name, such as `TrackTitle`.
    pub name: String,
    pub value: String,
    /// Whether the value can be written back. It can when the tag holds only this one value
    /// for its name, so writing it replaces nothing else.
    pub editable: bool,
}

/// A text item of a file, with the tag and key it belongs to.
pub(crate) struct TextItem {
    pub tag_type: TagType,
    pub key: ItemKey,
    pub line: TagLine,
}

/// The text items of a file, in the order the track info panel lists them.
pub(crate) fn text_items(tagged: &TaggedFile) -> Vec<TextItem> {
    let mut items = Vec::new();
    for tag in tagged.tags() {
        for item in tag.items() {
            let Some(text) = item.value().text() else {
                continue;
            };
            let text = text.trim();
            if text.is_empty() {
                continue;
            }
            let same_key = tag
                .items()
                .filter(|other| other.key() == item.key() && other.value().text().is_some())
                .count();
            items.push(TextItem {
                tag_type: tag.tag_type(),
                key: item.key(),
                line: TagLine {
                    name: format!("{:?}", item.key()),
                    value: text.to_owned(),
                    editable: same_key == 1,
                },
            });
        }
    }
    items
}

/// Read the tags of the file at `path`. `None` when the file can't be read.
pub fn track_details(path: &Path) -> Option<TrackDetails> {
    let tagged = lofty::read_from_path(path).ok()?;
    let mut details = TrackDetails {
        tags: text_items(&tagged)
            .into_iter()
            .map(|item| item.line)
            .collect(),
        ..TrackDetails::default()
    };
    for tag in tagged.tags() {
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
