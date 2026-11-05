//! Playlists: the list of playlists, and one playlist's tracks with play, shuffle, rename,
//! reorder (drag the grip) and remove.

use std::collections::HashMap;

use eframe::egui;
use player_core::{Library, MediaItem, PlaylistSummary};

use crate::albums::{
    PlayRequest, back_icon_button, play_icon_button, shuffle_icon_button, shuffled,
};
use crate::format;
use crate::icons;
use crate::search::search_fill;

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
    /// Small covers of the open playlist's tracks, by track id.
    thumbs: HashMap<i64, Option<egui::TextureHandle>>,
    /// The row the pointer was over in the last frame; its remove button is shown.
    hovered_row: Option<usize>,
}

impl PlaylistsView {
    /// All playlists, for the menus that add tracks to one.
    pub fn summaries(&self) -> &[PlaylistSummary] {
        &self.playlists
    }

    /// Open one playlist, as if it had been clicked in the list.
    pub fn open(&mut self, id: i64, library: &Library) {
        self.selected = Some(id);
        self.renaming = None;
        self.load_tracks(library);
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
            // The same filled field as the search box, with a plus in place of the magnifier.
            let framed = egui::Frame::new()
                .fill(search_fill(ui))
                .corner_radius(10.0)
                .inner_margin(egui::Margin::symmetric(12, 9))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                        icons::paint(
                            ui.painter(),
                            rect.center(),
                            icons::PLUS,
                            16.0,
                            ui.visuals().weak_text_color(),
                        );
                        ui.add_space(6.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_name)
                                .frame(egui::Frame::NONE)
                                .hint_text("New playlist name")
                                .margin(egui::Margin::symmetric(10, 6))
                                .align(egui::Align2::LEFT_CENTER)
                                .desired_width(200.0),
                        )
                    })
                    .inner
                });
            let field = framed.inner;
            // The button is as tall as the field, with the same corners as the Sync now button.
            let height = framed.response.rect.height();
            let create = ui
                .scope(|ui| {
                    // Wider than the default button padding, so the label has room either side.
                    ui.spacing_mut().button_padding = egui::vec2(18.0, 6.0);
                    ui.add_enabled(
                        !self.new_name.trim().is_empty(),
                        egui::Button::new(
                            egui::RichText::new("Create").color(egui::Color32::WHITE),
                        )
                        .fill(ui.visuals().selection.bg_fill)
                        .stroke(egui::Stroke::NONE)
                        .corner_radius(6.0)
                        .min_size(egui::vec2(0.0, height)),
                    )
                })
                .inner;
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
                // As many fixed-width columns as the window fits.
                let columns =
                    (((ui.available_width() + CARD_GAP) / (CARD_WIDTH + CARD_GAP)).floor()
                        as usize)
                        .max(1);
                egui::Grid::new("playlist_grid")
                    .num_columns(columns)
                    .spacing([CARD_GAP, CARD_GAP])
                    .min_col_width(CARD_WIDTH)
                    .max_col_width(CARD_WIDTH)
                    .show(ui, |ui| {
                        for (n, playlist) in self.playlists.iter().enumerate() {
                            let response = playlist_card(ui, playlist);
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
                            if (n + 1) % columns == 0 {
                                ui.end_row();
                            }
                        }
                        ui.end_row();
                    });
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
                let field = ui.add(
                    egui::TextEdit::singleline(buffer)
                        .margin(egui::Margin::symmetric(10, 6))
                        .align(egui::Align2::LEFT_CENTER)
                        .desired_width(260.0),
                );
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
        let mut hovered_now = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (n, track) in self.tracks.iter().enumerate() {
                    let row_rect = ui
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
                            // The position in the playlist, numbered as in the album listing.
                            let (number, _) = ui
                                .allocate_exact_size(egui::vec2(28.0, 36.0), egui::Sense::hover());
                            ui.painter().text(
                                number.left_center(),
                                egui::Align2::LEFT_CENTER,
                                (n + 1).to_string(),
                                egui::FontId::proportional(13.0),
                                ui.visuals().weak_text_color(),
                            );
                            let texture = track_thumbnail(&mut self.thumbs, ui.ctx(), track);
                            thumbnail(ui, texture.as_ref());
                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                let title = format::title(track);
                                if current == Some(track.id) {
                                    ui.label(crate::fonts::bold(title));
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
                                    // The minus appears only while the row is hovered.
                                    if self.hovered_row == Some(n) {
                                        let (rect, button) = ui.allocate_exact_size(
                                            egui::vec2(24.0, 24.0),
                                            egui::Sense::click(),
                                        );
                                        let colour = if button.hovered() {
                                            ui.visuals().strong_text_color()
                                        } else {
                                            ui.visuals().weak_text_color()
                                        };
                                        if button.hovered() {
                                            ui.ctx()
                                                .set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        icons::paint(
                                            ui.painter(),
                                            rect.center(),
                                            icons::MINUS,
                                            16.0,
                                            colour,
                                        );
                                        if button.clicked() {
                                            remove = Some(n);
                                        }
                                    } else {
                                        ui.add_space(24.0);
                                    }
                                    ui.add_space(8.0);
                                    ui.weak(format::duration(track.duration_ms));
                                },
                            );
                        })
                        .response
                        .rect;
                    if ui.rect_contains_pointer(row_rect) {
                        hovered_now = Some(n);
                    }
                    rows.push((n, row_rect));
                    ui.add_space(4.0);
                }
            });

        self.hovered_row = hovered_now;
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

/// Size of a playlist card, and the gap between cards. They match the radio station cards.
pub(crate) const CARD_WIDTH: f32 = 260.0;
const CARD_HEIGHT: f32 = 66.0;
pub(crate) const CARD_GAP: f32 = 12.0;

/// A playlist as a card: a tile with its initials, the name and the track count.
pub(crate) fn playlist_card(ui: &mut egui::Ui, playlist: &PlaylistSummary) -> egui::Response {
    let frame = egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(12.0)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(CARD_WIDTH - 24.0);
            // The content is centred vertically within the card's fixed height.
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), CARD_HEIGHT - 24.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let (tile, _) =
                        ui.allocate_exact_size(egui::vec2(42.0, 42.0), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(tile, 42.0 * 0.22, ui.visuals().selection.bg_fill);
                    ui.painter().text(
                        tile.center(),
                        egui::Align2::CENTER_CENTER,
                        initials(&playlist.name),
                        egui::FontId::proportional(15.0),
                        egui::Color32::WHITE,
                    );
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        // Two lines of text: centre them against the tile.
                        ui.add_space(((CARD_HEIGHT - 24.0) - 38.0) / 2.0);
                        ui.add(
                            egui::Label::new(egui::RichText::new(&playlist.name).strong())
                                .truncate(),
                        );
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("{} tracks", playlist.track_count)).weak(),
                        ));
                    });
                },
            );
        })
        .response;
    if ui.rect_contains_pointer(frame.rect) {
        ui.painter().rect_stroke(
            frame.rect,
            12.0,
            egui::Stroke::new(1.5, ui.visuals().selection.bg_fill),
            egui::StrokeKind::Inside,
        );
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    ui.interact(
        frame.rect,
        ui.scope_id().with(("playlist_card", playlist.id)),
        egui::Sense::click(),
    )
}

/// The first letter of the first two words, in capitals.
fn initials(name: &str) -> String {
    name.split_whitespace()
        .take(2)
        .filter_map(|word| word.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

/// The track's embedded cover as a small texture. It is decoded once per track.
fn track_thumbnail(
    thumbs: &mut HashMap<i64, Option<egui::TextureHandle>>,
    ctx: &egui::Context,
    track: &MediaItem,
) -> Option<egui::TextureHandle> {
    thumbs
        .entry(track.id)
        .or_insert_with(|| {
            let bytes = player_core::read_cover(&track.path)?;
            let image = image::load_from_memory(&bytes)
                .ok()?
                .thumbnail(72, 72)
                .to_rgba8();
            let size = [image.width() as usize, image.height() as usize];
            let colour = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
            Some(ctx.load_texture(
                format!("track-thumb-{}", track.id),
                colour,
                egui::TextureOptions::LINEAR,
            ))
        })
        .clone()
}

/// Draw a 36 px square thumbnail, or a plain square when there is no cover.
fn thumbnail(ui: &mut egui::Ui, texture: Option<&egui::TextureHandle>) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
    match texture {
        Some(texture) => {
            ui.put(
                rect,
                egui::Image::new(texture)
                    .fit_to_exact_size(rect.size())
                    .corner_radius(4.0),
            );
        }
        None => {
            ui.painter()
                .rect_filled(rect, 4.0, ui.visuals().faint_bg_color);
        }
    }
}

/// The drag handle for a row.
fn draw_grip(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    icons::paint(
        painter,
        rect.center(),
        icons::DOTS_SIX_VERTICAL,
        16.0,
        color,
    );
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
