//! A rodio `Source` that decodes with symphonia. Decoding is ours, and output is
//! handled by rodio. Seeking goes to symphonia's demuxer.

use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};
use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::codecs::registry::CodecRegistry;
use symphonia::core::errors::Error as DecodeError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::Time;

/// Shared slot holding the error that stopped a track early, if there was one.
pub type ErrorSlot = Arc<Mutex<Option<String>>>;

/// Symphonia's default codecs, plus Opus (which symphonia itself does not decode).
fn codecs() -> &'static CodecRegistry {
    static CODECS: OnceLock<CodecRegistry> = OnceLock::new();
    CODECS.get_or_init(|| {
        let mut registry = CodecRegistry::new();
        symphonia::default::register_enabled_codecs(&mut registry);
        registry.register_audio_decoder::<symphonia_adapter_opus_rs::OpusAdapter>();
        registry
    })
}

/// Streams interleaved `f32` samples decoded from one audio file.
pub struct SymphoniaSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    sample_rate: SampleRate,
    channels: ChannelCount,
    duration: Option<Duration>,
    /// Samples of the packet currently being played.
    samples: Vec<f32>,
    cursor: usize,
    finished: bool,
    /// Samples to drop after a seek, so playback starts exactly at the target.
    skip_samples: usize,
    /// Set when decoding fails for a reason other than the end of the file.
    error: ErrorSlot,
}

impl SymphoniaSource {
    /// Open and probe `path`, and prepare the first audio track for decoding.
    pub fn open(path: &Path, error: ErrorSlot) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let stream = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let options = FormatOptions::default();
        let format = symphonia::default::get_probe()
            .probe(&hint, stream, options, MetadataOptions::default())
            .map_err(|e| e.to_string())?;

        let (track_id, params, num_frames) = format
            .tracks()
            .iter()
            .find_map(|track| match &track.codec_params {
                Some(CodecParameters::Audio(params)) => {
                    Some((track.id, params.clone(), track.num_frames))
                }
                _ => None,
            })
            .ok_or("no audio track")?;

        let sample_rate = params
            .sample_rate
            .and_then(SampleRate::new)
            .ok_or("unknown sample rate")?;
        let duration = num_frames
            .map(|frames| Duration::from_secs_f64(frames as f64 / f64::from(sample_rate.get())));
        let channels = params
            .channels
            .as_ref()
            .and_then(|c| ChannelCount::new(c.count() as u16))
            .ok_or("unknown channel layout")?;

        let decoder = codecs()
            .make_audio_decoder(&params, &AudioDecoderOptions::default())
            .map_err(|e| e.to_string())?;

        Ok(Self {
            format,
            decoder,
            track_id,
            sample_rate,
            channels,
            duration,
            samples: Vec::new(),
            cursor: 0,
            finished: false,
            skip_samples: 0,
            error,
        })
    }

    /// Decode packets until one yields samples, or the stream ends.
    fn decode_next_packet(&mut self) {
        loop {
            let packet = match self.format.next_packet() {
                Ok(Some(packet)) => packet,
                // The end of the stream is normal, not an error.
                Ok(None) => {
                    self.finished = true;
                    return;
                }
                Err(e) => {
                    self.fail(e.to_string());
                    return;
                }
            };
            if packet.track_id != self.track_id {
                continue;
            }
            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    self.samples.clear();
                    decoded.copy_to_vec_interleaved(&mut self.samples);
                    // After an accurate seek the packet starts before the target.
                    let drop = self.skip_samples.min(self.samples.len());
                    self.samples.drain(..drop);
                    self.skip_samples -= drop;
                    self.cursor = 0;
                    return;
                }
                // A damaged packet is skipped; the rest of the file still plays.
                Err(DecodeError::DecodeError(_)) => continue,
                // The decoder asks to be reset; carry on with the next packet.
                Err(DecodeError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(e) => {
                    self.fail(e.to_string());
                    return;
                }
            }
        }
    }

    /// Stop the track and record why.
    fn fail(&mut self, message: String) {
        self.finished = true;
        if let Ok(mut slot) = self.error.lock() {
            *slot = Some(message);
        }
    }
}

impl Iterator for SymphoniaSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        loop {
            if let Some(&sample) = self.samples.get(self.cursor) {
                self.cursor += 1;
                return Some(sample);
            }
            if self.finished {
                return None;
            }
            self.decode_next_packet();
        }
    }
}

impl Source for SymphoniaSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.duration
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let not_supported = || SeekError::NotSupported {
            underlying_source: "symphonia",
        };
        let time = Time::try_from_secs_f64(pos.as_secs_f64()).ok_or_else(not_supported)?;
        let seeked = self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|_| not_supported())?;
        // Timestamps count frames; convert the gap to interleaved samples.
        let frames_to_skip = (seeked.required_ts.get() - seeked.actual_ts.get()).max(0) as usize;
        self.skip_samples = frames_to_skip * self.channels.get() as usize;
        self.decoder.reset();
        self.samples.clear();
        self.cursor = 0;
        self.finished = false;
        Ok(())
    }
}
