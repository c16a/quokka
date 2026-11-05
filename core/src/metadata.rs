use std::path::Path;

use lofty::file::{AudioFile, FileType, TaggedFileExt};
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
    /// Codec label, such as "FLAC" or "MP3".
    pub codec: Option<String>,
    pub bit_depth: Option<i64>,
    pub sample_rate: Option<i64>,
    pub channels: Option<i64>,
    /// Average bitrate in kbps: the file's own figure, or size over duration.
    pub bitrate_kbps: Option<i64>,
}

/// Read tags and stream properties. `size` is the file size in bytes, used to
/// estimate the bitrate when the format does not report one.
pub(crate) fn read(path: &Path, size: i64) -> Metadata {
    let Ok(tagged) = lofty::read_from_path(path) else {
        return Metadata::default();
    };

    let properties = tagged.properties();
    let duration_ms = i64::try_from(properties.duration().as_millis()).ok();
    let mut meta = Metadata {
        duration_ms,
        codec: Some(codec_label(tagged.file_type())),
        bit_depth: properties.bit_depth().map(i64::from),
        sample_rate: properties.sample_rate().map(i64::from),
        channels: properties.channels().map(i64::from),
        bitrate_kbps: properties
            .audio_bitrate()
            .or(properties.overall_bitrate())
            .map(i64::from)
            .or_else(|| estimate_kbps(size, duration_ms)),
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

/// Average kbps from size in bytes and duration in milliseconds (bits per millisecond).
fn estimate_kbps(size: i64, duration_ms: Option<i64>) -> Option<i64> {
    let duration_ms = duration_ms.filter(|&ms| ms > 0)?;
    Some(size.saturating_mul(8) / duration_ms)
}

fn codec_label(file_type: FileType) -> String {
    match file_type {
        FileType::Flac => "FLAC".into(),
        FileType::Mpeg => "MP3".into(),
        FileType::Aac => "AAC".into(),
        FileType::Mp4 => "MP4".into(),
        FileType::Vorbis => "Vorbis".into(),
        FileType::Opus => "Opus".into(),
        FileType::Wav => "WAV".into(),
        FileType::Aiff => "AIFF".into(),
        FileType::Ape => "APE".into(),
        FileType::WavPack => "WavPack".into(),
        FileType::Mpc => "Musepack".into(),
        FileType::Speex => "Speex".into(),
        other => format!("{other:?}"),
    }
}

/// Trim a tag value and drop it if nothing is left.
fn clean(value: Option<&str>) -> Option<String> {
    value.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
}
