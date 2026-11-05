//! Embedded cover art, read with lofty.

use std::path::Path;

use lofty::file::TaggedFileExt;
use lofty::picture::PictureType;

/// The front cover embedded in the audio file at `path`, or the first picture when there is
/// no front cover. `None` when the file has no pictures or cannot be read.
pub fn read_cover(path: &Path) -> Option<Vec<u8>> {
    let tagged = lofty::read_from_path(path).ok()?;
    let pictures: Vec<_> = tagged
        .tags()
        .iter()
        .flat_map(|tag| tag.pictures())
        .collect();
    pictures
        .iter()
        .find(|p| matches!(p.pic_type(), PictureType::CoverFront))
        .or(pictures.first())
        .map(|p| p.data().to_vec())
}
