//! Video played from the decoders, for the app: frames decode on a worker, the sound plays through
//! the output device, and the sound's clock (or the wall clock, once the sound has ended) decides
//! which frame is on screen. It implements [`Player`], so the playback bar drives it like any other.

use std::path::Path;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use player_core::video::{Cue, read_subtitles};

use std::sync::Arc;

use crate::decode::frames::Yuv;
use crate::decode::player::{FramePlayer, VideoFrame, VideoSource};
use crate::movie_audio::{AudioSource, MovieAudio};
use crate::{PlaybackError, Player, PlayerState, Result};

/// The parts of a file that are read on a worker thread, ready for the window's thread to start.
struct Prepared {
    /// The position the file starts from, in seconds.
    start: f64,
    video: VideoSource,
    /// The soundtrack's tables, kept for seeking.
    audio_source: Option<AudioSource>,
    /// The soundtrack, opened at `start`, with its output device.
    audio: Option<MovieAudio>,
    frames: FramePlayer,
    cues: Vec<Cue>,
}

/// Read a file's tables, open its decoders and sound output at `start`, and read its subtitles.
/// All of it runs off the window's thread: this is the slow part of loading.
fn prepare(path: &Path, start: f64) -> std::result::Result<Prepared, String> {
    let video = VideoSource::read(path).map_err(|error| error.to_string())?;
    let audio_source = AudioSource::read(path).map_err(|error| error.to_string())?;
    let frames = video.open_at(start).map_err(|error| error.to_string())?;
    let audio = match &audio_source {
        Some(source) => Some(source.open_at(start).map_err(|error| error.to_string())?),
        None => None,
    };
    let cues = read_subtitles(path).unwrap_or_default();
    Ok(Prepared {
        start,
        video,
        audio_source,
        audio,
        frames,
        cues,
    })
}

/// A movie being played from the decoders.
#[derive(Default)]
pub struct FrameVideo {
    frames: Option<FramePlayer>,
    audio: Option<MovieAudio>,
    /// The file's samples, read once, so that seeking reopens the decoders without reading the file again.
    video_source: Option<VideoSource>,
    audio_source: Option<AudioSource>,
    /// The subtitles of the file, and whether they are shown.
    cues: Vec<Cue>,
    subtitles_on: bool,
    /// The position the sound's clock started from: zero, or the last seek target.
    audio_offset: f64,
    /// A frame whose time has not come up yet.
    next: Option<VideoFrame>,
    /// The frame on screen, and how many frames have been shown, so the window knows when to redraw.
    image: Option<Arc<Yuv>>,
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
    /// The file being read on a worker thread, until it is ready.
    loading: Option<Receiver<std::result::Result<Prepared, String>>>,
    /// Whether play was pressed while the file was still loading.
    want_play: bool,
    /// Why the file could not be loaded, if it could not.
    failure: Option<String>,
}

impl FrameVideo {
    pub fn new() -> Self {
        Self {
            subtitles_on: true,
            ..Self::default()
        }
    }

    /// The caption due at the current position, if subtitles are shown and one is due.
    pub fn caption(&self) -> Option<String> {
        if !self.subtitles_on || !self.loaded {
            return None;
        }
        let now = self.clock();
        let due: Vec<&str> = self
            .cues
            .iter()
            .filter(|cue| cue.start <= now && now < cue.end)
            .map(|cue| cue.text.as_str())
            .collect();
        (!due.is_empty()).then(|| due.join("\n"))
    }

    /// Show or hide the subtitles.
    pub fn toggle_subtitles(&mut self) {
        self.subtitles_on = !self.subtitles_on;
    }

    /// Take the frames whose times have come. `true` when the frame on screen has changed.
    pub fn poll(&mut self) -> bool {
        if let Some(receiver) = self.loading.as_ref() {
            match receiver.try_recv() {
                Ok(Ok(prepared)) => {
                    self.loading = None;
                    if let Err(error) = self.finish_loading(prepared) {
                        self.failure = Some(error.to_string());
                    }
                }
                Ok(Err(reason)) => {
                    self.loading = None;
                    self.failure = Some(reason);
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.loading = None;
                    self.failure = Some("loading stopped".into());
                }
            }
            return false;
        }
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
                    self.image = Some(Arc::new(frame.image));
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

    /// Start what the worker prepared: the picture and the sound were both opened there.
    fn finish_loading(&mut self, prepared: Prepared) -> Result<()> {
        self.duration = prepared.video.duration();
        self.audio_offset = prepared.start;
        self.wall_base = prepared.start;
        self.audio = prepared.audio;
        self.frames = Some(prepared.frames);
        self.video_source = Some(prepared.video);
        self.audio_source = prepared.audio_source;
        self.cues = prepared.cues;
        self.loaded = true;
        if std::mem::take(&mut self.want_play) {
            self.play()?;
        }
        Ok(())
    }

    /// Load `path` to start at `start` seconds, on a worker thread. It plays once it is ready, if
    /// play was pressed meanwhile. Loading a movie already loaded replaces it.
    pub fn load_at(&mut self, path: &Path, start: f64) -> Result<()> {
        *self = FrameVideo::new();
        let (sender, receiver) = std::sync::mpsc::channel();
        let path = path.to_path_buf();
        std::thread::spawn(move || {
            let _ = sender.send(prepare(&path, start));
        });
        self.loading = Some(receiver);
        Ok(())
    }

    /// Why the file could not be loaded, if it could not.
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    /// The frame on screen, if one has been shown.
    pub fn image(&self) -> Option<&Arc<Yuv>> {
        self.image.as_ref()
    }

    /// How many frames have been shown. It changes whenever the picture does.
    pub fn shown(&self) -> u64 {
        self.shown
    }

    /// Whether anything is decoding or playing, so the window should keep redrawing.
    pub fn active(&self) -> bool {
        self.loading.is_some() || (self.loaded && self.playing && !self.ended)
    }

    /// Where playback is, in seconds.
    fn clock(&self) -> f64 {
        match &self.audio {
            Some(audio) if !self.sound_done => self.audio_offset + audio.clock(),
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
        self.load_at(path, 0.0)
    }

    fn play(&mut self) -> Result<()> {
        if self.loading.is_some() {
            self.want_play = true;
            return Ok(());
        }
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
        if self.loading.is_some() {
            self.want_play = false;
            return Ok(());
        }
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

    fn seek(&mut self, position: Duration) -> Result<()> {
        let Some(video) = self.video_source.as_ref() else {
            return Ok(());
        };
        let target = position.as_secs_f64();
        let frames = video
            .open_at(target)
            .map_err(|error| PlaybackError::Open(error.to_string()))?;
        let audio = match self.audio_source.as_ref() {
            Some(source) => Some(source.open_at(target)?),
            None => None,
        };
        let was_playing = self.playing;
        self.frames = Some(frames);
        self.audio = audio;
        self.audio_offset = target;
        self.next = None;
        self.sound_done = false;
        self.ended = false;
        self.wall_base = target;
        self.wall_started = None;
        if was_playing { self.play() } else { Ok(()) }
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
