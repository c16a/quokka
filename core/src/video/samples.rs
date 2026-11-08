//! The compressed frames of an MP4 file's first video track, for decoders. Each sample is one
//! frame in the codec's own packet format: length-prefixed NAL units for H.264 and HEVC.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use super::Codec;
use super::audio::{AudioCodec, AudioTrack, aac_format, ac3_from_dac3, descriptor, eac3_from_dec3};
use super::bmff::{be, boxes, child, find_top, full_box};
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

/// The first audio track of an MP4, Matroska or WebM file. `None` when there is none, or when its
/// codec is not one this reads.
pub fn read_audio_samples(path: &Path) -> Result<Option<AudioTrack>> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    let is_ebml = file.read(&mut magic)? == 4 && magic == super::EBML_MAGIC;
    file.seek(SeekFrom::Start(0))?;
    if is_ebml {
        return Ok(super::matroska::read_audio(&mut file)?);
    }
    let Some(moov) = find_top(&mut file, b"moov")? else {
        return Ok(None);
    };
    Ok(audio_track(&moov))
}

fn audio_track(moov: &[u8]) -> Option<AudioTrack> {
    let trak = boxes(moov)
        .filter(|(kind, _)| kind == b"trak")
        .map(|(_, body)| body)
        .find(|trak| handler(trak) == Some(*b"soun"))?;
    let mdia = child(trak, b"mdia")?;
    let timescale = timescale(child(mdia, b"mdhd")?)?;
    let stbl = child(child(mdia, b"minf")?, b"stbl")?;
    let (kind, entry) = sample_entry(stbl)?;
    // An audio sample entry: 28 bytes of fixed fields, then the codec's own boxes.
    let children = entry.get(28..)?;
    let (codec, config, sample_rate, channels) = match &kind {
        b"mp4a" => {
            let (_, esds) = full_box(child(children, b"esds")?)?;
            let config = descriptor(esds, 0x05)?.to_vec();
            let (rate, channels) = aac_format(&config)?;
            (AudioCodec::Aac, config, rate, channels)
        }
        b"ac-3" => {
            let record = child(children, b"dac3")?;
            let (rate, channels) = ac3_from_dac3(record)?;
            (AudioCodec::Ac3, record.to_vec(), rate, channels)
        }
        b"ec-3" => {
            let record = child(children, b"dec3")?;
            let (rate, channels) = eac3_from_dec3(record)?;
            (AudioCodec::Eac3, record.to_vec(), rate, channels)
        }
        _ => return None,
    };

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
