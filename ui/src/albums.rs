use eframe::egui;
use player_core::{AlbumSummary, Library, MediaItem};

use crate::format;

/// Height of a track row's content. Vertical padding comes from the row's frame.
const TRACK_ROW_HEIGHT: f32 = 52.0;

/// Space above and below the title and artist when a track row is highlighted.
const TRACK_PADDING: f32 = 6.0;

/// A request to play `tracks`, starting at `index`.
pub struct PlayRequest {
    pub tracks: Vec<MediaItem>,
    pub index: usize,
}

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

    /// Draw the view. Returns a play request when the user asks to play something.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        library: &Library,
        current: Option<i64>,
    ) -> Option<PlayRequest> {
        if self.albums.is_empty() {
            ui.label("No albums yet. Choose your library folder in Settings.");
            return None;
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

        let mut play = None;
        egui::CentralPanel::default().show(ui, |ui| {
            let Some(i) = self.selected else {
                ui.weak("Select an album.");
                return;
            };
            let album = &self.albums[i];
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(album.album.as_deref().unwrap_or("Unknown album"));
                    if !album.composers.is_empty() {
                        ui.label(album.composers.join(", "));
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if play_icon_button(ui).on_hover_text("Play album").clicked() {
                        play = Some(PlayRequest {
                            tracks: self.tracks.clone(),
                            index: 0,
                        });
                    }
                    ui.add_space(8.0);
                    if shuffle_icon_button(ui)
                        .on_hover_text("Shuffle album")
                        .clicked()
                    {
                        play = Some(PlayRequest {
                            tracks: shuffled(self.tracks.clone()),
                            index: 0,
                        });
                    }
                });
            });
            ui.separator();
            // Solid scroll bars take their own width, so they don't cover the track durations.
            ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, TRACK_ROW_HEIGHT, self.tracks.len(), |ui, range| {
                    for n in range {
                        let track = &self.tracks[n];
                        let is_current = current == Some(track.id);
                        // Reserved now, filled in once we know whether the row is hovered.
                        let background = ui.painter().add(egui::Shape::Noop);
                        let mut text_rect = egui::Rect::NOTHING;
                        let row = egui::Frame::new()
                            .inner_margin(egui::Margin::symmetric(8, 6))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let number =
                                        track.track.map(|t| t.to_string()).unwrap_or_default();
                                    ui.add_sized([32.0, 40.0], egui::Label::new(number));
                                    let block = ui.vertical(|ui| {
                                        let title = format::title(track);
                                        if is_current {
                                            ui.strong(title);
                                        } else {
                                            ui.label(title);
                                        }
                                        if let Some(artist) = track
                                            .artist
                                            .as_deref()
                                            .or(track.album_artist.as_deref())
                                        {
                                            ui.weak(artist);
                                        }
                                    });
                                    text_rect = block.response.rect;
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            // Gap between the duration and the scroll bar.
                                            ui.add_space(16.0);
                                            ui.weak(format::duration(track.duration_ms));
                                        },
                                    );
                                });
                            })
                            .response;

                        // The whole padded row is the hover and click target.
                        let response = ui.interact(
                            row.rect,
                            ui.scope_id().with(("track_row", n)),
                            egui::Sense::click(),
                        );
                        if response.hovered() {
                            // Padding is measured from the title and artist, so it is even above and below.
                            let highlight = egui::Rect::from_min_max(
                                egui::pos2(row.rect.left(), text_rect.top() - TRACK_PADDING),
                                egui::pos2(row.rect.right(), text_rect.bottom() + TRACK_PADDING),
                            );
                            ui.painter().set(
                                background,
                                egui::Shape::rect_filled(
                                    highlight,
                                    6.0,
                                    ui.visuals().faint_bg_color,
                                ),
                            );
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        // Double-click anywhere on the row to play from this track.
                        if response.double_clicked() {
                            play = Some(PlayRequest {
                                tracks: self.tracks.clone(),
                                index: n,
                            });
                        }
                    }
                });
        });
        play
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

/// Round button with a play triangle. No text, so it needs no font glyph.
fn play_icon_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    ui.painter().rect_filled(rect, 16.0, visuals.weak_bg_fill);
    let c = rect.center();
    let points = vec![
        egui::pos2(c.x - 4.0, c.y - 7.0),
        egui::pos2(c.x - 4.0, c.y + 7.0),
        egui::pos2(c.x + 8.0, c.y),
    ];
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        visuals.fg_stroke.color,
        egui::Stroke::NONE,
    ));
    response
}

/// The tracks in a random order (Fisher-Yates). The generator is seeded from the clock;
/// nothing here needs real randomness, so no extra crate is used.
fn shuffled(mut tracks: Vec<MediaItem>) -> Vec<MediaItem> {
    let mut state = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E37_79B9_7F4A_7C15)
        | 1;
    for i in (1..tracks.len()).rev() {
        // xorshift64
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let j = (state % (i as u64 + 1)) as usize;
        tracks.swap(i, j);
    }
    tracks
}

/// Round button with a shuffle icon (two crossing arrows). No text, so it needs no font glyph.
fn shuffle_icon_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    ui.painter().rect_filled(rect, 16.0, visuals.weak_bg_fill);
    let c = rect.center();
    let stroke = egui::Stroke::new(1.6, visuals.fg_stroke.color);
    let (left, right) = (c.x - 7.0, c.x + 7.0);
    let (top, bottom) = (c.y - 6.0, c.y + 6.0);
    let painter = ui.painter();
    // Two crossing lines, each ending in an arrow tip.
    painter.line_segment([egui::pos2(left, bottom), egui::pos2(right, top)], stroke);
    painter.line_segment([egui::pos2(left, top), egui::pos2(right, bottom)], stroke);
    painter.line_segment(
        [egui::pos2(right - 3.5, top), egui::pos2(right, top)],
        stroke,
    );
    painter.line_segment(
        [egui::pos2(right, top), egui::pos2(right, top + 3.5)],
        stroke,
    );
    painter.line_segment(
        [egui::pos2(right - 3.5, bottom), egui::pos2(right, bottom)],
        stroke,
    );
    painter.line_segment(
        [egui::pos2(right, bottom), egui::pos2(right, bottom - 3.5)],
        stroke,
    );
    response
}
