//! A source that plays one track and carries straight on into the next, so albums play without
//! a gap. The next track is opened ahead of time by the UI thread and handed over here.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};

use crate::symphonia_source::SymphoniaSource;

/// A track opened ahead of time, waiting to follow the current one.
pub(crate) type Upcoming = (PathBuf, Option<Duration>, SymphoniaSource);

/// State shared between the audio thread, which plays the chain, and the UI thread, which
/// arms the next track and reads the position.
pub(crate) struct Shared {
    /// Sample rate of every track in the chain. Tracks in another rate can't follow on.
    rate: u32,
    /// Frames played in the current track.
    frames: AtomicU64,
    /// The next track, if one has been armed. The audio thread takes it when the current ends.
    next: Mutex<Option<Upcoming>>,
    /// How many tracks the audio thread has moved on to.
    started_count: AtomicU64,
    /// The track the audio thread moved on to last, and its duration.
    started: Mutex<Option<(PathBuf, Option<Duration>)>>,
}

impl Shared {
    pub(crate) fn new(rate: u32) -> Self {
        Self {
            rate,
            frames: AtomicU64::new(0),
            next: Mutex::new(None),
            started_count: AtomicU64::new(0),
            started: Mutex::new(None),
        }
    }

    /// Position within the current track.
    pub(crate) fn position(&self) -> Duration {
        let frames = self.frames.load(Ordering::Relaxed);
        Duration::from_secs_f64(frames as f64 / f64::from(self.rate))
    }

    /// Set (or clear) the track that will follow the current one.
    pub(crate) fn arm(&self, upcoming: Option<Upcoming>) {
        if let Ok(mut slot) = self.next.lock() {
            *slot = upcoming;
        }
    }

    pub(crate) fn started_count(&self) -> u64 {
        self.started_count.load(Ordering::Acquire)
    }

    pub(crate) fn started(&self) -> Option<(PathBuf, Option<Duration>)> {
        self.started.lock().ok()?.clone()
    }
}

/// Plays the first track, then each armed track in turn.
pub(crate) struct Chain {
    inner: SymphoniaSource,
    shared: Arc<Shared>,
    /// Samples of the current frame already played.
    in_frame: u16,
}

impl Chain {
    pub(crate) fn new(first: SymphoniaSource, shared: Arc<Shared>) -> Self {
        Self {
            inner: first,
            shared,
            in_frame: 0,
        }
    }

    /// Move on to the armed track. Returns `None` when nothing is armed, or when the armed track
    /// has a different sample rate or channel count (rodio can't change those mid-stream).
    fn follow_next(&mut self) -> Option<()> {
        let (path, duration, source) = self.shared.next.lock().ok()?.take()?;
        if source.sample_rate() != self.inner.sample_rate()
            || source.channels() != self.inner.channels()
        {
            return None;
        }
        self.inner = source;
        self.in_frame = 0;
        self.shared.frames.store(0, Ordering::Relaxed);
        if let Ok(mut started) = self.shared.started.lock() {
            *started = Some((path, duration));
        }
        self.shared.started_count.fetch_add(1, Ordering::Release);
        Some(())
    }
}

impl Iterator for Chain {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if let Some(sample) = self.inner.next() {
            self.in_frame += 1;
            if self.in_frame >= self.inner.channels().get() {
                self.in_frame = 0;
                self.shared.frames.fetch_add(1, Ordering::Relaxed);
            }
            return Some(sample);
        }
        self.follow_next()?;
        self.next()
    }
}

impl Source for Chain {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        self.inner.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.inner.try_seek(pos)?;
        self.in_frame = 0;
        let frames = (pos.as_secs_f64() * f64::from(self.shared.rate)).round() as u64;
        self.shared.frames.store(frames, Ordering::Relaxed);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::symphonia_source::ErrorSlot;

    /// Write a 16-bit PCM WAV file.
    fn write_wav(path: &Path, rate: u32, channels: u16, samples: &[i16]) {
        let data_len = (samples.len() * 2) as u32;
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data_len).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&channels.to_le_bytes());
        out.extend_from_slice(&rate.to_le_bytes());
        out.extend_from_slice(&(rate * u32::from(channels) * 2).to_le_bytes());
        out.extend_from_slice(&(channels * 2).to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {
            out.extend_from_slice(&sample.to_le_bytes());
        }
        std::fs::write(path, out).unwrap();
    }

    fn temp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("player-chain-{}-{name}.wav", std::process::id()))
    }

    fn open(path: &Path) -> SymphoniaSource {
        SymphoniaSource::open(path, ErrorSlot::default()).unwrap()
    }

    #[test]
    fn the_armed_track_follows_with_no_gap() {
        let (a, b) = (temp("a"), temp("b"));
        write_wav(&a, 8000, 1, &[16000; 100]);
        write_wav(&b, 8000, 1, &[-16000; 50]);

        let shared = Arc::new(Shared::new(8000));
        let mut chain = Chain::new(open(&a), Arc::clone(&shared));
        shared.arm(Some((b.clone(), None, open(&b))));
        let samples: Vec<f32> = chain.by_ref().collect();

        assert_eq!(samples.len(), 150, "both tracks, back to back");
        assert!(
            samples[99] > 0.0 && samples[100] < 0.0,
            "the switch is at sample 100"
        );
        assert_eq!(shared.started_count(), 1);
        assert_eq!(shared.started().map(|(path, _)| path), Some(b.clone()));
        assert_eq!(shared.position(), Duration::from_secs_f64(50.0 / 8000.0));
        let _ = std::fs::remove_file(a);
        let _ = std::fs::remove_file(b);
    }

    #[test]
    fn a_track_in_another_rate_is_not_followed() {
        let (a, b) = (temp("c"), temp("d"));
        write_wav(&a, 8000, 1, &[16000; 100]);
        write_wav(&b, 44100, 1, &[16000; 100]);

        let shared = Arc::new(Shared::new(8000));
        let mut chain = Chain::new(open(&a), Arc::clone(&shared));
        shared.arm(Some((b.clone(), None, open(&b))));
        let samples: Vec<f32> = chain.by_ref().collect();

        assert_eq!(samples.len(), 100);
        assert_eq!(shared.started_count(), 0);
        let _ = std::fs::remove_file(a);
        let _ = std::fs::remove_file(b);
    }
}
