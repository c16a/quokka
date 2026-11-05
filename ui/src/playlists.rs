//! Playlists: the list of playlists, and one playlist's tracks with play, shuffle, rename,
//! reorder (drag the grip) and remove.

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
    /// Stored position of each track in the open playlist, for removing and reordering.
    positions: Vec<i64>,
    new_name: String,
    /// The name being typed, while the open playlist is being renamed.
    renaming: Option<String>,
    /// Index of the track whose grip is being dragged.
    dragging: Option<usize>,
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
            self.renaming = None;
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
        let mut rename = None;
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
                        if ui.button("Rename").clicked() {
                            rename = Some((playlist.id, playlist.name.clone()));
                            ui.close();
                        }
                        if ui.button("Delete playlist").clicked() {
                            delete = Some(playlist.id);
                            ui.close();
                        }
                    });
                }
            });
        if let Some((id, name)) = rename {
            self.selected = Some(id);
            self.renaming = Some(name);
            self.load_tracks(library);
        }
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

    /// One playlist: back button, name (with rename), play and shuffle, then its tracks.
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
        // Some(true) saves the new name, Some(false) cancels.
        let mut finish_rename = None;
        let mut start_rename = false;
        match &mut self.renaming {
            Some(buffer) => ui.horizontal(|ui| {
                let field = ui.add(egui::TextEdit::singleline(buffer).desired_width(260.0));
                if !field.has_focus() && !field.lost_focus() {
                    field.request_focus();
                }
                let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
                if ui.button("Save").clicked() || enter {
                    finish_rename = Some(true);
                } else if ui.button("Cancel").clicked() || escape {
                    finish_rename = Some(false);
                }
            }),
            None => ui.horizontal(|ui| {
                ui.heading(&name);
                if ui.small_button("Rename").clicked() {
                    start_rename = true;
                }
            }),
        };
        if start_rename {
            self.renaming = Some(name.clone());
        }
        if let Some(save) = finish_rename {
            let buffer = self.renaming.take().unwrap_or_default();
            if save
                && !buffer.trim().is_empty()
                && let Some(id) = self.selected
            {
                if let Err(e) = library.rename_playlist(id, &buffer) {
                    eprintln!("player: could not rename playlist: {e}");
                }
                self.reload(library);
            }
        }

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

        let mut rows: Vec<(usize, egui::Rect)> = Vec::with_capacity(self.tracks.len());
        let mut drag_started = None;
        let mut drag_stopped = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (n, track) in self.tracks.iter().enumerate() {
                    let row = ui
                        .horizontal(|ui| {
                            let (grip_rect, grip) =
                                ui.allocate_exact_size(egui::vec2(14.0, 36.0), egui::Sense::drag());
                            draw_grip(ui.painter(), grip_rect, ui.visuals().weak_text_color());
                            if grip.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                            }
                            if grip.drag_started() {
                                drag_started = Some(n);
                            }
                            if grip.drag_stopped() {
                                drag_stopped = Some(n);
                            }
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
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("Remove").clicked() {
                                        remove = Some(n);
                                    }
                                    ui.add_space(8.0);
                                    ui.weak(format::duration(track.duration_ms));
                                },
                            );
                        })
                        .response
                        .rect;
                    rows.push((n, row));
                    ui.add_space(4.0);
                }
            });

        if let Some(n) = drag_started {
            self.dragging = Some(n);
        }
        let pointer = ui.ctx().input(|i| i.pointer.interact_pos());
        let len = self.tracks.len();
        if let (Some(from), Some(_)) = (drag_stopped, self.dragging.take()) {
            // Dropped above the row under the pointer, or at the end of the list.
            if let Some(p) = pointer {
                let before = rows
                    .iter()
                    .find(|(_, r)| p.y < r.center().y)
                    .map_or(len, |(i, _)| *i);
                self.move_track(library, from, before);
            }
        } else if self.dragging.is_some()
            && let Some(p) = pointer
        {
            let before = rows
                .iter()
                .find(|(_, r)| p.y < r.center().y)
                .map_or(len, |(i, _)| *i);
            draw_drop_line(ui.painter(), &rows, before, ui.visuals().selection.bg_fill);
        }

        if back {
            self.selected = None;
            self.renaming = None;
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

    /// Move the track at `from` so it sits just before the track at `before` (or at the end when
    /// `before` is the number of tracks), and save the new order.
    fn move_track(&mut self, library: &Library, from: usize, before: usize) {
        if from >= self.tracks.len()
            || before == from
            || before == from + 1
            || before > self.tracks.len()
        {
            return;
        }
        let mut order: Vec<usize> = (0..self.tracks.len()).collect();
        let item = order.remove(from);
        let dest = if before > from { before - 1 } else { before };
        order.insert(dest, item);
        let stored: Vec<i64> = order.iter().map(|&n| self.positions[n]).collect();
        if let Some(id) = self.selected {
            if let Err(e) = library.reorder_playlist(id, &stored) {
                eprintln!("player: could not reorder playlist: {e}");
            }
            self.load_tracks(library);
        }
    }
}

/// Six dots in two columns: the drag handle for a row.
fn draw_grip(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let c = rect.center();
    for dy in [-6.0, 0.0, 6.0] {
        for dx in [-3.0, 3.0] {
            painter.circle_filled(c + egui::vec2(dx, dy), 1.5, color);
        }
    }
}

/// A line where the dragged row would be inserted.
fn draw_drop_line(
    painter: &egui::Painter,
    rows: &[(usize, egui::Rect)],
    before: usize,
    color: egui::Color32,
) {
    let Some((_, first)) = rows.first() else {
        return;
    };
    let y = match rows.iter().find(|(i, _)| *i == before) {
        Some((_, r)) => r.top(),
        None => rows.last().map_or(first.top(), |(_, r)| r.bottom()),
    };
    painter.line_segment(
        [egui::pos2(first.left(), y), egui::pos2(first.right(), y)],
        egui::Stroke::new(2.0, color),
    );
}
