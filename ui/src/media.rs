//! Media keys and the system's now-playing display: macOS Now Playing, MPRIS on Linux, and
//! System Media Transport Controls on Windows.

use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use eframe::egui;
use player_playback::PlayerState;
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
    SeekDirection,
};

use player_core::MediaItem;

use crate::format;
use crate::playback::Playback;

/// How far the system's seek buttons move, in seconds.
const SEEK_STEP: f64 = 10.0;

/// The system's view of playback is refreshed this often while a track plays, so the elapsed
/// time shown by the system stays close to the real position.
const REFRESH: Duration = Duration::from_secs(5);

pub struct Media {
    controls: MediaControls,
    events: Receiver<MediaControlEvent>,
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
            dbus_name: "player",
            hwnd: None,
        };
        let mut controls = MediaControls::new(config).ok()?;
        let (tx, events) = mpsc::channel();
        let ctx = ctx.clone();
        controls
            .attach(move |event| {
                let _ = tx.send(event);
                ctx.request_repaint();
            })
            .ok()?;
        Some(Self {
            controls,
            events,
            shown: None,
            sent_at: None,
        })
    }

    /// Apply the key presses and system requests received since the last frame.
    pub fn handle_events(&mut self, playback: &mut Playback) {
        while let Ok(event) = self.events.try_recv() {
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

/// Write the track's embedded cover to a temporary file, and return its `file://` URL. The
/// system reads the cover from there. `None` when the track has no cover or it can't be written.
fn cover_file(item: &MediaItem) -> Option<String> {
    let bytes = player_core::read_cover(&item.path)?;
    let path = std::env::temp_dir().join(format!("player-cover-{}", item.id));
    std::fs::write(&path, bytes).ok()?;
    Some(format!("file://{}", path.display()))
}
