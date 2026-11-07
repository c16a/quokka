use std::path::Path;

use lofty::file::{AudioFile, FileType, TaggedFileExt};
use lofty::tag::{Accessor, ItemKey};
use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::video::{VideoCodecId, VideoCodecParameters};
use symphonia::core::common::FourCc;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, Track};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

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
    /// The picture's stream, for video files.
    pub video: VideoInfo,
}

/// The first video track of a container, as the movie page shows it. Colour depth and chroma
/// subsampling come from the codec's processing step, not from here.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct VideoInfo {
    pub codec: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub duration_ms: Option<i64>,
}

/// Read the first video track from the container at `path`. Formats symphonia cannot
/// read, such as raw OBU streams, give nothing.
pub(crate) fn probe_video(path: &Path) -> VideoInfo {
    let format = probe_format(path);
    let Some((track, params)) = format.as_deref().and_then(first_video_track) else {
        // Symphonia gives no video track for some MP4 codecs (HEVC, for one). The sample entry
        // still names the codec.
        return VideoInfo {
            codec: mp4_codec(path),
            ..VideoInfo::default()
        };
    };
    let av1 = av1_config(path, params);
    VideoInfo {
        codec: video_codec_label(params, av1.is_some()).or_else(|| mp4_codec(path)),
        width: params.width.map(i64::from),
        height: params.height.map(i64::from),
        duration_ms: track
            .time_base
            .zip(track.duration)
            .and_then(|(base, duration)| base.calc_duration(duration))
            .and_then(|time| i64::try_from(time.as_millis()).ok()),
    }
}

fn probe_format(path: &Path) -> Option<Box<dyn FormatReader>> {
    let file = std::fs::File::open(path).ok()?;
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    symphonia::default::get_probe()
        .probe(
            &hint,
            stream,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .ok()
}

fn first_video_track(format: &dyn FormatReader) -> Option<(&Track, &VideoCodecParameters)> {
    format
        .tracks()
        .iter()
        .find_map(|track| match &track.codec_params {
            Some(CodecParameters::Video(params)) => Some((track, params)),
            _ => None,
        })
}

/// The name of a video codec the library knows, or `None`. Symphonia does not name AV1 in MP4
/// files, so an AV1 configuration record also counts as AV1.
fn video_codec_label(params: &VideoCodecParameters, is_av1: bool) -> Option<String> {
    let names = [
        (*b"avc1", "H.264"),
        (*b"hvc1", "HEVC"),
        (*b"hev1", "HEVC"),
        (*b"vp09", "VP9"),
    ];
    if let Some((_, name)) = names
        .iter()
        .find(|(fourcc, _)| params.codec == VideoCodecId::from(FourCc::new(*fourcc)))
    {
        return Some((*name).to_owned());
    }
    is_av1.then(|| "AV1".to_owned())
}

/// The AV1 codec configuration record (`av1C`) of the first video track, from the container's
/// extra data or, for MP4 files, from the `moov` box.
fn av1_config(path: &Path, params: &VideoCodecParameters) -> Option<Vec<u8>> {
    params
        .extra_data
        .iter()
        .map(|extra| &*extra.data)
        .find(|data| is_av1_config(data))
        .map(<[u8]>::to_vec)
        .or_else(|| match path.extension().and_then(|e| e.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("mp4") || ext.eq_ignore_ascii_case("m4v") => {
                mp4_av1_config(path)
            }
            _ => None,
        })
}

/// Whether `data` looks like an AV1 configuration record: version 1 with its marker bit set.
fn is_av1_config(data: &[u8]) -> bool {
    data.len() >= 4 && data[0] == 0x81
}

/// The codec of an MP4 file's sample entry, for the codecs symphonia does not name: H.264 and
/// HEVC. `None` for other files, and for MP4 files without these.
fn mp4_codec(path: &Path) -> Option<String> {
    let ext = path.extension().and_then(|e| e.to_str())?;
    if !(ext.eq_ignore_ascii_case("mp4") || ext.eq_ignore_ascii_case("m4v")) {
        return None;
    }
    let mut file = std::fs::File::open(path).ok()?;
    let moov = read_top_level_box(&mut file, b"moov")?;
    [
        (b"avc1", "H.264"),
        (b"avc3", "H.264"),
        (b"hvc1", "HEVC"),
        (b"hev1", "HEVC"),
    ]
    .iter()
    .find(|(fourcc, _)| moov.windows(4).any(|window| window == *fourcc))
    .map(|(_, name)| (*name).to_owned())
}

/// The `av1C` box inside the `moov` box of an MP4 file, if it has an AV1 track.
fn mp4_av1_config(path: &Path) -> Option<Vec<u8>> {
    mp4_config_after(path, b"av01", b"av1C").filter(|body| is_av1_config(body))
}

/// The body of the box named `config` that follows the first `entry` sample entry inside the
/// `moov` box of an MP4 file, such as `av1C` after `av01`, or `avcC` after `avc1`.
pub(crate) fn mp4_config_after(path: &Path, entry: &[u8; 4], config: &[u8; 4]) -> Option<Vec<u8>> {
    let mut file = std::fs::File::open(path).ok()?;
    let moov = read_top_level_box(&mut file, b"moov")?;
    let sample_entry = moov.windows(4).position(|w| w == entry)?;
    let at = sample_entry + moov[sample_entry..].windows(4).position(|w| w == config)?;
    let start = at.checked_sub(4)?;
    let size = usize::try_from(u32::from_be_bytes(moov.get(start..at)?.try_into().ok()?)).ok()?;
    moov.get(at + 4..start.checked_add(size)?)
        .map(<[u8]>::to_vec)
}

/// The body of the first top-level box of type `wanted`, or `None` when there is none or it is
/// too large to read into memory.
fn read_top_level_box(file: &mut std::fs::File, wanted: &[u8; 4]) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};

    /// Largest box read: a `moov` box is kilobytes to a few megabytes.
    const MAX_BOX: u64 = 64 * 1024 * 1024;

    let end = file.metadata().ok()?.len();
    let mut offset = 0u64;
    while offset.checked_add(8)? <= end {
        file.seek(SeekFrom::Start(offset)).ok()?;
        let mut header = [0u8; 16];
        let read = file.read(&mut header).ok()?;
        if read < 8 {
            return None;
        }
        let (size, header_len) =
            match u32::from_be_bytes([header[0], header[1], header[2], header[3]]) {
                1 if read == 16 => (u64::from_be_bytes(header[8..16].try_into().ok()?), 16),
                0 => (end - offset, 8),
                size => (u64::from(size), 8),
            };
        if size < header_len {
            return None;
        }
        if &header[4..8] == wanted {
            let body_len = size - header_len;
            if body_len > MAX_BOX {
                return None;
            }
            file.seek(SeekFrom::Start(offset + header_len)).ok()?;
            let mut body = vec![0u8; usize::try_from(body_len).ok()?];
            file.read_exact(&mut body).ok()?;
            return Some(body);
        }
        offset = offset.checked_add(size)?;
    }
    None
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
