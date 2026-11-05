use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;

/// Light, dark, or follow the operating system.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

/// How tracks are levelled with their ReplayGain tags. Off plays every track as it was mastered.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplayGainMode {
    #[default]
    Off,
    /// Each track gets its own gain.
    Track,
    /// Tracks keep the album's relative loudness, using the album gain.
    Album,
}

/// User settings, stored as JSON.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// Folders the library scans, in the order they were added.
    #[serde(default)]
    pub library_roots: Vec<PathBuf>,
    /// The single folder that earlier versions saved. It is read once and moved into
    /// `library_roots` on load, and never written back.
    #[serde(default, rename = "library_root", skip_serializing)]
    pub legacy_root: Option<PathBuf>,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub replay_gain: ReplayGainMode,
}

impl Settings {
    /// Load settings from `path`. A missing file yields the defaults.
    pub fn load(path: &Path) -> Result<Self> {
        let mut settings: Self = match fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e.into()),
        };
        if let Some(root) = settings.legacy_root.take()
            && !settings.library_roots.contains(&root)
        {
            settings.library_roots.insert(0, root);
        }
        Ok(settings)
    }

    /// Save settings to `path`, creating parent folders. Writes via a temp file and rename.
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
