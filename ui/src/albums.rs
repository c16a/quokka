use std::collections::HashMap;

use eframe::egui;
use player_core::{AlbumSummary, Library, MediaItem};

use crate::format;

/// Height of a track row's content. Vertical padding comes from the row's frame.
const TRACK_ROW_HEIGHT: f32 = 52.0;
/// Space above and below the title and artist when a track row is highlighted.
const TRACK_PADDING: f32 = 6.0;

/// Square side of a cover in the album grid, and in the album detail view.
const TILE_COVER: f32 = 160.0;
const COVER_DETAIL: f32 = 240.0;
/// Space under a grid tile's cover for its title and composer, and the gap between tiles.
const TILE_TEXT: f32 = 44.0;
const TILE_GAP: f32 = 20.0;
/// Largest side of a decoded cover. Bigger covers are scaled down to save memory.
const COVER_MAX_SIDE: u32 = 256;

/// A request to play `tracks`, starting at `index`.
pub struct PlayRequest {
    pub tracks: Vec<MediaItem>,
    pub index: usize,
}

/// The album grid, or one album's detail view (cover, actions and tracks) when one is selected.
#[derive(Default)]
pub struct AlbumsView {
    albums: Vec<AlbumSummary>,
    selected: Option<usize>,
    tracks: Vec<MediaItem>,
    /// Decoded covers by album index, loaded when first shown. `None` means no cover.
    covers: HashMap<usize, Option<egui::TextureHandle>>,
}

impl AlbumsView {
    pub fn reload(&mut self, library: &Library) {
        self.covers.clear();
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
        let ctx = ui.ctx().clone();

        let mut play = None;
        let mut back = false;
        let mut open = None;
        match self.selected {
            None => open = self.grid(ui, &ctx, library),
            Some(i) => {
                let (request, go_back) = self.detail(ui, &ctx, library, current, i);
                play = request;
                back = go_back;
            }
        }

        if let Some(i) = open {
            self.selected = Some(i);
            self.load_tracks(library);
        }
        if back {
            self.selected = None;
            self.load_tracks(library);
        }
        play
    }

    /// The album grid. Returns the album the user clicked, if any.
    fn grid(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, library: &Library) -> Option<usize> {
        let columns = ((ui.available_width() + TILE_GAP) / (TILE_COVER + TILE_GAP))
            .floor()
            .max(1.0) as usize;
        let row_height = TILE_COVER + TILE_TEXT + TILE_GAP;
        let rows = self.albums.len().div_ceil(columns);
        let mut opened = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, rows, |ui, range| {
                for row in range {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = TILE_GAP;
                        for column in 0..columns {
                            let i = row * columns + column;
                            if i >= self.albums.len() {
                                break;
                            }
                            let album = &self.albums[i];
                            let cover = cover_texture(&mut self.covers, ctx, library, i, album);
                            if album_tile(ui, cover.as_ref(), album).clicked() {
                                opened = Some(i);
                            }
                        }
                    });
                }
            });
        opened
    }

    /// One album: back button, cover, title, composers, actions, then the tracks.
    /// Returns a play request, and whether the user asked to go back to the grid.
    fn detail(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        library: &Library,
        current: Option<i64>,
        i: usize,
    ) -> (Option<PlayRequest>, bool) {
        let mut play = None;
        let mut back = false;
        let album = &self.albums[i];
        let cover = cover_texture(&mut self.covers, ctx, library, i, album);

        if back_icon_button(ui).on_hover_text("All albums").clicked() {
            back = true;
        }
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            cover_image(ui, cover.as_ref(), COVER_DETAIL);
            ui.add_space(24.0);
            ui.vertical(|ui| {
                ui.add_space(4.0);
                ui.heading(album.album.as_deref().unwrap_or("Unknown album"));
                if !album.composers.is_empty() {
                    ui.label(album.composers.join(", "));
                }
                let year = album.year.map(|y| format!(" · {y}")).unwrap_or_default();
                ui.weak(format!("{} tracks{year}", album.track_count));
                ui.add_space(16.0);
                ui.horizontal(|ui| {
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
                                let number = track.track.map(|t| t.to_string()).unwrap_or_default();
                                ui.add_sized([32.0, 40.0], egui::Label::new(number));
                                let block = ui.vertical(|ui| {
                                    let title = format::title(track);
                                    if is_current {
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
                            egui::Shape::rect_filled(highlight, 6.0, ui.visuals().faint_bg_color),
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
        (play, back)
    }
}

/// The album's cover as a texture. It is decoded once, then kept in `covers`.
/// `None` means the album has no readable cover.
fn cover_texture(
    covers: &mut HashMap<usize, Option<egui::TextureHandle>>,
    ctx: &egui::Context,
    library: &Library,
    index: usize,
    album: &AlbumSummary,
) -> Option<egui::TextureHandle> {
    covers
        .entry(index)
        .or_insert_with(|| {
            let bytes = library.album_cover(album.album.as_deref()).ok().flatten()?;
            let image = image::load_from_memory(&bytes)
                .ok()?
                .thumbnail(COVER_MAX_SIDE, COVER_MAX_SIDE)
                .to_rgba8();
            let size = [image.width() as usize, image.height() as usize];
            let color = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
            Some(ctx.load_texture(
                format!("album-cover-{index}"),
                color,
                egui::TextureOptions::LINEAR,
            ))
        })
        .clone()
}

/// Draw a square cover, or a plain square when there is no cover.
fn cover_image(ui: &mut egui::Ui, texture: Option<&egui::TextureHandle>, size: f32) {
    match texture {
        Some(texture) => {
            ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(size, size)));
        }
        None => {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 6.0, ui.visuals().faint_bg_color);
        }
    }
}

/// A grid tile: square cover, then the album name and its composers. Text that doesn't fit is clipped.
fn album_tile(
    ui: &mut egui::Ui,
    cover: Option<&egui::TextureHandle>,
    album: &AlbumSummary,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(TILE_COVER, TILE_COVER + TILE_TEXT),
        egui::Sense::click(),
    );
    let cover_rect = egui::Rect::from_min_size(rect.min, egui::vec2(TILE_COVER, TILE_COVER));
    let painter = ui.painter_at(rect);
    match cover {
        Some(texture) => painter.image(
            texture.id(),
            cover_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        ),
        None => painter.rect_filled(cover_rect, 6.0, ui.visuals().faint_bg_color),
    };
    if response.hovered() {
        painter.rect_stroke(
            cover_rect,
            6.0,
            egui::Stroke::new(2.0, ui.visuals().selection.bg_fill),
            egui::StrokeKind::Outside,
        );
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let text_top = cover_rect.bottom() + 6.0;
    painter.text(
        egui::pos2(rect.left(), text_top),
        egui::Align2::LEFT_TOP,
        album.album.as_deref().unwrap_or("Unknown album"),
        egui::FontId::proportional(14.0),
        ui.visuals().text_color(),
    );
    if !album.composers.is_empty() {
        painter.text(
            egui::pos2(rect.left(), text_top + 20.0),
            egui::Align2::LEFT_TOP,
            album.composers.join(", "),
            egui::FontId::proportional(12.0),
            ui.visuals().weak_text_color(),
        );
    }
    response
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

/// Round button with a left chevron, for going back to the album grid.
fn back_icon_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    ui.painter().rect_filled(rect, 16.0, visuals.weak_bg_fill);
    let c = rect.center();
    let stroke = egui::Stroke::new(2.0, visuals.fg_stroke.color);
    ui.painter().line_segment(
        [egui::pos2(c.x + 3.0, c.y - 6.0), egui::pos2(c.x - 3.0, c.y)],
        stroke,
    );
    ui.painter().line_segment(
        [egui::pos2(c.x - 3.0, c.y), egui::pos2(c.x + 3.0, c.y + 6.0)],
        stroke,
    );
    response
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
