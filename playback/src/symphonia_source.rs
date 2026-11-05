//! A rodio `Source` that decodes with symphonia. Decoding is ours, and output is
//! handled by rodio. Seeking goes to symphonia's demuxer.

use std::fs::File;
use std::path::Path;
use std::time::Duration;

use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, Decoder, DecoderOptions};
use symphonia::core::errors::Error as DecodeError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

/// Streams interleaved `f32` samples decoded from one audio file.
pub struct SymphoniaSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
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
}

impl SymphoniaSource {
    /// Open and probe `path`, and prepare the first audio track for decoding.
    pub fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let stream = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let options = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };
        let probed = symphonia::default::get_probe()
            .format(&hint, stream, &options, &MetadataOptions::default())
            .map_err(|e| e.to_string())?;
        let format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("no audio track")?;
        let params = track.codec_params.clone();
        let track_id = track.id;

        let sample_rate = params
            .sample_rate
            .and_then(SampleRate::new)
            .ok_or("unknown sample rate")?;
        let channels = params
            .channels
            .and_then(|c| ChannelCount::new(c.count() as u16))
            .ok_or("unknown channel layout")?;
        let duration = match (params.n_frames, params.sample_rate) {
            (Some(frames), Some(rate)) if rate > 0 => {
                Some(Duration::from_secs_f64(frames as f64 / rate as f64))
            }
            _ => None,
        };

        let decoder = symphonia::default::get_codecs()
            .make(&params, &DecoderOptions::default())
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
        })
    }

    /// Decode packets until one yields samples, or the stream ends.
    fn decode_next_packet(&mut self) {
        loop {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(_) => {
                    self.finished = true;
                    return;
                }
            };
            if packet.track_id() != self.track_id {
                continue;
            }
            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let capacity = decoded.capacity() as u64;
                    let spec = *decoded.spec();
                    let mut buffer = SampleBuffer::<f32>::new(capacity, spec);
                    buffer.copy_interleaved_ref(decoded);
                    self.samples.clear();
                    self.samples.extend_from_slice(buffer.samples());
                    // After an accurate seek the packet starts before the target.
                    let drop = self.skip_samples.min(self.samples.len());
                    self.samples.drain(..drop);
                    self.skip_samples -= drop;
                    self.cursor = 0;
                    return;
                }
                // A corrupt packet is skipped; the rest of the file still plays.
                Err(DecodeError::DecodeError(_)) => continue,
                Err(_) => {
                    self.finished = true;
                    return;
                }
            }
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
        let seeked = self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time: Time::from(pos),
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|_| SeekError::NotSupported {
                underlying_source: "symphonia",
            })?;
        // Timestamps count frames; convert the gap to interleaved samples.
        let frames_to_skip = seeked.required_ts.saturating_sub(seeked.actual_ts);
        self.skip_samples = frames_to_skip as usize * self.channels.get() as usize;
        self.decoder.reset();
        self.samples.clear();
        self.cursor = 0;
        self.finished = false;
        Ok(())
    }
}
