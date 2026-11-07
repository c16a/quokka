use std::collections::BTreeMap;
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

/// The kind of media a library folder holds. Each kind has its own scan rules and tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibraryKind {
    Music,
    Movies,
}

/// User settings, stored as JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Folders the library scans, in the order they were added.
    #[serde(default)]
    pub library_roots: Vec<PathBuf>,
    /// The single folder that earlier versions saved. It is read once and moved into
    /// `library_roots` on load, and never written back.
    #[serde(default, rename = "library_root", skip_serializing)]
    pub legacy_root: Option<PathBuf>,
    /// The kind of each folder in `library_roots`, keyed by the folder's path. A folder with no entry
    /// has not been given a kind yet.
    #[serde(default)]
    pub library_kinds: BTreeMap<String, LibraryKind>,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub replay_gain: ReplayGainMode,
    /// Whether to look up lyrics on lrclib.net for tracks with no `.lrc` file. Off until the user turns it on.
    #[serde(default)]
    pub lyrics_online: bool,
    /// Seconds over which each track fades into the next. Zero turns crossfade off.
    #[serde(default)]
    pub crossfade_secs: f32,
    /// Playback speed, where 1.0 is normal. Speeds other than 1.0 also change the pitch.
    #[serde(default = "default_speed")]
    pub playback_speed: f32,
    /// The user's own TMDb API key, for movie descriptions and cast. Not shipped with the app.
    /// Left out of the file when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tmdb_api_key: Option<String>,
}

fn default_speed() -> f32 {
    1.0
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            library_roots: Vec::new(),
            legacy_root: None,
            library_kinds: BTreeMap::new(),
            theme: Theme::default(),
            replay_gain: ReplayGainMode::default(),
            lyrics_online: false,
            crossfade_secs: 0.0,
            playback_speed: 1.0,
            tmdb_api_key: None,
        }
    }
}

impl Settings {
    /// The kind of the library folder at `root`, or `None` when it has not been set yet.
    pub fn library_kind(&self, root: &Path) -> Option<LibraryKind> {
        self.library_kinds.get(&kind_key(root)).copied()
    }

    /// Give the library folder at `root` a kind, replacing any earlier one.
    pub fn set_library_kind(&mut self, root: &Path, kind: LibraryKind) {
        self.library_kinds.insert(kind_key(root), kind);
    }

    /// Forget the kind of the library folder at `root`, once it has been removed from the library.
    pub fn forget_library_kind(&mut self, root: &Path) {
        self.library_kinds.remove(&kind_key(root));
    }

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

    /// Save settings to `path`, creating parent folders. Writes via a temp file and rename. The
    /// file is readable only by its owner, since it may hold an API key.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))?;
        }
        fs::rename(&tmp, path)?;
        Ok(())
    }
}

/// The key a folder's kind is stored under.
fn kind_key(root: &Path) -> String {
    root.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_without_kinds_loads_with_none_set() {
        let text = r#"{ "library_roots": ["/music", "/films"] }"#;
        let settings: Settings = serde_json::from_str(text).unwrap();
        assert_eq!(settings.library_roots.len(), 2);
        assert_eq!(settings.library_kind(Path::new("/music")), None);
    }

    #[test]
    fn kinds_survive_a_save_and_load() {
        let dir = std::env::temp_dir().join(format!("player-kinds-{}", std::process::id()));
        let path = dir.join("settings.json");
        let mut settings = Settings::default();
        settings.library_roots.push(PathBuf::from("/films"));
        settings.set_library_kind(Path::new("/films"), LibraryKind::Movies);
        settings.save(&path).unwrap();
        let loaded = Settings::load(&path).unwrap();
        assert_eq!(
            loaded.library_kind(Path::new("/films")),
            Some(LibraryKind::Movies)
        );
        fs::remove_dir_all(&dir).ok();
    }
}
