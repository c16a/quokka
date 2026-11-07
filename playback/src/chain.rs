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
    /// Length of a crossfade, in frames. Zero when crossfade is off.
    fade_frames: AtomicU64,
}

impl Shared {
    pub(crate) fn new(rate: u32) -> Self {
        Self {
            rate,
            frames: AtomicU64::new(0),
            next: Mutex::new(None),
            started_count: AtomicU64::new(0),
            started: Mutex::new(None),
            fade_frames: AtomicU64::new(0),
        }
    }

    /// Set how long a crossfade lasts, in frames. Zero turns crossfade off.
    pub(crate) fn set_fade(&self, frames: u64) {
        self.fade_frames.store(frames, Ordering::Relaxed);
    }

    pub(crate) fn fade(&self) -> u64 {
        self.fade_frames.load(Ordering::Relaxed)
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

/// Plays the first track, then each armed track in turn. When crossfade is on, the armed track
/// starts while the current one is still playing: the current track fades out under it.
pub(crate) struct Chain {
    /// The track now playing, which is the one the position is measured in.
    inner: SymphoniaSource,
    shared: Arc<Shared>,
    /// Samples of the current frame already played.
    in_frame: u16,
    /// The track fading out during a crossfade, if one is happening.
    outgoing: Option<SymphoniaSource>,
    /// Frames mixed so far in the crossfade, and the length of the crossfade in frames.
    fade_done: u64,
    fade_total: u64,
}

impl Chain {
    pub(crate) fn new(first: SymphoniaSource, shared: Arc<Shared>) -> Self {
        Self {
            inner: first,
            shared,
            in_frame: 0,
            outgoing: None,
            fade_done: 0,
            fade_total: 0,
        }
    }

    /// Count one sample of the track now playing, and a frame once all its channels have played.
    fn count_sample(&mut self) {
        self.in_frame += 1;
        if self.in_frame >= self.inner.channels().get() {
            self.in_frame = 0;
            self.shared.frames.fetch_add(1, Ordering::Relaxed);
            self.fade_done += 1;
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
        self.start_reported(path, duration);
        Some(())
    }

    /// Record that the output has moved on to `path`.
    fn start_reported(&mut self, path: PathBuf, duration: Option<Duration>) {
        if let Ok(mut started) = self.shared.started.lock() {
            *started = Some((path, duration));
        }
        self.shared.started_count.fetch_add(1, Ordering::Release);
    }

    /// Begin a crossfade when the current track is within one fade of its end and another track is
    /// armed with the same format. The armed track becomes the one playing; the old one fades out.
    fn maybe_start_fade(&mut self) {
        let fade = self.shared.fade();
        if fade == 0 {
            return;
        }
        let Some(total) = self.inner.total_duration() else {
            return;
        };
        let total_frames = (total.as_secs_f64() * f64::from(self.shared.rate)).round() as u64;
        // Never fade for longer than half the track, so a short track can't start fading at once.
        let fade = fade.min(total_frames / 2);
        let played = self.shared.frames.load(Ordering::Relaxed);
        if fade == 0 || total_frames.saturating_sub(played) > fade {
            return;
        }
        let upcoming = {
            let Ok(mut slot) = self.shared.next.lock() else {
                return;
            };
            let compatible = matches!(
                slot.as_ref(),
                Some((_, _, source))
                    if source.sample_rate() == self.inner.sample_rate()
                        && source.channels() == self.inner.channels()
            );
            if compatible { slot.take() } else { None }
        };
        let Some((path, duration, source)) = upcoming else {
            return;
        };
        self.outgoing = Some(std::mem::replace(&mut self.inner, source));
        self.fade_done = 0;
        self.fade_total = fade;
        self.in_frame = 0;
        self.shared.frames.store(0, Ordering::Relaxed);
        self.start_reported(path, duration);
    }

    /// The next sample of a crossfade: the outgoing track fading out, the incoming one fading in.
    /// `None` when the incoming track has no more samples.
    fn mix_next(&mut self) -> Option<f32> {
        let incoming = self.inner.next()?;
        let outgoing = self.outgoing.as_mut().and_then(|track| track.next());
        let t = (self.fade_done as f32 / self.fade_total.max(1) as f32).min(1.0);
        let mixed = match outgoing {
            Some(out) => out * (1.0 - t) + incoming * t,
            None => incoming,
        };
        self.count_sample();
        if outgoing.is_none() || self.fade_done >= self.fade_total {
            self.outgoing = None;
        }
        Some(mixed)
    }
}

impl Iterator for Chain {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.outgoing.is_none() {
            self.maybe_start_fade();
        }
        if self.outgoing.is_some() {
            if let Some(sample) = self.mix_next() {
                return Some(sample);
            }
            // The incoming track ended before the fade did, so the fade is over.
            self.outgoing = None;
        }
        if let Some(sample) = self.inner.next() {
            self.count_sample();
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
        self.outgoing = None;
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
        std::env::temp_dir().join(format!("quokka-chain-{}-{name}.wav", std::process::id()))
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

    #[test]
    fn crossfade_mixes_the_end_of_one_track_into_the_next() {
        let (a, b) = (temp("fade-a"), temp("fade-b"));
        write_wav(&a, 8000, 1, &[16000; 100]);
        write_wav(&b, 8000, 1, &[-16000; 50]);

        let shared = Arc::new(Shared::new(8000));
        shared.set_fade(20);
        let mut chain = Chain::new(open(&a), Arc::clone(&shared));
        shared.arm(Some((b.clone(), None, open(&b))));
        let samples: Vec<f32> = chain.by_ref().collect();

        // The first 80 samples are the first track alone, then 20 mixed, then 30 of the second.
        assert_eq!(samples.len(), 130);
        assert!(
            samples[79] > 0.0,
            "the first track is still audible before the fade"
        );
        assert!(
            samples[120] < 0.0,
            "the second track is playing once the fade is over"
        );
        assert!(
            samples[99] < 0.0 && samples[80] > 0.0,
            "the sign changes inside the fade"
        );
        assert_eq!(shared.started_count(), 1);
        assert_eq!(shared.position(), Duration::from_secs_f64(50.0 / 8000.0));
    }
}
