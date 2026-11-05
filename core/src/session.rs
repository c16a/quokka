//! What was playing when the app last closed, so it can pick up where it left off.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;

/// What repeat does when a track or the queue ends.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepeatMode {
    #[default]
    Off,
    /// Repeat the whole queue.
    Queue,
    /// Repeat the current track.
    Track,
}

impl RepeatMode {
    /// The next mode when the repeat button is pressed: off, queue, track, off.
    pub fn cycle(self) -> Self {
        match self {
            Self::Off => Self::Queue,
            Self::Queue => Self::Track,
            Self::Track => Self::Off,
        }
    }
}

/// The playback state saved on exit. Files are stored by path, so a file that has since been
/// removed from the library is skipped on restore.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// The queue, in play order.
    #[serde(default)]
    pub queue: Vec<PathBuf>,
    /// Position of the current track in `queue`.
    #[serde(default)]
    pub index: usize,
    /// Position within the current track, in seconds.
    #[serde(default)]
    pub position_secs: f64,
    #[serde(default)]
    pub shuffle: bool,
    #[serde(default)]
    pub repeat: RepeatMode,
}

impl Session {
    /// Load the saved session. A missing or unreadable file gives `None`, so the app starts
    /// fresh rather than failing.
    pub fn load(path: &Path) -> Option<Self> {
        let text = fs::read_to_string(path).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Save the session, creating parent folders. Writes via a temp file and rename.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        fs::rename(&tmp, path)?;
        Ok(())
    }
}
