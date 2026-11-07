//! Video playback through libmpv. mpv decodes the file and opens its own window for the picture.
//! Audio-only files stay on the rodio backend.
//!
//! libmpv's macOS video output hands window work to the main thread and waits for it. The app's UI
//! runs on the main thread, so every libmpv call happens on a worker thread instead. The methods
//! here only send commands and read a snapshot, so they never wait on mpv.

use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use libmpv2::Mpv;

use crate::{PlaybackError, Player, PlayerState, Result};

/// How often the worker reads mpv's state when no command is waiting.
const POLL: Duration = Duration::from_millis(50);

/// Highest volume mpv applies, as a percentage. 100 is the file's own level.
const MAX_VOLUME: f64 = 130.0;

/// What the UI reads: the worker thread keeps this up to date.
#[derive(Debug, Default, Clone)]
struct Snapshot {
    /// A file is open (or is opening).
    loaded: bool,
    paused: bool,
    /// mpv reached the end of the file.
    ended: bool,
    position: Option<f64>,
    duration: Option<f64>,
    /// Why the file stopped early, if it did. Cleared when a file loads.
    failure: Option<String>,
    /// mpv has started the file. Until then an idle mpv is only waiting for it to start.
    opened: bool,
}

/// A request from the UI to the worker thread.
enum Command {
    Load(String),
    Play,
    Pause,
    Stop,
    Seek(f64),
    Volume(f64),
    Speed(f64),
}

/// `Player` that plays through libmpv on its own thread.
pub struct MpvPlayer {
    commands: Sender<Command>,
    snapshot: Arc<Mutex<Snapshot>>,
}

impl MpvPlayer {
    /// Start mpv with no file loaded. Fails when libmpv cannot be initialised.
    pub fn new() -> Result<Self> {
        Self::start("gpu")
    }

    /// Like `new`, but the picture is decoded and discarded instead of shown. Use this where no
    /// window can be opened, such as a test process that never created the macOS application.
    pub fn without_window() -> Result<Self> {
        Self::start("null")
    }

    /// A cloneable handle to this player, for controls that run on the main thread.
    pub fn handle(&self) -> MpvHandle {
        MpvHandle {
            commands: self.commands.clone(),
            snapshot: Arc::clone(&self.snapshot),
        }
    }

    fn start(video_output: &'static str) -> Result<Self> {
        let (commands, requests) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();
        let snapshot = Arc::new(Mutex::new(Snapshot::default()));
        let shared = Arc::clone(&snapshot);
        thread::Builder::new()
            .name("mpv".into())
            .spawn(move || match init(video_output) {
                Ok(mpv) => {
                    let _ = ready_tx.send(Ok(()));
                    run(mpv, requests, shared);
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            })
            .map_err(|e| PlaybackError::Backend(e.to_string()))?;
        ready_rx
            .recv()
            .map_err(|_| PlaybackError::Backend("mpv did not start".into()))?
            .map_err(PlaybackError::Backend)?;
        Ok(Self { commands, snapshot })
    }

    fn snapshot(&self) -> Snapshot {
        self.snapshot.lock().map(|s| s.clone()).unwrap_or_default()
    }

    fn send(&self, command: Command) -> Result<()> {
        self.commands
            .send(command)
            .map_err(|_| PlaybackError::Backend("mpv has stopped".into()))
    }
}

/// A cloneable handle to a player, for the video window's controls. Its methods only send
/// commands and read the snapshot, as the player's own methods do.
#[derive(Clone)]
pub struct MpvHandle {
    commands: Sender<Command>,
    snapshot: Arc<Mutex<Snapshot>>,
}

/// What the controls show: the state, and the position and length in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Status {
    pub state: PlayerState,
    pub position: Option<f64>,
    pub duration: Option<f64>,
}

impl MpvHandle {
    pub fn play(&self) {
        let _ = self.commands.send(Command::Play);
    }

    pub fn pause(&self) {
        let _ = self.commands.send(Command::Pause);
    }

    pub fn stop(&self) {
        let _ = self.commands.send(Command::Stop);
    }

    pub fn seek(&self, secs: f64) {
        let _ = self.commands.send(Command::Seek(secs.max(0.0)));
    }

    /// Output level from 0 to 1.
    pub fn set_volume(&self, volume: f32) {
        let percent = (f64::from(volume.max(0.0)) * 100.0).min(MAX_VOLUME);
        let _ = self.commands.send(Command::Volume(percent));
    }

    pub fn status(&self) -> Status {
        let s = self.snapshot.lock().map(|s| s.clone()).unwrap_or_default();
        Status {
            state: state_of(&s),
            position: s.position,
            duration: s.duration,
        }
    }
}

/// The player state a snapshot describes.
fn state_of(s: &Snapshot) -> PlayerState {
    if !s.loaded {
        PlayerState::Idle
    } else if s.failure.is_some() || s.ended {
        PlayerState::Ended
    } else if s.paused {
        PlayerState::Paused
    } else {
        PlayerState::Playing
    }
}

/// Create the mpv instance, with the options the player depends on.
fn init(video_output: &str) -> std::result::Result<Mpv, String> {
    Mpv::with_initializer(|init| {
        // The control bar is ours, so mpv's own on-screen controls stay off.
        init.set_option("osc", "no")?;
        // Keep the last frame when a file ends, so the player reports `Ended` rather than idling.
        init.set_option("keep-open", "yes")?;
        // Stay running with nothing loaded, so the next file opens in the same window.
        init.set_option("idle", "yes")?;
        // Without a video output, mpv reports video-only files as having nothing to play.
        // `gpu` is the output the mpv command line uses by default.
        init.set_option("vo", video_output)?;
        // Library files are local. Never hand a URL to yt-dlp.
        init.set_option("ytdl", "no")?;
        init.set_option("terminal", "no")?;
        Ok(())
    })
    .map_err(|e| e.to_string())
}

/// The worker loop: apply each command, then refresh the snapshot from mpv.
fn run(mpv: Mpv, requests: Receiver<Command>, shared: Arc<Mutex<Snapshot>>) {
    // Closing the video window stops the file, rather than leaving it paused in a window.
    let _ = mpv.command("keybind", &["CLOSE_WIN", "stop"]);
    loop {
        match requests.recv_timeout(POLL) {
            Ok(command) => apply(&mpv, command, &shared),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        refresh(&mpv, &shared);
    }
}

fn apply(mpv: &Mpv, command: Command, shared: &Mutex<Snapshot>) {
    let update = |f: &dyn Fn(&mut Snapshot)| {
        if let Ok(mut s) = shared.lock() {
            f(&mut s);
        }
    };
    match command {
        Command::Load(path) => {
            // Forget any events left from the previous file, then open the new one paused.
            drain(mpv, shared);
            update(&|s| {
                s.loaded = true;
                s.paused = true;
                s.ended = false;
                s.opened = false;
                s.position = None;
                s.duration = None;
                s.failure = None;
            });
            let opened = mpv
                .set_property("pause", true)
                .and_then(|_| mpv.command("loadfile", &[path.as_str(), "replace"]));
            if let Err(e) = opened {
                update(&|s| {
                    s.loaded = false;
                    s.failure = Some(format!("could not open {path} ({e})"));
                });
            }
        }
        Command::Play => {
            let at_end = shared.lock().map(|s| s.ended).unwrap_or(false);
            if at_end {
                let _ = mpv.command("seek", &["0", "absolute"]);
            }
            let _ = mpv.set_property("pause", false);
            update(&|s| s.paused = false);
        }
        Command::Pause => {
            let _ = mpv.set_property("pause", true);
            update(&|s| s.paused = true);
        }
        Command::Stop => {
            let _ = mpv.command("stop", &[]);
            update(&|s| {
                s.loaded = false;
                s.ended = false;
                s.position = None;
                s.duration = None;
                s.failure = None;
            });
        }
        Command::Seek(secs) => {
            let secs = format!("{secs:.3}");
            let _ = mpv.command("seek", &[secs.as_str(), "absolute"]);
        }
        Command::Volume(percent) => {
            let _ = mpv.set_property("volume", percent);
        }
        Command::Speed(speed) => {
            let _ = mpv.set_property("speed", speed);
        }
    }
}

/// Read mpv's events and properties into the snapshot.
fn refresh(mpv: &Mpv, shared: &Mutex<Snapshot>) {
    drain(mpv, shared);
    let loaded = shared.lock().map(|s| s.loaded).unwrap_or(false);
    if !loaded {
        if let Ok(mut s) = shared.lock() {
            s.position = None;
            s.duration = None;
        }
        return;
    }
    // Read everything from mpv before taking the lock. mpv can wait on the main thread, and the
    // main thread reads this snapshot, so holding the lock across an mpv call can deadlock.
    let idle = mpv.get_property::<bool>("idle-active").unwrap_or(false);
    let position = mpv
        .get_property::<f64>("time-pos")
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.0);
    let duration = mpv
        .get_property::<f64>("duration")
        .ok()
        .filter(|v| v.is_finite() && *v > 0.0);
    let paused = mpv.get_property::<bool>("pause").ok();
    let ended = mpv.get_property::<bool>("eof-reached").unwrap_or(false);

    let Ok(mut s) = shared.lock() else {
        return;
    };
    if !s.loaded {
        return;
    }
    // Once a file has started, mpv going idle means it was stopped from outside the app, such as
    // by closing its window. The file is no longer loaded. A file that failed to open also leaves
    // mpv idle, but its failure is shown, not the close.
    if idle && s.opened && s.failure.is_none() {
        s.loaded = false;
        s.opened = false;
        s.paused = false;
        s.ended = false;
        s.position = None;
        s.duration = None;
        return;
    }
    if !idle {
        s.opened = true;
    }
    s.position = position;
    s.duration = duration;
    if let Some(paused) = paused {
        s.paused = paused;
    }
    s.ended = ended;
}

/// Take the events mpv has queued. An error event means the file could not play.
fn drain(mpv: &Mpv, shared: &Mutex<Snapshot>) {
    while let Some(event) = mpv.wait_event(0.0) {
        if let Err(e) = event
            && let Ok(mut s) = shared.lock()
        {
            s.failure.get_or_insert_with(|| e.to_string());
        }
    }
}

impl Player for MpvPlayer {
    fn load(&mut self, path: &Path) -> Result<()> {
        let name = path
            .to_str()
            .ok_or_else(|| PlaybackError::Open(path.display().to_string()))?;
        // Show the new file as loaded and paused straight away. The worker confirms it.
        if let Ok(mut s) = self.snapshot.lock() {
            *s = Snapshot {
                loaded: true,
                paused: true,
                ..Snapshot::default()
            };
        }
        self.send(Command::Load(name.to_owned()))
    }

    fn play(&mut self) -> Result<()> {
        if !self.snapshot().loaded {
            return Err(PlaybackError::NoMedia);
        }
        self.send(Command::Play)
    }

    fn pause(&mut self) -> Result<()> {
        if !self.snapshot().loaded {
            return Err(PlaybackError::NoMedia);
        }
        self.send(Command::Pause)
    }

    fn stop(&mut self) -> Result<()> {
        self.send(Command::Stop)?;
        if let Ok(mut s) = self.snapshot.lock() {
            *s = Snapshot::default();
        }
        Ok(())
    }

    fn seek(&mut self, position: Duration) -> Result<()> {
        if !self.snapshot().loaded {
            return Err(PlaybackError::NoMedia);
        }
        self.send(Command::Seek(position.as_secs_f64()))
    }

    fn position(&self) -> Option<Duration> {
        let s = self.snapshot();
        if !s.loaded {
            return None;
        }
        s.position.map(Duration::from_secs_f64)
    }

    fn duration(&self) -> Option<Duration> {
        let s = self.snapshot();
        if !s.loaded {
            return None;
        }
        s.duration.map(Duration::from_secs_f64)
    }

    fn state(&self) -> PlayerState {
        state_of(&self.snapshot())
    }

    fn set_volume(&mut self, volume: f32) -> Result<()> {
        let percent = (f64::from(volume.max(0.0)) * 100.0).min(MAX_VOLUME);
        self.send(Command::Volume(percent))
    }

    fn set_speed(&mut self, speed: f32) -> Result<()> {
        if speed <= 0.0 {
            return Ok(());
        }
        // mpv keeps the pitch when the speed changes, unless `audio-pitch-correction` is off.
        self.send(Command::Speed(f64::from(speed)))
    }

    fn last_error(&self) -> Option<String> {
        self.snapshot().failure
    }
}
