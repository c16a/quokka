//! The compressed frames of an MP4 file's first video track, for decoders. Each sample is one
//! frame in the codec's own packet format: length-prefixed NAL units for H.264 and HEVC.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use super::Codec;
use super::audio::{AudioCodec, AudioTrack, aac_format, ac3_from_dac3, descriptor, eac3_from_dec3};
use super::bmff::{be, boxes, child, find_top, full_box};
use super::ebml::{elements, text, uint, vint};
use super::mp4::{codec_of, handler, sample_entry, timescale};
use crate::Result;

/// One compressed frame of a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample {
    /// Where the frame starts in the file.
    pub offset: u64,
    /// The frame's size in bytes.
    pub size: u32,
    /// The decode time, in ticks of the track's time scale.
    pub decode_time: i64,
    /// The presentation time, in the same ticks.
    pub presentation_time: i64,
    /// Whether the frame decodes without the frames before it.
    pub sync: bool,
}

/// The first video track of an MP4 file, as decoders need it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleTrack {
    pub codec: Codec,
    pub width: u32,
    pub height: u32,
    /// The codec's configuration record: the `avcC` or `hvcC` box body.
    pub config: Vec<u8>,
    /// Ticks per second for the times in `samples`.
    pub timescale: u64,
    pub samples: Vec<Sample>,
}

/// The samples of the first video track of an MP4, Matroska or WebM file. `None` for other files,
/// and for codecs whose samples this does not yet read.
pub fn read_samples(path: &Path) -> Result<Option<SampleTrack>> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    let is_ebml = file.read(&mut magic)? == 4 && magic == super::EBML_MAGIC;
    file.seek(SeekFrom::Start(0))?;
    if is_ebml {
        return Ok(super::matroska::read_samples(&mut file)?);
    }
    let Some(moov) = find_top(&mut file, b"moov")? else {
        return Ok(None);
    };
    Ok(track(&moov))
}

fn track(moov: &[u8]) -> Option<SampleTrack> {
    let trak = boxes(moov)
        .filter(|(kind, _)| kind == b"trak")
        .map(|(_, body)| body)
        .find(|trak| handler(trak) == Some(*b"vide"))?;
    let mdia = child(trak, b"mdia")?;
    let timescale = timescale(child(mdia, b"mdhd")?)?;
    let stbl = child(child(mdia, b"minf")?, b"stbl")?;
    let (kind, entry) = sample_entry(stbl)?;
    let codec = codec_of(&kind)?;
    let children = entry.get(78..)?;
    // ProRes has no configuration record; its sample entry is enough.
    let config = match codec {
        Codec::H264 => child(children, b"avcC")?,
        Codec::Hevc => child(children, b"hvcC")?,
        Codec::Vp9 => child(children, b"vpcC")?,
        Codec::Av1 => child(children, b"av1C")?,
        Codec::ProRes => &[],
    }
    .to_vec();

    let sizes = sample_sizes(stbl)?;
    let offsets = sample_offsets(stbl, &sizes)?;
    let decode_times = decode_times(stbl, sizes.len())?;
    let composition = composition_offsets(stbl, sizes.len())?;
    let sync = sync_samples(stbl);

    let samples = (0..sizes.len())
        .map(|index| Sample {
            offset: offsets[index],
            size: sizes[index],
            decode_time: decode_times[index],
            presentation_time: decode_times[index] + composition[index],
            sync: sync
                .as_ref()
                .is_none_or(|set| set.contains(&(index as u32 + 1))),
        })
        .collect();
    Some(SampleTrack {
        codec,
        width: be::<2>(entry, 24)? as u32,
        height: be::<2>(entry, 26)? as u32,
        config,
        timescale,
        samples,
    })
}

/// The size of every sample: a fixed size, or one entry per sample in `stsz`.
pub(crate) fn sample_sizes(stbl: &[u8]) -> Option<Vec<u32>> {
    let (_, rest) = full_box(child(stbl, b"stsz")?)?;
    let fixed = be::<4>(rest, 0)?;
    let count = be::<4>(rest, 4)? as usize;
    if fixed != 0 {
        return Some(vec![fixed as u32; count]);
    }
    (0..count)
        .map(|index| be::<4>(rest, 8 + 4 * index).map(|size| size as u32))
        .collect()
}

/// The file offset of every sample, from the chunk offsets (`stco` or `co64`) and the
/// sample-to-chunk runs (`stsc`).
pub(crate) fn sample_offsets(stbl: &[u8], sizes: &[u32]) -> Option<Vec<u64>> {
    let chunks = chunk_offsets(stbl)?;
    let (_, rest) = full_box(child(stbl, b"stsc")?)?;
    let runs = be::<4>(rest, 0)? as usize;
    let runs: Vec<(u32, u32)> = (0..runs)
        .map(|index| {
            let at = 4 + 12 * index;
            Some((be::<4>(rest, at)? as u32, be::<4>(rest, at + 4)? as u32))
        })
        .collect::<Option<_>>()?;

    let mut offsets = Vec::with_capacity(sizes.len());
    for (chunk_index, &chunk_offset) in chunks.iter().enumerate() {
        let chunk_number = chunk_index as u32 + 1;
        // The last run that starts at or before this chunk says how many samples it holds.
        let per_chunk = runs
            .iter()
            .rev()
            .find(|(first, _)| *first <= chunk_number)?
            .1 as usize;
        let mut at = chunk_offset;
        for _ in 0..per_chunk {
            if offsets.len() == sizes.len() {
                return Some(offsets);
            }
            offsets.push(at);
            at += u64::from(sizes[offsets.len() - 1]);
        }
    }
    (offsets.len() == sizes.len()).then_some(offsets)
}

fn chunk_offsets(stbl: &[u8]) -> Option<Vec<u64>> {
    if let Some(stco) = child(stbl, b"stco") {
        let (_, rest) = full_box(stco)?;
        let count = be::<4>(rest, 0)? as usize;
        (0..count)
            .map(|index| be::<4>(rest, 4 + 4 * index))
            .collect()
    } else {
        let (_, rest) = full_box(child(stbl, b"co64")?)?;
        let count = be::<4>(rest, 0)? as usize;
        (0..count)
            .map(|index| be::<8>(rest, 4 + 8 * index))
            .collect()
    }
}

/// The decode time of every sample, from `stts`.
pub(crate) fn decode_times(stbl: &[u8], count: usize) -> Option<Vec<i64>> {
    let (_, rest) = full_box(child(stbl, b"stts")?)?;
    let runs = be::<4>(rest, 0)? as usize;
    let mut times = Vec::with_capacity(count);
    let mut time = 0i64;
    for index in 0..runs {
        let samples = be::<4>(rest, 4 + 8 * index)?;
        let duration = be::<4>(rest, 8 + 8 * index)? as i64;
        for _ in 0..samples {
            if times.len() == count {
                return Some(times);
            }
            times.push(time);
            time += duration;
        }
    }
    (times.len() == count).then_some(times)
}

/// The offset from decode time to presentation time of every sample, from `ctts`. Zero when the
/// track has none.
fn composition_offsets(stbl: &[u8], count: usize) -> Option<Vec<i64>> {
    let Some(ctts) = child(stbl, b"ctts") else {
        return Some(vec![0; count]);
    };
    let (version, rest) = full_box(ctts)?;
    let runs = be::<4>(rest, 0)? as usize;
    let mut offsets = Vec::with_capacity(count);
    for index in 0..runs {
        let samples = be::<4>(rest, 4 + 8 * index)?;
        let raw = be::<4>(rest, 8 + 8 * index)? as u32;
        // Version 1 offsets are signed; version 0 offsets are not.
        let offset = if version == 1 {
            i64::from(raw as i32)
        } else {
            i64::from(raw)
        };
        for _ in 0..samples {
            if offsets.len() == count {
                return Some(offsets);
            }
            offsets.push(offset);
        }
    }
    (offsets.len() == count).then_some(offsets)
}

/// The 1-based numbers of the sync samples, from `stss`. `None` when every sample is a sync sample.
fn sync_samples(stbl: &[u8]) -> Option<Vec<u32>> {
    let (_, rest) = full_box(child(stbl, b"stss")?)?;
    let count = be::<4>(rest, 0)? as usize;
    (0..count)
        .map(|index| be::<4>(rest, 4 + 4 * index).map(|number| number as u32))
        .collect()
}

/// One audio track of a file, as a track list shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioTrackInfo {
    /// The track's position among the file's audio tracks, from 0.
    pub index: usize,
    /// The language code the file gives, such as `eng`.
    pub language: Option<String>,
    /// The channel count, when the file states it.
    pub channels: Option<u32>,
    /// Whether `read_audio_samples_track` can read this track's codec.
    pub supported: bool,
}

/// The first audio track of an MP4, Matroska or WebM file. `None` when there is none, or when its
/// codec is not one this reads.
pub fn read_audio_samples(path: &Path) -> Result<Option<AudioTrack>> {
    read_audio_samples_track(path, 0)
}

/// The audio track at `index` among the file's audio tracks, counted from 0. `None` when there is
/// no such track, or when its codec is not one this reads.
pub fn read_audio_samples_track(path: &Path, index: usize) -> Result<Option<AudioTrack>> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    let is_ebml = file.read(&mut magic)? == 4 && magic == super::EBML_MAGIC;
    file.seek(SeekFrom::Start(0))?;
    if is_ebml {
        return Ok(super::matroska::read_audio(&mut file, index)?);
    }
    let Some(moov) = find_top(&mut file, b"moov")? else {
        return Ok(None);
    };
    Ok(audio_traks(&moov).nth(index).and_then(audio_track))
}

/// Every audio track of an MP4, Matroska or WebM file, in file order. Empty when there is no audio.
pub fn audio_tracks(path: &Path) -> Result<Vec<AudioTrackInfo>> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    let is_ebml = file.read(&mut magic)? == 4 && magic == super::EBML_MAGIC;
    file.seek(SeekFrom::Start(0))?;
    if is_ebml {
        return Ok(matroska_audio_tracks(&mut file)?);
    }
    let Some(moov) = find_top(&mut file, b"moov")? else {
        return Ok(Vec::new());
    };
    Ok(audio_traks(&moov)
        .enumerate()
        .map(|(index, trak)| mp4_track_info(index, trak))
        .collect())
}

/// The audio tracks of a movie's `moov` box, in file order.
fn audio_traks(moov: &[u8]) -> impl Iterator<Item = &[u8]> {
    boxes(moov)
        .filter(|(kind, _)| kind == b"trak")
        .map(|(_, body)| body)
        .filter(|trak| handler(trak) == Some(*b"soun"))
}

/// What the track list shows of an MP4 audio track. Its channels and support come from its sample
/// entry, which is the part that decides whether the codec can be read.
fn mp4_track_info(index: usize, trak: &[u8]) -> AudioTrackInfo {
    let mdia = child(trak, b"mdia");
    let language = mdia
        .and_then(|mdia| child(mdia, b"mdhd"))
        .and_then(mdhd_language);
    let format = mdia
        .and_then(|mdia| child(mdia, b"minf"))
        .and_then(|minf| child(minf, b"stbl"))
        .and_then(sample_entry)
        .and_then(|(kind, entry)| audio_format(&kind, entry));
    AudioTrackInfo {
        index,
        language,
        channels: format.as_ref().map(|(_, _, _, channels)| *channels),
        supported: format.is_some(),
    }
}

/// The language of a media header, as its three-letter code such as `eng`.
fn mdhd_language(mdhd: &[u8]) -> Option<String> {
    let (version, rest) = full_box(mdhd)?;
    // The times come before the language: 16 bytes in version 0, 28 in version 1.
    let at = if version == 1 { 28 } else { 16 };
    let packed = be::<2>(rest, at)?;
    // Three letters, each five bits counted from 0x60, after one unused bit.
    let letters = [10u32, 5, 0].map(|shift| ((packed >> shift) & 0x1F) as u8);
    letters
        .into_iter()
        .map(|letter| {
            (1..=26)
                .contains(&letter)
                .then_some(char::from(letter + 0x60))
        })
        .collect()
}

/// The Matroska element ids the track listing reads.
const SEGMENT: u64 = 0x1853_8067;
const CLUSTER: u64 = 0x1F43_B675;
const TRACKS: u64 = 0x1654_AE6B;
const TRACK_ENTRY: u64 = 0xAE;
const TRACK_TYPE: u64 = 0x83;
const CODEC_ID: u64 = 0x86;
const LANGUAGE: u64 = 0x22_B59C;
const AUDIO: u64 = 0xE1;
const CHANNELS: u64 = 0x9F;
/// The TrackType of an audio track.
const AUDIO_TRACK: u64 = 2;
/// The most bytes read at once for a header or the `Tracks` element. Both are small in practice,
/// so a larger size is treated as damage rather than allocated.
const MAX_READ_BYTES: u64 = 1 << 20;

/// The audio tracks of a Matroska or WebM file, from its `Tracks` element. That element comes
/// before the first cluster, so the walk stops there and the clusters are not read.
fn matroska_audio_tracks(file: &mut File) -> io::Result<Vec<AudioTrackInfo>> {
    let end = file.seek(SeekFrom::End(0))?;
    let mut position = 0u64;
    while position < end {
        let Some((id, size, header)) = element_header(file, position, end)? else {
            break;
        };
        let body = position + header;
        let body_end = size.map_or(end, |size| body.saturating_add(size).min(end));
        match id {
            SEGMENT => position = body,
            TRACKS => {
                let tracks = read_range(file, body, body_end)?;
                return Ok(matroska_audio_info(&tracks));
            }
            CLUSTER => break,
            _ => position = body_end,
        }
    }
    Ok(Vec::new())
}

/// The audio tracks in a `Tracks` element body. A track's index counts only the audio tracks.
fn matroska_audio_info(tracks: &[u8]) -> Vec<AudioTrackInfo> {
    let mut infos = Vec::new();
    for (id, entry) in elements(tracks) {
        if id != TRACK_ENTRY {
            continue;
        }
        let mut kind = 0;
        let mut codec_id = String::new();
        let mut language = None;
        let mut channels = None;
        for (field, value) in elements(entry) {
            match field {
                TRACK_TYPE => kind = uint(value),
                CODEC_ID => codec_id = text(value),
                LANGUAGE => language = Some(text(value)),
                AUDIO => {
                    channels = elements(value)
                        .find(|(number, _)| *number == CHANNELS)
                        .and_then(|(_, count)| u32::try_from(uint(count)).ok());
                }
                _ => {}
            }
        }
        if kind != AUDIO_TRACK {
            continue;
        }
        let index = infos.len();
        infos.push(AudioTrackInfo {
            index,
            language,
            channels,
            // The Matroska reader decodes these codecs at any track index.
            supported: matches!(codec_id.as_str(), "A_AAC" | "A_AC3" | "A_EAC3"),
        });
    }
    infos
}

/// The id, the size (`None` when unknown) and the header length of the element at `at`.
fn element_header(
    file: &mut File,
    at: u64,
    end: u64,
) -> io::Result<Option<(u64, Option<u64>, u64)>> {
    let head = read_range(file, at, end.min(at.saturating_add(12)))?;
    let Some((id, id_len)) = vint(&head, true) else {
        return Ok(None);
    };
    let Some((size, size_len)) = head.get(id_len..).and_then(|rest| vint(rest, false)) else {
        return Ok(None);
    };
    // An all-ones size means the size is unknown.
    let unknown = size == (1u64 << (7 * size_len)) - 1;
    Ok(Some((
        id,
        (!unknown).then_some(size),
        (id_len + size_len) as u64,
    )))
}

/// The bytes from `start` to `end` of the file, or the first `MAX_READ_BYTES` of them.
fn read_range(file: &mut File, start: u64, end: u64) -> io::Result<Vec<u8>> {
    let len = end.saturating_sub(start).min(MAX_READ_BYTES);
    file.seek(SeekFrom::Start(start))?;
    let mut data = vec![0u8; len as usize];
    file.read_exact(&mut data)?;
    Ok(data)
}

/// The codec, configuration record, sample rate and channel count of an audio sample entry.
fn audio_format(kind: &[u8; 4], entry: &[u8]) -> Option<(AudioCodec, Vec<u8>, u32, u32)> {
    // An audio sample entry: 28 bytes of fixed fields, then the codec's own boxes.
    let children = entry.get(28..)?;
    match kind {
        b"mp4a" => {
            let (_, esds) = full_box(child(children, b"esds")?)?;
            let config = descriptor(esds, 0x05)?.to_vec();
            let (rate, channels) = aac_format(&config)?;
            Some((AudioCodec::Aac, config, rate, channels))
        }
        b"ac-3" => {
            let record = child(children, b"dac3")?;
            let (rate, channels) = ac3_from_dac3(record)?;
            Some((AudioCodec::Ac3, record.to_vec(), rate, channels))
        }
        b"ec-3" => {
            let record = child(children, b"dec3")?;
            let (rate, channels) = eac3_from_dec3(record)?;
            Some((AudioCodec::Eac3, record.to_vec(), rate, channels))
        }
        _ => None,
    }
}

fn audio_track(trak: &[u8]) -> Option<AudioTrack> {
    let mdia = child(trak, b"mdia")?;
    let timescale = timescale(child(mdia, b"mdhd")?)?;
    let stbl = child(child(mdia, b"minf")?, b"stbl")?;
    let (kind, entry) = sample_entry(stbl)?;
    let (codec, config, sample_rate, channels) = audio_format(&kind, entry)?;

    let sizes = sample_sizes(stbl)?;
    let offsets = sample_offsets(stbl, &sizes)?;
    let times = decode_times(stbl, sizes.len())?;
    let samples = (0..sizes.len())
        .map(|index| Sample {
            offset: offsets[index],
            size: sizes[index],
            decode_time: times[index],
            presentation_time: times[index],
            sync: true,
        })
        .collect();
    Some(AudioTrack {
        codec,
        sample_rate,
        channels,
        config,
        timescale,
        priming_frames: priming_frames(trak, sample_rate, timescale),
        samples,
    })
}

/// The frames the edit list drops from the start: the media time of the first entry that plays
/// media, in frames. Encoders write this for the delay the decoder adds.
fn priming_frames(trak: &[u8], sample_rate: u32, timescale: u64) -> u64 {
    let Some(elst) = child(trak, b"edts").and_then(|edts| child(edts, b"elst")) else {
        return 0;
    };
    let Some((version, rest)) = full_box(elst) else {
        return 0;
    };
    let entries = be::<4>(rest, 0).unwrap_or(0) as usize;
    // Each entry: a segment duration, then a media time, then the rate. Version 1 uses 64 bits.
    let (size, media_at) = if version == 1 { (20, 8) } else { (12, 4) };
    for index in 0..entries {
        let at = 4 + index * size;
        let media = if version == 1 {
            be::<8>(rest, at + media_at).map(|value| value as i64)
        } else {
            be::<4>(rest, at + media_at).map(|value| i64::from(value as u32 as i32))
        };
        if let Some(media) = media.filter(|media| *media >= 0) {
            return (media as u64).saturating_mul(u64::from(sample_rate)) / timescale.max(1);
        }
    }
    0
}

/// The frames an MP4 file's first audio track asks the decoder to drop at its start, from its edit
/// list. `None` when the file is not an MP4 with an audio track.
pub fn mp4_priming_frames(path: &Path) -> Option<u64> {
    let mut file = File::open(path).ok()?;
    let moov = find_top(&mut file, b"moov").ok()??;
    let trak = boxes(&moov)
        .filter(|(kind, _)| kind == b"trak")
        .map(|(_, body)| body)
        .find(|trak| handler(trak) == Some(*b"soun"))?;
    let mdia = child(trak, b"mdia")?;
    let timescale = timescale(child(mdia, b"mdhd")?)?;
    let stbl = child(child(mdia, b"minf")?, b"stbl")?;
    let (_, entry) = sample_entry(stbl)?;
    // The sample rate is a 16.16 fixed point number in the audio sample entry.
    let rate = (be::<4>(entry, 24)? >> 16) as u32;
    Some(priming_frames(trak, rate, timescale))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An EBML element with a one-byte size, which is enough for these small bodies.
    fn element(id: &[u8], body: &[u8]) -> Vec<u8> {
        let mut out = id.to_vec();
        out.push(0x80 | body.len() as u8);
        out.extend_from_slice(body);
        out
    }

    fn audio_entry(codec: &[u8], language: &[u8], channels: u8) -> Vec<u8> {
        let audio = element(&[0xE1], &element(&[0x9F], &[channels]));
        let body = [
            element(&[0x83], &[2]),
            element(&[0x86], codec),
            element(&[0x22, 0xB5, 0x9C], language),
            audio,
        ]
        .concat();
        element(&[0xAE], &body)
    }

    #[test]
    fn mdhd_language_reads_the_packed_code() {
        // A version 0 media header: version and flags, four 4-byte times, then the language.
        let mut mdhd = vec![0u8; 4 + 16 + 2];
        let eng: u16 = (5 << 10) | (14 << 5) | 7;
        mdhd[20..22].copy_from_slice(&eng.to_be_bytes());
        assert_eq!(mdhd_language(&mdhd).as_deref(), Some("eng"));
        mdhd[20..22].fill(0);
        assert_eq!(mdhd_language(&mdhd), None);
    }

    #[test]
    fn matroska_tracks_count_only_audio_tracks() {
        let video = element(&[0xAE], &element(&[0x83], &[1]));
        let tracks = [
            video,
            audio_entry(b"A_AAC", b"eng", 2),
            audio_entry(b"A_OPUS", b"jpn", 6),
        ]
        .concat();
        let infos = matroska_audio_info(&tracks);
        assert_eq!(
            infos,
            vec![
                AudioTrackInfo {
                    index: 0,
                    language: Some("eng".into()),
                    channels: Some(2),
                    supported: true,
                },
                AudioTrackInfo {
                    index: 1,
                    language: Some("jpn".into()),
                    channels: Some(6),
                    supported: false,
                },
            ]
        );
    }

    #[test]
    fn matroska_audio_tracks_are_supported_at_any_index() {
        let tracks = [
            audio_entry(b"A_AAC", b"eng", 2),
            audio_entry(b"A_AC3", b"jpn", 6),
        ]
        .concat();
        let infos = matroska_audio_info(&tracks);
        assert!(infos.iter().all(|info| info.supported));
    }

    /// An EBML element with an eight-byte size, so that any body length fits.
    fn sized_element(id: &[u8], body: &[u8]) -> Vec<u8> {
        let mut out = id.to_vec();
        out.push(0x01);
        out.extend_from_slice(&(body.len() as u64).to_be_bytes()[1..]);
        out.extend_from_slice(body);
        out
    }

    /// A Matroska file with a video track (number 1) and two AAC audio tracks (numbers 2 and 3),
    /// whose blocks are interleaved in one cluster.
    fn two_audio_file() -> Vec<u8> {
        let audio = |number: u8, rate: f32| {
            let audio_fields = [
                sized_element(&[0xB5], &rate.to_be_bytes()),
                sized_element(&[0x9F], &[2]),
            ]
            .concat();
            sized_element(
                &[0xAE],
                &[
                    sized_element(&[0xD7], &[number]),
                    sized_element(&[0x83], &[2]),
                    sized_element(&[0x86], b"A_AAC"),
                    sized_element(&[0xE1], &audio_fields),
                ]
                .concat(),
            )
        };
        // A SimpleBlock: track number, 16-bit timecode, flags (a keyframe), then the frame.
        let block = |track: u8, payload: &[u8]| {
            let mut body = vec![0x80 | track, 0, 0, 0x80];
            body.extend_from_slice(payload);
            sized_element(&[0xA3], &body)
        };
        let video = sized_element(
            &[0xAE],
            &[sized_element(&[0xD7], &[1]), sized_element(&[0x83], &[1])].concat(),
        );
        let tracks = sized_element(
            &[0x16, 0x54, 0xAE, 0x6B],
            &[video, audio(2, 48_000.0), audio(3, 44_100.0)].concat(),
        );
        let cluster = sized_element(
            &[0x1F, 0x43, 0xB6, 0x75],
            &[
                sized_element(&[0xE7], &[0]),
                block(2, &[0xAA; 5]),
                block(3, &[0xBB; 7]),
                block(2, &[0xCC; 3]),
            ]
            .concat(),
        );
        let segment = sized_element(&[0x18, 0x53, 0x80, 0x67], &[tracks, cluster].concat());
        [sized_element(&[0x1A, 0x45, 0xDF, 0xA3], &[]), segment].concat()
    }

    #[test]
    fn matroska_reads_each_audio_track_by_index() {
        use crate::video::matroska::read_audio;
        let file = two_audio_file();
        let payloads = |track: &AudioTrack| -> Vec<Vec<u8>> {
            track
                .samples
                .iter()
                .map(|sample| {
                    let start = sample.offset as usize;
                    file[start..start + sample.size as usize].to_vec()
                })
                .collect()
        };

        let mut cursor = std::io::Cursor::new(file.clone());
        let first = read_audio(&mut cursor, 0)
            .expect("the file parses")
            .expect("track 0 is read");
        assert_eq!((first.sample_rate, first.channels), (48_000, 2));
        assert_eq!(payloads(&first), vec![vec![0xAA; 5], vec![0xCC; 3]]);

        let mut cursor = std::io::Cursor::new(file.clone());
        let second = read_audio(&mut cursor, 1)
            .expect("the file parses")
            .expect("track 1 is read");
        assert_eq!(second.sample_rate, 44_100);
        assert_eq!(payloads(&second), vec![vec![0xBB; 7]]);

        let mut cursor = std::io::Cursor::new(file);
        assert!(
            read_audio(&mut cursor, 2)
                .expect("the file parses")
                .is_none()
        );
    }
}
