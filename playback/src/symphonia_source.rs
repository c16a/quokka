//! A rodio `Source` whose samples come from a decoder thread. The thread decodes ahead
//! of playback into a bounded buffer, so the audio thread only copies samples out.
//! Seeks are sent to the thread, and samples from before a seek are dropped by generation.

use std::fs::File;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};
use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::codecs::registry::CodecRegistry;
use symphonia::core::errors::Error as DecodeError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::Time;

/// Shared slot holding the error that stopped a track early, if there was one.
pub type ErrorSlot = Arc<Mutex<Option<String>>>;

/// Decoded chunks the thread may run ahead of playback (about 1.5 s of 96 kHz stereo).
const BUFFER_CHUNKS: usize = 32;
/// How long the thread waits before retrying when the buffer is full.
const RETRY: Duration = Duration::from_millis(2);
/// How often an idle thread (end of file reached) checks for a seek or for shutdown.
const IDLE: Duration = Duration::from_millis(20);

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

/// Requests from the source to the decoder thread.
enum Command {
    Seek { time: Time, generation: u64 },
}

/// Output from the decoder thread. Each message is tagged with the generation it was
/// decoded for, so the source can drop anything from before a seek.
enum Message {
    Samples { generation: u64, samples: Vec<f32> },
    End { generation: u64 },
}

/// Streams interleaved `f32` samples decoded on a background thread.
pub struct SymphoniaSource {
    commands: Sender<Command>,
    messages: Receiver<Message>,
    sample_rate: SampleRate,
    channels: ChannelCount,
    duration: Option<Duration>,
    /// Incremented by every seek. Messages with another generation are stale.
    generation: u64,
    /// Samples of the chunk currently being played.
    samples: Vec<f32>,
    cursor: usize,
    finished: bool,
}

/// Everything the decoder thread needs. It is moved into the thread when the source opens.
struct Worker {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    channels: usize,
    /// Samples to drop after a seek, so playback starts exactly at the target.
    skip_samples: usize,
    generation: u64,
    error: ErrorSlot,
}

/// One step of decoding: a chunk of samples, or the end of the stream.
enum Chunk {
    Samples(Vec<f32>),
    Finished,
}

impl SymphoniaSource {
    /// Open and probe `path`, and start the decoder thread for its first audio track.
    pub fn open(path: &Path, error: ErrorSlot) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        Self::from_source(Box::new(file), &hint, error)
    }

    /// Decode from any source. `hint` names the format when the caller knows it. A source that
    /// can't seek (a live stream) has no length, and can't be sought.
    pub fn from_source(
        source: Box<dyn MediaSource>,
        hint: &Hint,
        error: ErrorSlot,
    ) -> Result<Self, String> {
        let stream = MediaSourceStream::new(source, Default::default());
        let format = symphonia::default::get_probe()
            .probe(
                hint,
                stream,
                FormatOptions::default(),
                MetadataOptions::default(),
            )
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
        let channels = params
            .channels
            .as_ref()
            .and_then(|c| ChannelCount::new(c.count() as u16))
            .ok_or("unknown channel layout")?;
        let duration = num_frames
            .map(|frames| Duration::from_secs_f64(frames as f64 / f64::from(sample_rate.get())));

        let decoder = codecs()
            .make_audio_decoder(&params, &AudioDecoderOptions::default())
            .map_err(|e| e.to_string())?;

        let (command_tx, command_rx) = mpsc::channel();
        let (message_tx, message_rx) = mpsc::sync_channel(BUFFER_CHUNKS);
        let worker = Worker {
            format,
            decoder,
            track_id,
            channels: channels.get() as usize,
            skip_samples: 0,
            generation: 0,
            error,
        };
        thread::Builder::new()
            .name("decoder".into())
            .spawn(move || worker.run(command_rx, message_tx))
            .map_err(|e| e.to_string())?;

        Ok(Self {
            commands: command_tx,
            messages: message_rx,
            sample_rate,
            channels,
            duration,
            generation: 0,
            samples: Vec::new(),
            cursor: 0,
            finished: false,
        })
    }
}

impl Worker {
    /// Decode ahead into the buffer, and act on seeks as they arrive.
    fn run(mut self, commands: Receiver<Command>, messages: SyncSender<Message>) {
        let mut pending: Option<Message> = None;
        let mut finished = false;
        loop {
            // Only the most recent seek matters.
            let mut seek = None;
            loop {
                match commands.try_recv() {
                    Ok(command) => seek = Some(command),
                    Err(TryRecvError::Empty) => break,
                    // The source was dropped, so there is nothing left to decode for.
                    Err(TryRecvError::Disconnected) => return,
                }
            }
            if let Some(Command::Seek { time, generation }) = seek {
                // Anything decoded before the seek is now stale.
                pending = None;
                finished = false;
                if let Err(message) = self.seek(time, generation) {
                    self.fail(message);
                    pending = Some(Message::End { generation });
                    finished = true;
                }
                continue;
            }

            if let Some(message) = pending.take() {
                match messages.try_send(message) {
                    Ok(()) => {}
                    Err(TrySendError::Full(message)) => {
                        pending = Some(message);
                        thread::sleep(RETRY);
                    }
                    Err(TrySendError::Disconnected(_)) => return,
                }
                continue;
            }

            if finished {
                thread::sleep(IDLE);
                continue;
            }

            match self.next_chunk() {
                Chunk::Samples(samples) => {
                    pending = Some(Message::Samples {
                        generation: self.generation,
                        samples,
                    });
                }
                Chunk::Finished => {
                    finished = true;
                    pending = Some(Message::End {
                        generation: self.generation,
                    });
                }
            }
        }
    }

    /// Decode packets until one yields samples, or the stream ends.
    fn next_chunk(&mut self) -> Chunk {
        loop {
            let packet = match self.format.next_packet() {
                Ok(Some(packet)) => packet,
                // The end of the stream is normal, not an error.
                Ok(None) => return Chunk::Finished,
                Err(e) => {
                    self.fail(e.to_string());
                    return Chunk::Finished;
                }
            };
            if packet.track_id != self.track_id {
                continue;
            }
            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let mut samples = Vec::new();
                    decoded.copy_to_vec_interleaved(&mut samples);
                    // After an accurate seek the packet starts before the target.
                    let drop = self.skip_samples.min(samples.len());
                    samples.drain(..drop);
                    self.skip_samples -= drop;
                    if !samples.is_empty() {
                        return Chunk::Samples(samples);
                    }
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
                    return Chunk::Finished;
                }
            }
        }
    }

    /// Move the demuxer to `time` and start a new generation of output.
    fn seek(&mut self, time: Time, generation: u64) -> Result<(), String> {
        self.generation = generation;
        let seeked = self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| e.to_string())?;
        // Timestamps count frames; convert the gap to interleaved samples.
        let frames = (seeked.required_ts.get() - seeked.actual_ts.get()).max(0) as usize;
        self.skip_samples = frames * self.channels;
        self.decoder.reset();
        Ok(())
    }

    /// Stop the track and record why.
    fn fail(&self, message: String) {
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
            match self.messages.recv() {
                Ok(Message::Samples {
                    generation,
                    samples,
                }) if generation == self.generation => {
                    self.samples = samples;
                    self.cursor = 0;
                }
                Ok(Message::End { generation }) if generation == self.generation => {
                    self.finished = true;
                }
                // Left over from before a seek.
                Ok(_) => {}
                // The decoder thread has exited.
                Err(_) => self.finished = true,
            }
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
        self.generation += 1;
        self.commands
            .send(Command::Seek {
                time,
                generation: self.generation,
            })
            .map_err(|_| not_supported())?;
        // The decoder thread applies the seek; anything already buffered is now stale.
        self.samples.clear();
        self.cursor = 0;
        self.finished = false;
        Ok(())
    }
}
