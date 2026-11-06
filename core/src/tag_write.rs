//! Writing a file's text tags. Only the values the user changed are written, and only to tags
//! that hold one value for their name, so nothing else in the file is touched.

use std::path::Path;

use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{ItemValue, TagItem};

use crate::details::{TagLine, text_items};
use crate::{Error, Result};

/// Write `values` into the text tags of the file at `path`. `expected` must be the tag lines the
/// file had when it was read, in the same order. If the file has changed since, nothing is written.
pub(crate) fn write_tags(path: &Path, expected: &[TagLine], values: &[String]) -> Result<()> {
    let mut tagged = lofty::read_from_path(path).map_err(|e| Error::Tag(e.to_string()))?;
    let current = text_items(&tagged);
    let same_lines = current.len() == expected.len()
        && values.len() == expected.len()
        && current
            .iter()
            .zip(expected)
            .all(|(item, line)| item.line.name == line.name);
    if !same_lines {
        return Err(Error::TagsChanged);
    }

    let mut writes = Vec::new();
    for ((item, line), value) in current.iter().zip(expected).zip(values) {
        if value == &line.value {
            continue;
        }
        if !line.editable {
            return Err(Error::TagNotEditable(line.name.clone()));
        }
        writes.push((item.tag_type, item.key, value.clone()));
    }
    if writes.is_empty() {
        return Ok(());
    }

    // Each tag is rebuilt in the order it had, with only the edited value changed. Replacing an
    // item in place would move it to the end, and the lines would no longer match the panel.
    for (tag_type, key, value) in writes {
        let tag = tagged
            .tag_mut(tag_type)
            .ok_or_else(|| Error::Tag(format!("the file has no {tag_type:?} tag")))?;
        let items: Vec<TagItem> = tag
            .items()
            .cloned()
            .map(|item| {
                if item.key() == key {
                    TagItem::new(key, ItemValue::Text(value.clone()))
                } else {
                    item
                }
            })
            .collect();
        tag.retain(|_| false);
        for item in items {
            tag.push_unchecked(item);
        }
    }
    tagged
        .save_to_path(path, WriteOptions::default())
        .map_err(|e| Error::Tag(e.to_string()))
}
