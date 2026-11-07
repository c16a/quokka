//! How far each file was played, so that playing it again can resume from there. Keyed by the
//! file's path, and written by the player as it plays, so the last saved position survives the app
//! being killed.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{OptionalExtension, params};

use crate::{Library, Result};

/// A file's saved position.
#[derive(Debug, Clone, PartialEq)]
pub struct Progress {
    pub path: PathBuf,
    /// How far the file was played, in seconds.
    pub position_secs: f64,
    /// The file's length in seconds, when the player knew it.
    pub duration_secs: Option<f64>,
    /// When the position was saved, in seconds since the Unix epoch.
    pub updated: i64,
}

/// How close to the end a position may be and still count as the end. Past this, a file is
/// finished, and starting it again begins from the start.
pub const FINISHED_WITHIN_SECS: f64 = 60.0;

/// Whether a position is close enough to the end of a file of `duration_secs` to count as finished.
pub fn finished(position_secs: f64, duration_secs: Option<f64>) -> bool {
    duration_secs.is_some_and(|duration| position_secs >= duration - FINISHED_WITHIN_SECS)
}

impl Library {
    /// Save how far the file at `path` was played.
    pub fn save_progress(
        &self,
        path: &Path,
        position_secs: f64,
        duration_secs: Option<f64>,
    ) -> Result<()> {
        let updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs() as i64);
        self.conn.execute(
            "INSERT INTO progress(path, position_secs, duration_secs, updated) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(path) DO UPDATE SET position_secs = excluded.position_secs, \
             duration_secs = excluded.duration_secs, updated = excluded.updated",
            params![
                path.to_string_lossy(),
                position_secs,
                duration_secs,
                updated
            ],
        )?;
        Ok(())
    }

    /// The saved position of the file at `path`, if there is one.
    pub fn progress(&self, path: &Path) -> Result<Option<Progress>> {
        let row = self
            .conn
            .query_row(
                "SELECT position_secs, duration_secs, updated FROM progress WHERE path = ?1",
                [path.to_string_lossy()],
                |row| {
                    Ok((
                        row.get::<_, f64>(0)?,
                        row.get::<_, Option<f64>>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()?;
        Ok(row.map(|(position_secs, duration_secs, updated)| Progress {
            path: path.to_path_buf(),
            position_secs,
            duration_secs,
            updated,
        }))
    }

    /// Forget the saved position of the file at `path`, so it plays from the start.
    pub fn clear_progress(&self, path: &Path) -> Result<()> {
        self.conn.execute(
            "DELETE FROM progress WHERE path = ?1",
            [path.to_string_lossy()],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_position_near_the_end_is_finished() {
        assert!(finished(7050.0, Some(7100.0)));
        assert!(!finished(7000.0, Some(7100.0)));
        assert!(!finished(16.0, Some(7100.0)));
        assert!(!finished(7000.0, None));
    }
}
