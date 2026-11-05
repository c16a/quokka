//! Playlists: the list of playlists, and one playlist's tracks with play, shuffle and remove.

use eframe::egui;
use player_core::{Library, MediaItem, PlaylistSummary};

use crate::albums::{
    PlayRequest, back_icon_button, play_icon_button, shuffle_icon_button, shuffled,
};
use crate::format;

/// The playlist list, or the tracks of one playlist when one is open.
#[derive(Default)]
pub struct PlaylistsView {
    playlists: Vec<PlaylistSummary>,
    /// The open playlist, if any.
    selected: Option<i64>,
    tracks: Vec<MediaItem>,
    /// Stored position of each track in the open playlist, for removing it.
    positions: Vec<i64>,
    new_name: String,
}

impl PlaylistsView {
    /// All playlists, for the menus that add tracks to one.
    pub fn summaries(&self) -> &[PlaylistSummary] {
        &self.playlists
    }

    /// Reload the playlists from the library, and the open one's tracks.
    pub fn reload(&mut self, library: &Library) {
        self.playlists = library.playlists().unwrap_or_else(|e| {
            eprintln!("player: could not load playlists: {e}");
            Vec::new()
        });
        if self
            .selected
            .is_some_and(|id| !self.playlists.iter().any(|p| p.id == id))
        {
            self.selected = None;
        }
        self.load_tracks(library);
    }

    fn load_tracks(&mut self, library: &Library) {
        match self.selected {
            Some(id) => {
                self.tracks = library.playlist_tracks(id).unwrap_or_default();
                self.positions = library.playlist_positions(id).unwrap_or_default();
            }
            None => {
                self.tracks.clear();
                self.positions.clear();
            }
        }
    }

    /// Draw the view. Returns a play request when the user asks to play the open playlist.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        library: &Library,
        current: Option<i64>,
    ) -> Option<PlayRequest> {
        match self.selected {
            None => {
                self.list(ui, library);
                None
            }
            Some(_) => self.detail(ui, library, current),
        }
    }

    /// The list of playlists, with a field to create one.
    fn list(&mut self, ui: &mut egui::Ui, library: &Library) {
        ui.heading("Playlists");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let field = ui.add(
                egui::TextEdit::singleline(&mut self.new_name)
                    .hint_text("New playlist name")
                    .desired_width(240.0),
            );
            let create = ui.add_enabled(
                !self.new_name.trim().is_empty(),
                egui::Button::new("Create"),
            );
            let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if (create.clicked() || enter) && !self.new_name.trim().is_empty() {
                match library.create_playlist(&self.new_name) {
                    Ok(id) => {
                        self.new_name.clear();
                        self.selected = Some(id);
                        self.reload(library);
                    }
                    Err(e) => eprintln!("player: could not create playlist: {e}"),
                }
            }
        });
        ui.add_space(12.0);

        if self.playlists.is_empty() {
            ui.weak("No playlists yet. Add tracks from an album or the search results.");
            return;
        }
        let mut delete = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for playlist in &self.playlists {
                    let label = format!("{}   ·   {} tracks", playlist.name, playlist.track_count);
                    let response = ui.add_sized(
                        [ui.available_width(), 36.0],
                        egui::Button::new(label).corner_radius(6.0),
                    );
                    if response.clicked() {
                        self.selected = Some(playlist.id);
                    }
                    response.context_menu(|ui| {
                        if ui.button("Delete playlist").clicked() {
                            delete = Some(playlist.id);
                            ui.close();
                        }
                    });
                }
            });
        if let Some(id) = delete {
            if let Err(e) = library.delete_playlist(id) {
                eprintln!("player: could not delete playlist: {e}");
            }
            self.reload(library);
        }
        if self.selected.is_some() {
            self.load_tracks(library);
        }
    }

    /// One playlist: back button, name, play and shuffle, then its tracks.
    fn detail(
        &mut self,
        ui: &mut egui::Ui,
        library: &Library,
        current: Option<i64>,
    ) -> Option<PlayRequest> {
        let mut play = None;
        let mut back = false;
        let mut remove = None;
        let name = self
            .playlists
            .iter()
            .find(|p| Some(p.id) == self.selected)
            .map(|p| p.name.clone())
            .unwrap_or_default();

        if back_icon_button(ui)
            .on_hover_text("All playlists")
            .clicked()
        {
            back = true;
        }
        ui.add_space(12.0);
        ui.heading(name);
        ui.weak(format!("{} tracks", self.tracks.len()));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if play_icon_button(ui)
                .on_hover_text("Play playlist")
                .clicked()
            {
                play = Some(PlayRequest {
                    tracks: self.tracks.clone(),
                    index: 0,
                });
            }
            ui.add_space(8.0);
            if shuffle_icon_button(ui)
                .on_hover_text("Shuffle playlist")
                .clicked()
            {
                play = Some(PlayRequest {
                    tracks: shuffled(self.tracks.clone()),
                    index: 0,
                });
            }
        });
        ui.add_space(12.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (n, track) in self.tracks.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            let title = format::title(track);
                            if current == Some(track.id) {
                                ui.strong(title);
                            } else {
                                ui.label(title);
                            }
                            if let Some(artist) =
                                track.artist.as_deref().or(track.album_artist.as_deref())
                            {
                                ui.weak(artist);
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Remove").clicked() {
                                remove = Some(n);
                            }
                            ui.add_space(8.0);
                            ui.weak(format::duration(track.duration_ms));
                        });
                    });
                    ui.add_space(4.0);
                }
            });

        if back {
            self.selected = None;
            self.tracks.clear();
            self.positions.clear();
        }
        if let (Some(n), Some(id)) = (remove, self.selected)
            && let Some(&position) = self.positions.get(n)
        {
            if let Err(e) = library.remove_from_playlist(id, position) {
                eprintln!("player: could not remove track: {e}");
            }
            self.load_tracks(library);
        }
        play
    }
}
