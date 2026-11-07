//! Subtitles: SRT files beside a movie, and the text tracks inside MP4 and Matroska files. Each
//! cue is the text shown between two times, in seconds from the start.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use super::bmff::{boxes, child, find_top};
use super::matroska::{SUBTITLE_TRACK, read_track};
use super::mp4::{handler, sample_entry, timescale};
use super::samples::{decode_times, sample_offsets, sample_sizes};

/// The EBML magic that starts Matroska and WebM files.
const EBML_MAGIC: [u8; 4] = [0x1A, 0x45, 0xDF, 0xA3];

/// How long a cue lasts when the next one does not say where it ends.
const DEFAULT_CUE_SECONDS: f64 = 2.0;

/// One piece of subtitle text.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// The subtitles for a movie: the SRT file beside it, when there is one, else the first text track
/// inside it. Empty when there are none.
pub fn read_subtitles(path: &Path) -> std::io::Result<Vec<Cue>> {
    let sidecar = path.with_extension("srt");
    if sidecar.is_file() {
        let text = String::from_utf8_lossy(&std::fs::read(&sidecar)?).into_owned();
        return Ok(parse_srt(&text));
    }
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    let is_ebml = file.read(&mut magic)? == 4 && magic == EBML_MAGIC;
    file.seek(SeekFrom::Start(0))?;
    let cues = if is_ebml {
        matroska_text(&mut file)?
    } else {
        mp4_text(&mut file)?
    };
    Ok(cues.unwrap_or_default())
}

/// The cues of an SRT file. Times are `hh:mm:ss,mmm` (or with a full stop), cues are separated by
/// blank lines, and simple HTML tags are dropped from the text.
pub fn parse_srt(text: &str) -> Vec<Cue> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut cues = Vec::new();
    for block in text.split("\n\n") {
        let lines: Vec<&str> = block
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        let Some(timing) = lines.iter().position(|line| line.contains("-->")) else {
            continue;
        };
        let Some((start, end)) = lines[timing].split_once("-->") else {
            continue;
        };
        let (Some(start), Some(end)) = (srt_time(start), srt_time(end)) else {
            continue;
        };
        let body: Vec<String> = lines[timing + 1..]
            .iter()
            .map(|line| strip_tags(line))
            .collect();
        let text = body.join("\n");
        if !text.is_empty() {
            cues.push(Cue { start, end, text });
        }
    }
    cues
}

/// A time in an SRT file, in seconds.
fn srt_time(text: &str) -> Option<f64> {
    let text = text.trim();
    let (clock, millis) = text.split_once([',', '.'])?;
    let mut parts = clock.split(':').map(|part| part.trim().parse::<f64>().ok());
    let (hours, minutes, seconds) = (parts.next()??, parts.next()??, parts.next()??);
    let millis: f64 = millis.trim().parse().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds + millis / 1000.0)
}

/// The text without `<...>` markup.
fn strip_tags(line: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for ch in line.chars() {
        match ch {
            '<' => inside = true,
            '>' if inside => inside = false,
            _ if !inside => out.push(ch),
            _ => {}
        }
    }
    out
}

/// Cues from a text track in an MP4 file (`tx3g`, the `mov_text` format). Each sample is a length
/// and then the text.
fn mp4_text(file: &mut File) -> std::io::Result<Option<Vec<Cue>>> {
    let Some(moov) = find_top(file, b"moov")? else {
        return Ok(None);
    };
    let Some(trak) = boxes(&moov)
        .filter(|(kind, _)| kind == b"trak")
        .map(|(_, body)| body)
        .find(|trak| matches!(handler(trak), Some(kind) if kind == *b"sbtl" || kind == *b"text" || kind == *b"subt"))
    else {
        return Ok(None);
    };
    let Some(mdia) = child(trak, b"mdia") else {
        return Ok(None);
    };
    let (Some(scale), Some(stbl)) = (
        child(mdia, b"mdhd").and_then(timescale),
        child(mdia, b"minf").and_then(|minf| child(minf, b"stbl")),
    ) else {
        return Ok(None);
    };
    let Some((kind, _)) = sample_entry(stbl) else {
        return Ok(None);
    };
    if &kind != b"tx3g" {
        return Ok(None);
    }
    let Some(sizes) = sample_sizes(stbl) else {
        return Ok(None);
    };
    let Some(times) = decode_times(stbl, sizes.len()) else {
        return Ok(None);
    };
    let Some(offsets) = sample_offsets(stbl, &sizes) else {
        return Ok(None);
    };
    let scale = scale as f64;
    let mut cues = Vec::new();
    for index in 0..sizes.len() {
        let start = times[index] as f64 / scale;
        let end = times
            .get(index + 1)
            .map_or(start + DEFAULT_CUE_SECONDS, |&next| next as f64 / scale);
        let mut data = vec![0u8; sizes[index] as usize];
        file.seek(SeekFrom::Start(offsets[index]))?;
        file.read_exact(&mut data)?;
        let Some(length) = data
            .get(..2)
            .map(|bytes| usize::from(u16::from_be_bytes([bytes[0], bytes[1]])))
        else {
            continue;
        };
        let text = String::from_utf8_lossy(data.get(2..2 + length).unwrap_or(&[])).into_owned();
        if !text.is_empty() {
            cues.push(Cue { start, end, text });
        }
    }
    Ok(Some(cues))
}

/// Cues from the subtitle track of a Matroska or WebM file. Plain text and ASS both work; ASS keeps
/// its text after the eight fields that come before it.
fn matroska_text(file: &mut File) -> std::io::Result<Option<Vec<Cue>>> {
    let Some((track, scale, samples)) = read_track(file, SUBTITLE_TRACK)? else {
        return Ok(None);
    };
    let ass = matches!(track.codec_id.as_str(), "S_TEXT/ASS" | "S_TEXT/SSA");
    if !ass && !matches!(track.codec_id.as_str(), "S_TEXT/UTF8" | "S_TEXT/ASCII") {
        return Ok(None);
    }
    let scale = scale as f64;
    let mut cues = Vec::new();
    for (index, sample) in samples.iter().enumerate() {
        let start = sample.presentation_time as f64 / scale;
        let end = samples
            .get(index + 1)
            .map_or(start + DEFAULT_CUE_SECONDS, |next| {
                next.presentation_time as f64 / scale
            });
        let mut data = vec![0u8; sample.size as usize];
        file.seek(SeekFrom::Start(sample.offset))?;
        file.read_exact(&mut data)?;
        let raw = String::from_utf8_lossy(&data).into_owned();
        let text = if ass {
            // Dialogue fields: ReadOrder, Layer, Style, Name, MarginL, MarginR, MarginV, Effect, Text.
            let after_fields = raw.splitn(9, ',').nth(8).unwrap_or(&raw);
            after_fields.replace("\\N", "\n").replace("\\n", "\n")
        } else {
            raw
        };
        let text = strip_ass_overrides(&text);
        if !text.is_empty() {
            cues.push(Cue { start, end, text });
        }
    }
    Ok(Some(cues))
}

/// The text without ASS override blocks such as `{\an8}`.
fn strip_ass_overrides(text: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for ch in text.chars() {
        match ch {
            '{' => inside = true,
            '}' if inside => inside = false,
            _ if !inside => out.push(ch),
            _ => {}
        }
    }
    out.trim().to_owned()
}
