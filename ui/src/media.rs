//! Media keys and the system's now-playing display: macOS Now Playing, MPRIS on Linux, and
//! System Media Transport Controls on Windows.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use eframe::egui;
#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::runtime::ProtocolObject;
#[cfg(target_os = "macos")]
use objc2_foundation::{NSActivityOptions, NSObjectProtocol, NSProcessInfo, NSString};
use quokka_playback::PlayerState;
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
    SeekDirection,
};

use quokka_core::MediaItem;

use crate::format;
use crate::playback::Playback;

/// How far the system's seek buttons move, in seconds.
const SEEK_STEP: f64 = 10.0;

/// The system's view of playback is refreshed this often while a track plays, so the elapsed
/// time shown by the system stays close to the real position.
const REFRESH: Duration = Duration::from_secs(5);

pub struct Media {
    controls: MediaControls,
    /// The track and state last published. The system is told again only when one changes.
    shown: Option<(Option<i64>, PlayerState)>,
    sent_at: Option<Instant>,
}

impl Media {
    /// Connect to the system's media controls. Returns `None` when they can't be set up, so the
    /// app runs without media keys.
    pub fn new(ctx: &egui::Context) -> Option<Self> {
        // Windows needs the window handle, and the app does not pass it in yet. Without it the
        // media keys stay off there, rather than the setup panicking.
        if cfg!(windows) {
            return None;
        }
        let config = PlatformConfig {
            display_name: "Quokka",
            dbus_name: "quokka",
            hwnd: None,
        };
        let mut controls = MediaControls::new(config).ok()?;
        let (tx, events) = mpsc::channel();
        DRAIN.with(|drain| drain.borrow_mut().events = Some(events));
        let ctx = ctx.clone();
        controls
            .attach(move |event| {
                let _ = tx.send(event);
                post_drain();
                ctx.request_repaint();
            })
            .ok()?;
        Some(Self {
            controls,
            shown: None,
            sent_at: None,
        })
    }

    /// Tell the system what is playing, when the track or the state changes, and now and then
    /// while a track plays.
    pub fn publish(&mut self, playback: &Playback) {
        let state = playback.state();
        let id = playback.current_id();
        let due =
            state == PlayerState::Playing && self.sent_at.is_none_or(|t| t.elapsed() >= REFRESH);
        if self.shown == Some((id, state)) && !due {
            return;
        }

        if self.shown.map(|(shown, _)| shown) != Some(id)
            && let Some(item) = playback.current_item()
        {
            let title = format::title(&item);
            let artist = item.artist.as_deref().or(item.album_artist.as_deref());
            let duration = item
                .duration_ms
                .and_then(|ms| u64::try_from(ms).ok())
                .map(Duration::from_millis);
            let cover = cover_file(&item);
            let _ = self.controls.set_metadata(MediaMetadata {
                title: Some(&title),
                album: item.album.as_deref(),
                artist,
                cover_url: cover.as_deref(),
                duration,
            });
        }

        let progress = playback.position().map(MediaPosition);
        let media = match state {
            PlayerState::Playing => MediaPlayback::Playing { progress },
            PlayerState::Paused => MediaPlayback::Paused { progress },
            PlayerState::Idle | PlayerState::Ended => MediaPlayback::Stopped,
        };
        let _ = self.controls.set_playback(media);
        self.shown = Some((id, state));
        self.sent_at = Some(Instant::now());
    }
}

/// Apply one key press or system request to the playback.
fn apply(event: MediaControlEvent, playback: &mut Playback) {
    match event {
        MediaControlEvent::Play => playback.play(),
        MediaControlEvent::Pause => playback.pause(),
        MediaControlEvent::Toggle => playback.toggle(),
        MediaControlEvent::Next => playback.next(),
        MediaControlEvent::Previous => playback.previous(),
        MediaControlEvent::Stop => playback.stop(),
        MediaControlEvent::Seek(SeekDirection::Forward) => {
            playback.seek_relative(SEEK_STEP);
        }
        MediaControlEvent::Seek(SeekDirection::Backward) => {
            playback.seek_relative(-SEEK_STEP);
        }
        MediaControlEvent::SeekBy(direction, by) => {
            let secs = by.as_secs_f64();
            match direction {
                SeekDirection::Forward => playback.seek_relative(secs),
                SeekDirection::Backward => playback.seek_relative(-secs),
            }
        }
        MediaControlEvent::SetPosition(MediaPosition(at)) => {
            playback.seek(at.as_secs_f64());
        }
        MediaControlEvent::SetVolume(level) => playback.set_volume(level as f32),
        _ => {}
    }
}

/// The state the event handler needs, kept on the main thread. The players stay there, so the
/// playback is reached only from the main thread.
#[derive(Default)]
struct Drain {
    playback: Option<Weak<RefCell<Playback>>>,
    events: Option<Receiver<MediaControlEvent>>,
}

thread_local! {
    static DRAIN: RefCell<Drain> = RefCell::new(Drain::default());
}

/// Let the media event handler reach the app's playback. The app calls this once it has made the
/// playback shared.
pub fn register(playback: &Rc<RefCell<Playback>>) {
    DRAIN.with(|drain| drain.borrow_mut().playback = Some(Rc::downgrade(playback)));
}

/// Apply the key presses and system requests received so far. Runs on the main thread, both from
/// the app's frames and from the main dispatch queue, so it works while the window is not drawn.
/// Call it only when the app is not holding the playback.
pub fn drain_events() {
    let Some(playback) =
        DRAIN.with(|drain| drain.borrow().playback.as_ref().and_then(Weak::upgrade))
    else {
        return;
    };
    let Ok(mut playback) = playback.try_borrow_mut() else {
        // The app is using the playback right now. Try again once it has let go.
        post_drain();
        return;
    };
    DRAIN.with(|drain| {
        let drain = drain.borrow();
        if let Some(events) = drain.events.as_ref() {
            while let Ok(event) = events.try_recv() {
                apply(event, &mut playback);
            }
        }
    });
}

/// Schedule `drain_events` on the main dispatch queue. That queue is served by the app's run loop,
/// so it runs even when the window is covered and eframe is not drawing it.
#[cfg(target_os = "macos")]
fn post_drain() {
    unsafe extern "C" {
        static _dispatch_main_q: std::ffi::c_void;
        fn dispatch_async_f(
            queue: *const std::ffi::c_void,
            context: *mut std::ffi::c_void,
            work: extern "C" fn(*mut std::ffi::c_void),
        );
    }
    extern "C" fn run(_context: *mut std::ffi::c_void) {
        drain_events();
    }
    // SAFETY: the main queue is libdispatch's own static, and `run` ignores its context.
    unsafe {
        dispatch_async_f(
            std::ptr::addr_of!(_dispatch_main_q),
            std::ptr::null_mut(),
            run,
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn post_drain() {}

/// Write the track's embedded cover to a temporary file, and return its `file://` URL. The
/// system reads the cover from there. `None` when the track has no cover or it can't be written.
fn cover_file(item: &MediaItem) -> Option<String> {
    let bytes = quokka_core::read_cover(&item.path)?;
    let path = std::env::temp_dir().join(format!("quokka-cover-{}", item.id));
    std::fs::write(&path, bytes).ok()?;
    Some(format!("file://{}", path.display()))
}

/// Keeps the system from suspending the app while media plays. macOS naps a background app that is
/// not doing user-initiated work, and a napped app handles the media keys and the now-playing
/// controls late, or not until it is brought forward. Holding an activity while playing avoids that.
#[derive(Default)]
pub struct Activity {
    #[cfg(target_os = "macos")]
    token: Option<Retained<ProtocolObject<dyn NSObjectProtocol>>>,
}

impl Activity {
    /// Hold the activity while `playing` is true, and release it when playback stops.
    pub fn set(&mut self, playing: bool) {
        #[cfg(target_os = "macos")]
        {
            match (playing, self.token.is_some()) {
                (true, false) => {
                    let reason = NSString::from_str("Playing media");
                    let info = NSProcessInfo::processInfo();
                    self.token = Some(info.beginActivityWithOptions_reason(
                        NSActivityOptions::UserInitiatedAllowingIdleSystemSleep,
                        &reason,
                    ));
                }
                (false, true) => {
                    if let Some(token) = self.token.take() {
                        // SAFETY: the token came from `beginActivityWithOptions_reason` and is released once.
                        unsafe { NSProcessInfo::processInfo().endActivity(&token) };
                    }
                }
                _ => {}
            }
        }
        #[cfg(not(target_os = "macos"))]
        let _ = playing;
    }
}
