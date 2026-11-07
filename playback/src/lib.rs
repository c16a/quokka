//! Playback abstraction. The UI and the library talk to playback only through
//! [`Player`]; concrete backends (rodio for audio, libmpv for video) implement it.

mod chain;
pub mod decode;
mod loudness;
mod mock;
pub mod movie_audio;
mod mpv_backend;
mod rodio_backend;
mod stream;
mod symphonia_source;
#[cfg(target_os = "macos")]
mod video_window;

use std::path::Path;
use std::time::Duration;

use thiserror::Error;

pub use loudness::{Loudness, measure_loudness};
pub use mock::MockPlayer;
pub use mpv_backend::MpvPlayer;
#[cfg(target_os = "macos")]
pub use video_window::VideoPlayer;
/// Elsewhere the video plays in mpv's own window, with its own controls.
#[cfg(not(target_os = "macos"))]
pub type VideoPlayer = MpvPlayer;
pub use rodio_backend::{RodioPlayer, output_device_names};
pub use stream::open_stream;
pub use symphonia_source::SymphoniaSource;

#[derive(Debug, Error)]
pub enum PlaybackError {
    #[error("could not open {0}")]
    Open(String),
    #[error("no media loaded")]
    NoMedia,
    #[error("playback backend error: {0}")]
    Backend(String),
}

pub type Result<T> = std::result::Result<T, PlaybackError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    /// Nothing loaded.
    Idle,
    /// Loaded and playing.
    Playing,
    /// Loaded and paused, including right after `load`.
    Paused,
    /// Playback reached the end of the media.
    Ended,
}

/// Backend-agnostic playback control. Keep backend types out of this trait.
pub trait Player {
    /// Load a file, replacing whatever was loaded. Leaves the player paused at zero.
    fn load(&mut self, path: &Path) -> Result<()>;
    fn play(&mut self) -> Result<()>;
    fn pause(&mut self) -> Result<()>;
    /// Unload and return to `Idle`.
    fn stop(&mut self) -> Result<()>;
    fn seek(&mut self, position: Duration) -> Result<()>;
    /// Current position, or `None` when nothing is loaded.
    fn position(&self) -> Option<Duration>;
    /// Total length, or `None` when unknown or nothing is loaded.
    fn duration(&self) -> Option<Duration>;
    fn state(&self) -> PlayerState;

    /// Set the output level, from 0 (silent) to 1 (full).
    fn set_volume(&mut self, _volume: f32) -> Result<()> {
        Ok(())
    }

    /// Play at `speed` times normal speed. Speeds other than 1 also change the pitch.
    fn set_speed(&mut self, _speed: f32) -> Result<()> {
        Ok(())
    }

    /// Fade each track into the next over `secs` seconds. Zero turns crossfade off.
    fn set_crossfade(&mut self, _secs: f32) -> Result<()> {
        Ok(())
    }

    /// Why the current track stopped before its end, if it did. Cleared when a track loads.
    fn last_error(&self) -> Option<String> {
        None
    }
}
