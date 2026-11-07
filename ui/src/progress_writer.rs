//! Saves how far the current file has been played. A thread of its own owns a database connection
//! and writes what it is sent, so the window never waits for the database. The window sends a
//! position about once a second, so a killed app loses at most that much progress.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::thread;

/// One position to save: a file, how far it was played, and its length if known.
pub struct Save {
    pub path: PathBuf,
    pub position_secs: f64,
    pub duration_secs: Option<f64>,
}

/// The handle the window uses to save progress.
pub struct ProgressWriter {
    sender: Sender<Save>,
}

impl ProgressWriter {
    /// Start the writing thread for the library at `db`. `None` when the thread cannot start.
    pub fn start(db: &Path) -> Option<Self> {
        let (sender, receiver) = mpsc::channel::<Save>();
        let db = db.to_path_buf();
        thread::Builder::new()
            .name("progress".into())
            .spawn(move || {
                let Ok(library) = player_core::Library::open(&db) else {
                    eprintln!("player: could not open the library to save progress");
                    return;
                };
                // Ends when the window, and so the sender, goes away.
                for save in receiver {
                    if let Err(e) =
                        library.save_progress(&save.path, save.position_secs, save.duration_secs)
                    {
                        eprintln!("player: could not save progress: {e}");
                    }
                }
            })
            .ok()?;
        Some(Self { sender })
    }

    /// Queue a save. The thread writes it in order with the others.
    pub fn save(&self, save: Save) {
        let _ = self.sender.send(save);
    }
}
