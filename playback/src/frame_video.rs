//! Video played from the decoders, for the app: frames decode on a worker, the sound plays through
//! the output device, and the sound's clock (or the wall clock, once the sound has ended) decides
//! which frame is on screen. It implements [`Player`], so the playback bar drives it like any other.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::time::{Duration, Instant};

use quokka_core::video::{Cue, read_subtitles};

use std::sync::Arc;

use crate::decode::frames::Yuv;
use crate::decode::player::{FramePlayer, VideoFrame, VideoSource};
use crate::movie_audio::{AudioSource, MovieAudio};
use crate::{PlaybackError, Player, PlayerState, Result};

/// How long a frame step waits for the decoder, or for a seek to open, before it gives up.
const STEP_WAIT: Duration = Duration::from_secs(1);

/// How often a frame step checks the decoder while it waits.
const STEP_POLL: Duration = Duration::from_millis(2);

/// The parts of a file that are read on a worker thread, ready for the window's thread to start.
struct Prepared {
    /// The position the file starts from, in seconds.
    start: f64,
    /// The audio track the soundtrack was read from.
    audio_track: usize,
    video: VideoSource,
    /// The soundtrack's tables, kept for seeking.
    audio_source: Option<AudioSource>,
    /// The soundtrack, opened at `start`, with its output device.
    audio: Option<MovieAudio>,
    frames: FramePlayer,
    cues: Vec<Cue>,
}

/// A seek on a worker thread, until its decoder and sound are open.
struct PendingSeek {
    target: f64,
    /// Whether playback runs once the seek completes. Play and pause pressed meanwhile change it.
    was_playing: bool,
    opened: Receiver<std::result::Result<Opened, String>>,
}

/// The decoder and the sound output, opened at one position.
type Opened = (FramePlayer, Option<MovieAudio>);

/// Open the picture's decoder and the soundtrack's output at `start`.
fn open_streams(
    video: &VideoSource,
    audio_source: Option<&AudioSource>,
    start: f64,
) -> std::result::Result<Opened, String> {
    let frames = video.open_at(start).map_err(|error| error.to_string())?;
    let audio = match audio_source {
        Some(source) => Some(source.open_at(start).map_err(|error| error.to_string())?),
        None => None,
    };
    Ok((frames, audio))
}

/// Read a file's tables, open its decoders and sound output at `start`, and read its subtitles.
/// All of it runs off the window's thread: this is the slow part of loading.
fn prepare(path: &Path, start: f64, audio_track: usize) -> std::result::Result<Prepared, String> {
    let video = VideoSource::read(path).map_err(|error| error.to_string())?;
    let audio_source =
        AudioSource::read_track(path, audio_track).map_err(|error| error.to_string())?;
    let (frames, audio) = open_streams(&video, audio_source.as_ref(), start)?;
    let cues = read_subtitles(path).unwrap_or_default();
    Ok(Prepared {
        start,
        audio_track,
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
    video_source: Option<Arc<VideoSource>>,
    audio_source: Option<Arc<AudioSource>>,
    /// The seek being opened on a worker thread, if one is.
    seeking: Option<PendingSeek>,
    /// The subtitles of the file, and whether they are shown.
    cues: Vec<Cue>,
    subtitles_on: bool,
    /// The position the sound's clock started from: zero, or the last seek target.
    audio_offset: f64,
    /// A frame whose time has not come up yet.
    next: Option<VideoFrame>,
    /// The frame on screen, its time, and how many frames have been shown, so the window knows when
    /// to redraw.
    image: Option<Arc<Yuv>>,
    image_time: Option<f64>,
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
    /// The file being played, so that its soundtrack can be read again for another track.
    path: Option<PathBuf>,
    /// The audio track the soundtrack is read from, among the file's audio tracks.
    audio_track: usize,
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
        let now = self
            .seeking
            .as_ref()
            .map_or_else(|| self.clock(), |pending| pending.target);
        // The cues are sorted by start, so those starting later cannot be due.
        let started = self.cues.partition_point(|cue| cue.start <= now);
        let due: Vec<&str> = self.cues[..started]
            .iter()
            .filter(|cue| now < cue.end)
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
        if let Some(pending) = self.seeking.take() {
            match pending.opened.try_recv() {
                Ok(Ok((frames, audio))) => {
                    if let Err(error) =
                        self.finish_seek(pending.target, pending.was_playing, frames, audio)
                    {
                        self.failure = Some(error.to_string());
                    }
                }
                Ok(Err(reason)) => self.failure = Some(reason),
                Err(TryRecvError::Empty) => {
                    self.seeking = Some(pending);
                    return false;
                }
                Err(TryRecvError::Disconnected) => {
                    self.failure = Some("seeking stopped".into());
                }
            }
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
                    self.set_picture(frame);
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
        let loaded_track = prepared.audio_track;
        self.duration = prepared.video.duration();
        self.audio_offset = prepared.start;
        self.wall_base = prepared.start;
        self.audio = prepared.audio;
        self.frames = Some(prepared.frames);
        self.video_source = Some(Arc::new(prepared.video));
        self.audio_source = prepared.audio_source.map(Arc::new);
        let mut cues = prepared.cues;
        cues.sort_by(|a, b| a.start.total_cmp(&b.start));
        self.cues = cues;
        self.loaded = true;
        // The track may have been chosen while the file loaded, so the sound is read again for it.
        if loaded_track != self.audio_track {
            let track = self.audio_track;
            self.set_audio_track(track)?;
        }
        if std::mem::take(&mut self.want_play) {
            self.play()?;
        }
        Ok(())
    }

    /// Start the decoder and sound that a seek opened, and carry on from the target. Playback
    /// resumes if it was running when the seek began.
    fn finish_seek(
        &mut self,
        target: f64,
        was_playing: bool,
        frames: FramePlayer,
        audio: Option<MovieAudio>,
    ) -> Result<()> {
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

    /// Load `path` to start at `start` seconds, on a worker thread. It plays once it is ready, if
    /// play was pressed meanwhile. Loading a movie already loaded replaces it.
    pub fn load_at(&mut self, path: &Path, start: f64) -> Result<()> {
        *self = FrameVideo::new();
        let path = path.to_path_buf();
        self.path = Some(path.clone());
        // The track is back at 0 after the reset, so each file starts on its first audio track.
        let audio_track = self.audio_track;
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(prepare(&path, start, audio_track));
        });
        self.loading = Some(receiver);
        Ok(())
    }

    /// Choose which of the file's audio tracks plays. A loaded file reopens its sound at the
    /// current position, keeping the playing or paused state. A file still loading picks the track
    /// up when it is ready.
    pub fn set_audio_track(&mut self, track: usize) -> Result<()> {
        let path = match self.path.clone() {
            Some(path) if self.loaded && self.loading.is_none() => path,
            _ => {
                self.audio_track = track;
                return Ok(());
            }
        };
        let Some(source) = AudioSource::read_track(&path, track)? else {
            return Err(PlaybackError::Backend(format!(
                "the file has no audio track {track}"
            )));
        };
        self.audio_track = track;
        self.audio_source = Some(Arc::new(source));
        // Seeking to the current position opens the decoders and the sound from the new source.
        let position = self.position().unwrap_or_default();
        self.seek(position)
    }

    /// Why the file could not be loaded, if it could not.
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    /// The frame on screen, if one has been shown.
    pub fn image(&self) -> Option<&Arc<Yuv>> {
        self.image.as_ref()
    }

    /// The time of the frame on screen, in seconds, if one has been shown.
    pub fn image_time(&self) -> Option<f64> {
        self.image_time
    }

    /// Show the frame before the one on screen (`forward` false), or the frame after it, while
    /// paused. Playback stays paused. `frame_secs` is the length of one frame.
    pub fn step(&mut self, forward: bool, frame_secs: f64) -> Result<()> {
        if !self.loaded {
            return Ok(());
        }
        self.pause()?;
        if !self.settle_seek()? {
            return Ok(());
        }
        if forward {
            if let Some(frame) = self.next_frame_waiting() {
                self.show_stepped(frame);
            }
            Ok(())
        } else {
            self.step_back(frame_secs)
        }
    }

    /// How many frames have been shown. It changes whenever the picture does.
    pub fn shown(&self) -> u64 {
        self.shown
    }

    /// Whether anything is decoding or playing, so the window should keep redrawing.
    pub fn active(&self) -> bool {
        self.loading.is_some()
            || self.seeking.is_some()
            || (self.loaded && self.playing && !self.ended)
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

    /// Put `frame` on screen as the clock reaches it.
    fn set_picture(&mut self, frame: VideoFrame) {
        self.image_time = Some(frame.time);
        self.image = Some(Arc::new(frame.image));
        self.shown += 1;
    }

    /// Put `frame` on screen and move the paused clock to its time, so that the position and the
    /// picture agree.
    fn show_stepped(&mut self, frame: VideoFrame) {
        let time = frame.time;
        self.set_picture(frame);
        self.move_clock(time);
    }

    /// Set the paused position to `time`. The sound stays paused: its offset moves so that its own
    /// clock reads `time`, which is how a seek sets the position too.
    fn move_clock(&mut self, time: f64) {
        if let Some(audio) = &self.audio
            && !self.sound_done
        {
            self.audio_offset = time - audio.clock();
        }
        self.wall_base = time;
    }

    /// Wait briefly for a seek to open its decoder and sound, and start them. `false` when the seek
    /// has not finished or has failed, so the step is not taken.
    fn settle_seek(&mut self) -> Result<bool> {
        let Some(pending) = self.seeking.take() else {
            return Ok(true);
        };
        match pending.opened.recv_timeout(STEP_WAIT) {
            Ok(Ok((frames, audio))) => {
                self.finish_seek(pending.target, false, frames, audio)?;
                Ok(true)
            }
            Ok(Err(reason)) => {
                self.failure = Some(reason);
                Ok(false)
            }
            Err(RecvTimeoutError::Timeout) => {
                self.seeking = Some(pending);
                Ok(false)
            }
            Err(RecvTimeoutError::Disconnected) => {
                self.failure = Some("seeking stopped".into());
                Ok(false)
            }
        }
    }

    /// The next frame, waiting briefly for the decoder to produce one. `None` when the file has
    /// ended, or when no frame came in time.
    fn next_frame_waiting(&mut self) -> Option<VideoFrame> {
        if let Some(frame) = self.next.take() {
            return Some(frame);
        }
        let deadline = Instant::now() + STEP_WAIT;
        loop {
            let frames = self.frames.as_mut()?;
            match frames.next_frame() {
                Ok(Some(frame)) => return Some(frame),
                Ok(None) if !frames.finished() && Instant::now() < deadline => {
                    std::thread::sleep(STEP_POLL);
                }
                Ok(None) => return None,
                Err(_) => {
                    self.ended = true;
                    return None;
                }
            }
        }
    }

    /// Show the frame before the picture on screen. The decoder is reopened a little before it and
    /// read on up to the picture's time, keeping the last frame before it.
    fn step_back(&mut self, frame_secs: f64) -> Result<()> {
        let clock = self.clock();
        // The picture is the last frame at or before the clock. Stepping from its time, not the
        // clock's, moves a frame even when the clock is partway through one.
        let current = match self.image_time {
            Some(time) if time <= clock && clock - time < frame_secs => time,
            _ => clock,
        };
        self.seek(Duration::from_secs_f64(
            (current - 1.5 * frame_secs).max(0.0),
        ))?;
        if !self.settle_seek()? {
            return Ok(());
        }
        let mut decoded = Vec::new();
        while let Some(frame) = self.next_frame_waiting() {
            let reached = frame.time >= current;
            decoded.push(frame);
            if reached {
                break;
            }
        }
        let times: Vec<f64> = decoded.iter().map(|frame| frame.time).collect();
        let Some(index) = backward_index(&times, current) else {
            return Ok(());
        };
        // The frame that reached the picture's time waits in `next`, so that `poll` shows it on time.
        let mut rest = decoded.split_off(index + 1);
        if let Some(frame) = decoded.pop() {
            self.show_stepped(frame);
        }
        self.next = rest.pop();
        Ok(())
    }
}

/// The index of the frame a backward step shows, among frame times in order, ending with the first
/// frame at or after `current`: the last frame before `current`, or that first frame when none is.
fn backward_index(times: &[f64], current: f64) -> Option<usize> {
    let before = times.partition_point(|&time| time < current);
    if before > 0 {
        Some(before - 1)
    } else if times.is_empty() {
        None
    } else {
        Some(0)
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
        if let Some(pending) = self.seeking.as_mut() {
            pending.was_playing = true;
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
        if let Some(pending) = self.seeking.as_mut() {
            pending.was_playing = false;
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
        let Some(video) = self.video_source.clone() else {
            return Ok(());
        };
        let target = position.as_secs_f64();
        // A seek that is already pending keeps the playing state it was started with.
        let was_playing = match self.seeking.take() {
            Some(pending) => pending.was_playing,
            None => {
                let playing = self.playing;
                self.pause()?;
                playing
            }
        };
        let audio_source = self.audio_source.clone();
        let (sender, opened) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(open_streams(&video, audio_source.as_deref(), target));
        });
        self.seeking = Some(PendingSeek {
            target,
            was_playing,
            opened,
        });
        Ok(())
    }

    fn position(&self) -> Option<Duration> {
        if let Some(pending) = self.seeking.as_ref() {
            return Some(Duration::from_secs_f64(pending.target));
        }
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

#[cfg(test)]
mod tests {
    use super::backward_index;

    #[test]
    fn backward_step_takes_the_last_frame_before_the_picture() {
        let times = [0.0, 0.04, 0.08, 0.12];
        assert_eq!(backward_index(&times, 0.12), Some(2));
        assert_eq!(backward_index(&times, 0.10), Some(2));
    }

    #[test]
    fn backward_step_takes_the_first_frame_when_none_is_before() {
        assert_eq!(backward_index(&[0.0, 0.04], 0.0), Some(0));
        assert_eq!(backward_index(&[0.12], 0.12), Some(0));
        assert_eq!(backward_index(&[], 1.0), None);
    }
}
