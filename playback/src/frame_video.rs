//! Video played from the decoders, for the app: frames decode on a worker, the sound plays through
//! the output device, and the sound's clock (or the wall clock, once the sound has ended) decides
//! which frame is on screen. It implements [`Player`], so the playback bar drives it like any other.

use std::path::Path;
use std::time::{Duration, Instant};

use crate::decode::frames::Rgba;
use crate::decode::player::{FramePlayer, VideoFrame};
use crate::movie_audio::MovieAudio;
use crate::{PlaybackError, Player, PlayerState, Result};

/// A movie being played from the decoders.
#[derive(Default)]
pub struct FrameVideo {
    frames: Option<FramePlayer>,
    audio: Option<MovieAudio>,
    /// A frame whose time has not come up yet.
    next: Option<VideoFrame>,
    /// The frame on screen, and how many frames have been shown, so the window knows when to redraw.
    image: Option<Rgba>,
    shown: u64,
    /// Whether the sound has ended, and the wall clock has taken over from it.
    sound_done: bool,
    /// Wall-clock playback: the position reached so far, and when it last started moving.
    wall_base: f64,
    wall_started: Option<Instant>,
    playing: bool,
    loaded: bool,
    ended: bool,
    duration: Option<Duration>,
}

impl FrameVideo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Take the frames whose times have come. `true` when the frame on screen has changed.
    pub fn poll(&mut self) -> bool {
        if !self.loaded {
            return false;
        }
        self.check_sound_end();
        let clock = self.clock();
        let mut changed = false;
        loop {
            if self.next.is_none() {
                let Some(frames) = self.frames.as_mut() else {
                    break;
                };
                match frames.next_frame() {
                    Ok(Some(frame)) => self.next = Some(frame),
                    Ok(None) => break,
                    Err(_) => {
                        self.ended = true;
                        break;
                    }
                }
            }
            match self.next.take() {
                Some(frame) if frame.time <= clock => {
                    self.image = Some(frame.image);
                    self.shown += 1;
                    changed = true;
                }
                other => {
                    self.next = other;
                    break;
                }
            }
        }
        if self.frames.as_ref().is_some_and(FramePlayer::finished)
            && self.next.is_none()
            && self.sound_done_or_none()
        {
            self.ended = true;
            self.stop_wall();
        }
        changed
    }

    /// The frame on screen, if one has been shown.
    pub fn image(&self) -> Option<&Rgba> {
        self.image.as_ref()
    }

    /// How many frames have been shown. It changes whenever the picture does.
    pub fn shown(&self) -> u64 {
        self.shown
    }

    /// Whether anything is decoding or playing, so the window should keep redrawing.
    pub fn active(&self) -> bool {
        self.loaded && self.playing && !self.ended
    }

    /// Where playback is, in seconds.
    fn clock(&self) -> f64 {
        match &self.audio {
            Some(audio) if !self.sound_done => audio.clock(),
            _ => self.wall_position(),
        }
    }

    fn wall_position(&self) -> f64 {
        self.wall_base
            + self
                .wall_started
                .map_or(0.0, |started| started.elapsed().as_secs_f64())
    }

    /// Start the wall clock from wherever the sound has got to, once the sound has ended.
    fn check_sound_end(&mut self) {
        if self.sound_done {
            return;
        }
        if let Some(audio) = &self.audio
            && audio.finished()
        {
            self.sound_done = true;
            self.wall_base = audio.clock();
            if self.playing {
                self.wall_started = Some(Instant::now());
            }
        }
    }

    fn sound_done_or_none(&self) -> bool {
        self.audio.is_none() || self.sound_done
    }

    /// Stop the wall clock where it is.
    fn stop_wall(&mut self) {
        if let Some(started) = self.wall_started.take() {
            self.wall_base += started.elapsed().as_secs_f64();
        }
    }
}

impl Player for FrameVideo {
    fn load(&mut self, path: &Path) -> Result<()> {
        *self = FrameVideo::new();
        let frames =
            FramePlayer::open(path).map_err(|error| PlaybackError::Open(error.to_string()))?;
        self.duration = frames.duration();
        self.frames = Some(frames);
        self.audio = MovieAudio::open(path)?;
        self.loaded = true;
        Ok(())
    }

    fn play(&mut self) -> Result<()> {
        if !self.loaded || self.ended {
            return Ok(());
        }
        self.playing = true;
        match &self.audio {
            Some(audio) if !self.sound_done => audio.play(),
            _ => self.wall_started = Some(Instant::now()),
        }
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        if !self.loaded {
            return Ok(());
        }
        self.playing = false;
        match &self.audio {
            Some(audio) if !self.sound_done => audio.pause(),
            _ => self.stop_wall(),
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        *self = FrameVideo::new();
        Ok(())
    }

    fn seek(&mut self, _position: Duration) -> Result<()> {
        // Seeking needs the decoders restarted from a keyframe. That comes after the playback
        // controls are settled.
        Err(PlaybackError::Backend(
            "seeking in video is not available yet".into(),
        ))
    }

    fn position(&self) -> Option<Duration> {
        self.loaded
            .then(|| Duration::from_secs_f64(self.clock().max(0.0)))
    }

    fn duration(&self) -> Option<Duration> {
        self.duration
    }

    fn state(&self) -> PlayerState {
        if !self.loaded {
            PlayerState::Idle
        } else if self.ended {
            PlayerState::Ended
        } else if self.playing {
            PlayerState::Playing
        } else {
            PlayerState::Paused
        }
    }

    fn set_volume(&mut self, volume: f32) -> Result<()> {
        if let Some(audio) = &self.audio {
            audio.set_volume(volume);
        }
        Ok(())
    }
}
