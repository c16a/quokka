use eframe::egui;
use player_core::{AlbumSummary, Library, MediaItem};

use crate::format;

/// Left: album list (virtualised). Right: header with composers, then the tracks.
#[derive(Default)]
pub struct AlbumsView {
    albums: Vec<AlbumSummary>,
    selected: Option<usize>,
    tracks: Vec<MediaItem>,
}

impl AlbumsView {
    pub fn reload(&mut self, library: &Library) {
        self.albums = library.albums().unwrap_or_else(|e| {
            eprintln!("player: could not load albums: {e}");
            Vec::new()
        });
        if self.selected.is_some_and(|i| i >= self.albums.len()) {
            self.selected = None;
        }
        self.load_tracks(library);
    }

    fn load_tracks(&mut self, library: &Library) {
        self.tracks = match self.selected.and_then(|i| self.albums.get(i)) {
            Some(album) => library
                .album_tracks(album.album.as_deref())
                .unwrap_or_else(|e| {
                    eprintln!("player: could not load tracks: {e}");
                    Vec::new()
                }),
            None => Vec::new(),
        };
    }

    pub fn show(&mut self, ui: &mut egui::Ui, library: &Library) {
        if self.albums.is_empty() {
            ui.label("No albums yet. Choose your library folder in Settings.");
            return;
        }

        let mut clicked = None;
        egui::Panel::left("album_list")
            .resizable(true)
            .default_size(340.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show_rows(ui, 44.0, self.albums.len(), |ui, range| {
                        for i in range {
                            let album = &self.albums[i];
                            let title = album.album.as_deref().unwrap_or("Unknown album");
                            let artists = join_or(&album.artists, "Unknown artist");
                            let year = album.year.map(|y| format!(" · {y}")).unwrap_or_default();
                            let text =
                                format!("{title}\n{artists}{year} · {} tracks", album.track_count);
                            if ui
                                .selectable_label(self.selected == Some(i), text)
                                .clicked()
                            {
                                clicked = Some(i);
                            }
                        }
                    });
            });

        if let Some(i) = clicked {
            self.selected = Some(i);
            self.load_tracks(library);
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let Some(i) = self.selected else {
                ui.weak("Select an album.");
                return;
            };
            let album = &self.albums[i];
            ui.heading(album.album.as_deref().unwrap_or("Unknown album"));
            if !album.composers.is_empty() {
                ui.label(album.composers.join(", "));
            }
            ui.separator();
            // Solid scroll bars take their own width, so they don't cover the track durations.
            ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, 44.0, self.tracks.len(), |ui, range| {
                    for n in range {
                        let track = &self.tracks[n];
                        ui.horizontal(|ui| {
                            let number = track.track.map(|t| t.to_string()).unwrap_or_default();
                            ui.add_sized([32.0, 40.0], egui::Label::new(number));
                            ui.vertical(|ui| {
                                ui.label(format::title(track));
                                if let Some(artist) =
                                    track.artist.as_deref().or(track.album_artist.as_deref())
                                {
                                    ui.weak(artist);
                                }
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    // Gap between the durations and the scroll bar.
                                    ui.add_space(16.0);
                                    ui.weak(format::duration(track.duration_ms));
                                },
                            );
                        });
                    }
                });
        });
    }
}

/// Join names with ", ", or return `fallback` when there are none.
fn join_or(names: &[String], fallback: &str) -> String {
    if names.is_empty() {
        fallback.to_owned()
    } else {
        names.join(", ")
    }
}
