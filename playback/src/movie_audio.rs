//! Audio for movies. The soundtrack is decoded by the audio decoder chain on a worker thread, and
//! played through the output device. The samples the device has pulled give the playback clock, so
//! the video can follow the sound.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::thread;
use std::time::Duration;

use quokka_core::video::{AudioTrack, read_audio_samples_track};
use rodio::{ChannelCount, MixerDeviceSink, Player, SampleRate, Source};

use crate::PlaybackError;
use crate::decode::audio::{self, AudioStream};
use crate::rodio_backend::open_output;

/// Decoded chunks waiting for the device: about a second of audio at most.
const CHUNKS_IN_FLIGHT: usize = 32;

/// How long the device waits for a chunk before it plays silence instead.
const WAIT_FOR_CHUNK: Duration = Duration::from_millis(5);

/// Messages from the decoding thread.
enum Message {
    Ready,
    Chunk(Vec<f32>),
    Failed(String),
}

/// The soundtrack of a movie, playing through the output device.
pub struct MovieAudio {
    /// Keeps the device open for as long as the audio plays.
    _sink: MixerDeviceSink,
    player: Player,
    /// Samples the device has pulled, across all channels.
    pulled: Arc<AtomicU64>,
    /// Set once the device has pulled the last of the soundtrack.
    ended: Arc<AtomicBool>,
    channels: u32,
    sample_rate: u32,
}

impl MovieAudio {
    /// Start the first audio track of `path`. `Ok(None)` when the file has no audio this reads.
    /// The audio starts paused.
    pub fn open(path: &Path) -> Result<Option<Self>, PlaybackError> {
        Self::open_at(path, 0.0)
    }

    /// As `open`, but starting at `start` seconds. See [`AudioSource::open_at`].
    pub fn open_at(path: &Path, start: f64) -> Result<Option<Self>, PlaybackError> {
        Self::open_track(path, 0, start)
    }

    /// As `open_at`, for the audio track at `track` among the file's audio tracks.
    pub fn open_track(
        path: &Path,
        track: usize,
        start: f64,
    ) -> Result<Option<Self>, PlaybackError> {
        match AudioSource::read_track(path, track)? {
            Some(source) => source.open_at(start).map(Some),
            None => Ok(None),
        }
    }

    /// Where playback is, in seconds, as the device has heard it so far.
    pub fn clock(&self) -> f64 {
        let samples = self.pulled.load(Ordering::Relaxed);
        samples as f64 / f64::from(self.channels * self.sample_rate)
    }

    /// Whether the soundtrack has played to its end.
    pub fn finished(&self) -> bool {
        self.ended.load(Ordering::Relaxed)
    }

    /// Set the output level, from 0 (silent) to 1 (full).
    pub fn set_volume(&self, volume: f32) {
        self.player.set_volume(volume);
    }

    pub fn play(&self) {
        self.player.play();
    }

    pub fn pause(&self) {
        self.player.pause();
    }
}

/// A movie's soundtrack, read once. Opening it at a position from here does not read the file's
/// tables again, which is most of what a seek used to cost.
pub struct AudioSource {
    path: PathBuf,
    track: Arc<AudioTrack>,
}

impl AudioSource {
    /// Read the first audio track of `path`. `None` when the file has no audio this reads.
    pub fn read(path: &Path) -> Result<Option<Self>, PlaybackError> {
        Self::read_track(path, 0)
    }

    /// Read the audio track at `track` among the file's audio tracks. `None` when there is no such
    /// track, or its codec is not one this reads.
    pub fn read_track(path: &Path, track: usize) -> Result<Option<Self>, PlaybackError> {
        let samples = read_audio_samples_track(path, track)
            .map_err(|error| PlaybackError::Open(error.to_string()))?;
        Ok(samples.map(|samples| AudioSource {
            path: path.to_path_buf(),
            track: Arc::new(samples),
        }))
    }

    /// Start the soundtrack at `start` seconds. The decoder starts at the frame before `start`, and
    /// the samples up to `start` are dropped, so the sound begins exactly there. Paused.
    pub fn open_at(&self, start: f64) -> Result<MovieAudio, PlaybackError> {
        let track = &self.track;
        let target = start * track.timescale as f64;
        let first = track
            .samples
            .iter()
            .rposition(|sample| (sample.presentation_time as f64) <= target)
            .unwrap_or(0);
        let frame_start = track.samples.get(first).map_or(0.0, |sample| {
            sample.presentation_time as f64 / track.timescale as f64
        });
        // From the start of the file, the decoder's priming is dropped as well.
        let priming = if first == 0 {
            track.priming_frames as f64
        } else {
            0.0
        };
        let skip_frames =
            (priming + (start - frame_start).max(0.0) * f64::from(track.sample_rate)) as usize;
        let skip = skip_frames * track.channels as usize;
        let stream = AudioStream {
            codec: track.codec,
            sample_rate: track.sample_rate,
            channels: track.channels,
            config: track.config.clone(),
        };
        let (channels, sample_rate) = (stream.channels, stream.sample_rate);
        let (sender, messages) = mpsc::sync_channel(CHUNKS_IN_FLIGHT);
        let path = self.path.clone();
        let decoded = Arc::clone(track);
        thread::spawn(move || decode(&path, decoded, stream, sender, first, skip));

        match messages.recv() {
            Ok(Message::Ready) => {}
            Ok(Message::Failed(reason)) => return Err(PlaybackError::Backend(reason)),
            _ => return Err(PlaybackError::Backend("the audio decoder stopped".into())),
        }

        let sink = open_output(Some(sample_rate))
            .map_err(|error| PlaybackError::Backend(error.to_string()))?;
        let pulled = Arc::new(AtomicU64::new(0));
        let ended = Arc::new(AtomicBool::new(false));
        let source = ChunkSource {
            messages,
            current: Vec::new(),
            position: 0,
            channels: nonzero_channels(channels)?,
            sample_rate: nonzero_rate(sample_rate)?,
            pulled: Arc::clone(&pulled),
            ended_flag: Arc::clone(&ended),
            ended: false,
        };
        let player = Player::connect_new(sink.mixer());
        player.pause();
        player.append(source);
        Ok(MovieAudio {
            _sink: sink,
            player,
            pulled,
            ended,
            channels,
            sample_rate,
        })
    }
}

fn nonzero_channels(channels: u32) -> Result<ChannelCount, PlaybackError> {
    u16::try_from(channels)
        .ok()
        .and_then(ChannelCount::new)
        .ok_or_else(|| PlaybackError::Backend("the audio has no channels".into()))
}

fn nonzero_rate(rate: u32) -> Result<SampleRate, PlaybackError> {
    SampleRate::new(rate)
        .ok_or_else(|| PlaybackError::Backend("the audio has no sample rate".into()))
}

/// The decoding thread: decode each frame of the track and send the samples on. The decoder is
/// opened here, since decoders are not sent between threads.
fn decode(
    path: &Path,
    track: Arc<AudioTrack>,
    stream: AudioStream,
    sender: SyncSender<Message>,
    first: usize,
    mut skip: usize,
) {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    let mut decoder = match audio::open(&stream) {
        Ok(decoder) => decoder,
        Err(error) => {
            let _ = sender.send(Message::Failed(error.to_string()));
            return;
        }
    };
    if sender.send(Message::Ready).is_err() {
        return;
    }
    let Ok(mut file) = File::open(path) else {
        let _ = sender.send(Message::Failed("could not reopen the file".into()));
        return;
    };
    for sample in &track.samples[first..] {
        let mut data = vec![0u8; sample.size as usize];
        if file.seek(SeekFrom::Start(sample.offset)).is_err() || file.read_exact(&mut data).is_err()
        {
            let _ = sender.send(Message::Failed("could not read an audio frame".into()));
            return;
        }
        match decoder.decode(&data) {
            Ok(mut chunk) => {
                // Samples before the requested start are dropped, so the sound starts there.
                let dropped = skip.min(chunk.len());
                chunk.drain(..dropped);
                skip -= dropped;
                if !chunk.is_empty() && sender.send(Message::Chunk(chunk)).is_err() {
                    return;
                }
            }
            Err(error) => {
                let _ = sender.send(Message::Failed(error.to_string()));
                return;
            }
        }
    }
    if let Ok(tail) = decoder.finish()
        && !tail.is_empty()
    {
        let _ = sender.send(Message::Chunk(tail));
    }
}

/// The source the device pulls. It plays decoded chunks in order, and silence while the decoder
/// is behind. It counts what it hands the device, which is the clock.
struct ChunkSource {
    messages: Receiver<Message>,
    current: Vec<f32>,
    position: usize,
    channels: ChannelCount,
    sample_rate: SampleRate,
    pulled: Arc<AtomicU64>,
    /// Shared with `MovieAudio`, so the window can see the end.
    ended_flag: Arc<AtomicBool>,
    ended: bool,
}

impl ChunkSource {
    /// Make the next chunk current, waiting briefly for it. `false` once the decoder has finished.
    fn refill(&mut self) -> bool {
        while self.position >= self.current.len() {
            if self.ended {
                return false;
            }
            match self.messages.recv_timeout(WAIT_FOR_CHUNK) {
                Ok(Message::Chunk(chunk)) => {
                    self.current = chunk;
                    self.position = 0;
                }
                Ok(Message::Ready) => {}
                Ok(Message::Failed(_)) | Err(RecvTimeoutError::Disconnected) => self.ended = true,
                Err(RecvTimeoutError::Timeout) => return true,
            }
        }
        true
    }
}

impl Iterator for ChunkSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if !self.refill() {
            self.ended_flag.store(true, Ordering::Relaxed);
            return None;
        }
        if self.position < self.current.len() {
            let sample = self.current[self.position];
            self.position += 1;
            self.pulled.fetch_add(1, Ordering::Relaxed);
            Some(sample)
        } else {
            // Behind the decoder: play a silent sample and keep the clock where it is.
            Some(0.0)
        }
    }
}

impl Source for ChunkSource {
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
        None
    }
}
