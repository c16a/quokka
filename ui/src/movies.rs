use eframe::egui;
use player_core::{Library, MediaItem};

use crate::format;

/// Video files, one per row, sorted by title.
#[derive(Default)]
pub struct MoviesView {
    movies: Vec<MediaItem>,
}

impl MoviesView {
    pub fn reload(&mut self, library: &Library) {
        self.movies = library.movies().unwrap_or_else(|e| {
            eprintln!("player: could not load movies: {e}");
            Vec::new()
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        if self.movies.is_empty() {
            ui.label("No movies yet. Choose your library folder in Settings.");
            return;
        }
        ui.label(format!("{} movies", self.movies.len()));
        ui.separator();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, 36.0, self.movies.len(), |ui, range| {
                for i in range {
                    let movie = &self.movies[i];
                    ui.horizontal(|ui| {
                        ui.label(crate::fonts::bold(format::title(movie)));
                        ui.weak(format!(
                            "{} · {}",
                            format::duration(movie.duration_ms),
                            format::size(movie.size)
                        ));
                    })
                    .response
                    .on_hover_text(movie.path.display().to_string());
                }
            });
    }
}
