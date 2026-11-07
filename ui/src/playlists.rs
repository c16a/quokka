//! Playlists: the list of playlists, and one playlist's tracks with play, shuffle, rename,
//! reorder (drag the grip) and remove.

use std::collections::HashMap;

use eframe::egui;
use quokka_core::{Library, MediaItem, PlaylistSummary};

use crate::albums::{
    PlayRequest, back_icon_button, play_icon_button, shuffle_icon_button, shuffled,
};
use crate::covers::{self, Cover, THUMB_MAX_SIDE, Worker, evict, upload};
use crate::format;
use crate::icons;
use crate::playback::MenuRequest;
use crate::queue;
use crate::search::search_fill;
use crate::squircle::{self, Padding, SquircleButton};

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
    thumbs: Thumbs,
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
    /// Whether one playlist's page is open rather than the list.
    pub fn showing_detail(&self) -> bool {
        self.selected.is_some()
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        library: &Library,
        current: Option<i64>,
        queue: &mut Vec<MenuRequest>,
        playlists: &[PlaylistSummary],
    ) -> Option<PlayRequest> {
        match self.selected {
            None => {
                crate::page(ui, egui::Margin::symmetric(32, 32), |ui| {
                    self.list(ui, library)
                });
                None
            }
            Some(_) => self.detail(ui, library, current, queue, playlists),
        }
    }

    /// The list of playlists, with a field to create one.
    fn list(&mut self, ui: &mut egui::Ui, library: &Library) {
        crate::fonts::page_title(ui, "Playlists");
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
                        crate::squircle::SquircleTextField::singleline(&mut self.new_name)
                            .hint_text("New playlist name")
                            .desired_width(200.0)
                            .show(ui)
                    })
                    .inner
                });
            let field = framed.inner;
            // The button is as tall as the field, with the same corners as the Sync now button.
            let height = framed.response.rect.height();
            let create = ui
                .add_enabled_ui(!self.new_name.trim().is_empty(), |ui| {
                    let label = egui::RichText::new("Create").color(egui::Color32::WHITE);
                    let vertical = ((height - 18.0) / 2.0).max(0.0);
                    SquircleButton::new(egui::Label::new(label).extend())
                        .padding(Padding::symmetric(18.0, vertical))
                        .fill(ui.visuals().selection.bg_fill)
                        .show(ui)
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
        {
            // As many fixed-width columns as the window fits.
            let columns = (((ui.available_width() + CARD_GAP) / (CARD_WIDTH + CARD_GAP)).floor()
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
                            if squircle::text_button(ui, "Rename").clicked() {
                                rename = Some((playlist.id, playlist.name.clone()));
                                ui.close();
                            }
                            if squircle::text_button(ui, "Delete playlist").clicked() {
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
        }
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
        queue: &mut Vec<MenuRequest>,
        playlists: &[PlaylistSummary],
    ) -> Option<PlayRequest> {
        let mut play = None;
        let mut back = false;
        let mut remove = None;
        self.thumbs.poll(ui.ctx());
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
                let field = crate::squircle::SquircleTextField::singleline(buffer)
                    .desired_width(260.0)
                    .show(ui);
                if !field.has_focus() && !field.lost_focus() {
                    field.request_focus();
                }
                let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
                if squircle::text_button(ui, "Save").clicked() || enter {
                    finish_rename = Some(true);
                } else if squircle::text_button(ui, "Cancel").clicked() || escape {
                    finish_rename = Some(false);
                }
            }),
            None => ui.horizontal(|ui| {
                crate::fonts::page_title(ui, &name);
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

        let mut drag_started = None;
        let mut drag_stopped = None;
        let mut hovered_now = None;
        let len = self.tracks.len();
        // Left and right edges of the list, and the height of its first row, once scrolled.
        let mut geometry = None;
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let output = egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, ROW_PITCH, len, |ui, range| {
                    for n in range {
                        let track = &self.tracks[n];
                        let background = ui.painter().add(egui::Shape::Noop);
                        let row_rect = ui
                            .allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), ROW_HEIGHT),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let (grip_rect, grip) = ui.allocate_exact_size(
                                        egui::vec2(14.0, 36.0),
                                        egui::Sense::drag(),
                                    );
                                    draw_grip(
                                        ui.painter(),
                                        grip_rect,
                                        ui.visuals().weak_text_color(),
                                    );
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
                                    // The position in the playlist, as in the album listing. The track
                                    // that is playing shows the equaliser instead.
                                    let (number, _) = ui.allocate_exact_size(
                                        egui::vec2(28.0, 36.0),
                                        egui::Sense::hover(),
                                    );
                                    if current == Some(track.id) {
                                        icons::paint(
                                            ui.painter(),
                                            number.center(),
                                            icons::EQUALIZER,
                                            16.0,
                                            ui.visuals().hyperlink_color,
                                        );
                                    } else {
                                        ui.painter().text(
                                            number.left_center(),
                                            egui::Align2::LEFT_CENTER,
                                            (n + 1).to_string(),
                                            egui::FontId::proportional(13.0),
                                            ui.visuals().weak_text_color(),
                                        );
                                    }
                                    let cover = self.thumbs.get(ui.ctx(), track);
                                    thumbnail(ui, cover.as_ref());
                                    ui.add_space(8.0);
                                    // Title and artist: one block, centred on the row by the ink of its glyphs.
                                    let (block, _) = ui.allocate_exact_size(
                                        egui::vec2(240.0, ROW_HEIGHT),
                                        egui::Sense::hover(),
                                    );
                                    let title = format::title(track);
                                    let artist = track
                                        .artist
                                        .as_deref()
                                        .or(track.album_artist.as_deref())
                                        .unwrap_or("")
                                        .to_owned();
                                    let (title_colour, title_font) = if current == Some(track.id) {
                                        (
                                            ui.visuals().hyperlink_color,
                                            egui::FontId::new(14.0, crate::fonts::bold_family()),
                                        )
                                    } else {
                                        (
                                            ui.visuals().text_color(),
                                            egui::FontId::proportional(14.0),
                                        )
                                    };
                                    let artist_colour = ui.visuals().weak_text_color();
                                    let painter = ui.painter();
                                    let title_galley =
                                        painter.layout_no_wrap(title, title_font, title_colour);
                                    let artist_galley = painter.layout_no_wrap(
                                        artist,
                                        egui::FontId::proportional(12.0),
                                        artist_colour,
                                    );
                                    // The ink runs from the top of the title's glyphs to the bottom of the artist's.
                                    let ink_top = title_galley.mesh_bounds.top();
                                    let ink_bottom = title_galley.size().y
                                        + TITLE_GAP
                                        + artist_galley.mesh_bounds.bottom();
                                    let top = block.center().y - (ink_top + ink_bottom) / 2.0;
                                    let artist_top = top + title_galley.size().y + TITLE_GAP;
                                    painter.galley(
                                        egui::pos2(block.left(), top),
                                        title_galley,
                                        title_colour,
                                    );
                                    painter.galley(
                                        egui::pos2(block.left(), artist_top),
                                        artist_galley,
                                        artist_colour,
                                    );
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
                                                    ui.ctx().set_cursor_icon(
                                                        egui::CursorIcon::PointingHand,
                                                    );
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
                                },
                            )
                            .response
                            .rect;
                        ui.interact(
                            row_rect,
                            ui.scope_id().with(("playlist_row", n)),
                            egui::Sense::click(),
                        )
                        .context_menu(|ui| queue::track_menu(ui, queue, track, playlists));
                        if current == Some(track.id) {
                            ui.painter().set(
                                background,
                                egui::Shape::rect_filled(
                                    row_rect,
                                    8.0,
                                    ui.visuals().faint_bg_color,
                                ),
                            );
                        }
                        if ui.rect_contains_pointer(row_rect) {
                            hovered_now = Some(n);
                        }
                        ui.add_space(ROW_PITCH - ROW_HEIGHT);
                    }
                });
            let area = output.inner_rect;
            geometry = Some((
                area.left(),
                area.right(),
                area.top() - output.state.clamped_offset().y,
            ));
        });

        self.hovered_row = hovered_now;
        if let Some(n) = drag_started {
            self.dragging = Some(n);
        }
        let pointer = ui.ctx().input(|i| i.pointer.interact_pos());
        if let (Some(from), Some(_)) = (drag_stopped, self.dragging.take()) {
            // Dropped above the row under the pointer, or at the end of the list.
            if let (Some(p), Some((_, _, origin))) = (pointer, geometry) {
                let before = insertion_index(p.y, origin, len);
                self.move_track(library, from, before);
            }
        } else if self.dragging.is_some()
            && let (Some(p), Some((left, right, origin))) = (pointer, geometry)
        {
            let before = insertion_index(p.y, origin, len);
            draw_drop_line(
                ui.painter(),
                left,
                right,
                origin + before as f32 * ROW_PITCH - 2.0,
                ui.visuals().selection.bg_fill,
            );
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

/// Height of a track row, and the distance from one row's top to the next.
const ROW_HEIGHT: f32 = 52.0;
/// Gap between a track's title line and its artist line.
const TITLE_GAP: f32 = 2.0;
const ROW_PITCH: f32 = 56.0;

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
                    // The text is centred vertically against the tile, within the card's height.
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), CARD_HEIGHT - 24.0),
                        egui::Layout::top_down(egui::Align::Min)
                            .with_main_align(egui::Align::Center),
                        |ui| {
                            // Nudged down: the line boxes sit high against the tile.
                            ui.add_space(4.0);
                            ui.add(
                                egui::Label::new(egui::RichText::new(&playlist.name).strong())
                                    .truncate(),
                            );
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!("{} tracks", playlist.track_count))
                                    .weak(),
                            ));
                        },
                    );
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

/// Most track thumbnails kept as textures.
const THUMB_CAP: usize = 1000;

/// Where a track's thumbnail stands: being decoded, or done (`None` means there is no cover).
#[derive(Clone)]
enum ThumbState {
    Loading,
    Ready(Option<Cover>),
}

/// Track thumbnails, decoded off the UI thread and kept by track id. The least recently used are
/// dropped beyond `THUMB_CAP`.
#[derive(Default)]
struct Thumbs {
    entries: HashMap<i64, (u64, ThumbState)>,
    tick: u64,
    worker: Option<Worker<i64>>,
}

impl Thumbs {
    /// Take in the thumbnails decoded since the last frame, and drop the least recently used
    /// beyond the cap.
    fn poll(&mut self, ctx: &egui::Context) {
        let Some(worker) = &self.worker else {
            return;
        };
        for (id, decoded) in worker.finished() {
            let cover = decoded.map(|decoded| upload(ctx, &format!("track-thumb-{id}"), decoded));
            if let Some((_, state)) = self.entries.get_mut(&id) {
                *state = ThumbState::Ready(cover);
            }
        }
        evict(&mut self.entries, THUMB_CAP, |state| {
            matches!(state, ThumbState::Ready(_))
        });
    }

    /// The track's thumbnail, or `None` while it decodes. The first request queues the decoding.
    fn get(&mut self, ctx: &egui::Context, track: &MediaItem) -> Option<Cover> {
        self.tick += 1;
        let tick = self.tick;
        if let Some((used, state)) = self.entries.get_mut(&track.id) {
            *used = tick;
            return match state {
                ThumbState::Ready(cover) => cover.clone(),
                ThumbState::Loading => None,
            };
        }
        let path = track.path.clone();
        let worker = self.worker.get_or_insert_with(|| Worker::new(ctx));
        worker.submit(
            track.id,
            Box::new(move || {
                quokka_core::read_cover(&path)
                    .and_then(|bytes| covers::decode(&bytes, THUMB_MAX_SIDE))
            }),
        );
        self.entries.insert(track.id, (tick, ThumbState::Loading));
        None
    }
}

/// Draw a 36 px square thumbnail, or a plain square while it loads or when there is no cover.
fn thumbnail(ui: &mut egui::Ui, cover: Option<&Cover>) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
    match cover {
        Some(cover) => {
            ui.put(
                rect,
                egui::Image::new(&cover.colour)
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
fn draw_drop_line(painter: &egui::Painter, left: f32, right: f32, y: f32, color: egui::Color32) {
    painter.line_segment(
        [egui::pos2(left, y), egui::pos2(right, y)],
        egui::Stroke::new(2.0, color),
    );
}

/// The position a row dropped at height `y` goes to: before the first row whose centre is below
/// `y`. Rows start at `origin` and are `ROW_PITCH` apart.
fn insertion_index(y: f32, origin: f32, len: usize) -> usize {
    let index = ((y - origin - ROW_HEIGHT / 2.0) / ROW_PITCH).floor() as isize + 1;
    index.clamp(0, len as isize) as usize
}
