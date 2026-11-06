use std::collections::{HashMap, HashSet};

use eframe::egui;
use player_core::{
    AlbumFilter, AlbumSort, AlbumSummary, FavoriteKind, Library, MediaItem, PlaylistSummary,
};

use crate::covers::{self, COVER_MAX_SIDE, Cover, Worker, evict, upload};
use crate::format;
use crate::icons;
use crate::playback::MenuRequest;
use crate::queue;

/// Height of a track row's content. Vertical padding comes from the row's frame.
const TRACK_ROW_HEIGHT: f32 = 52.0;

/// Colour of a favourite star.
const FAVORITE_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 196, 64);

/// The name of a sort order, as shown in the grid's menu.
fn sort_label(sort: AlbumSort) -> &'static str {
    match sort {
        AlbumSort::Name => "Name",
        AlbumSort::Year => "Year",
        AlbumSort::Added => "Date added",
    }
}

/// Square side of a cover in the album grid, and in the album detail view.
pub(crate) const TILE_COVER: f32 = 160.0;
const COVER_DETAIL: f32 = 220.0;
/// Codecs that are lossless, shown as "lossless" in an album's details.
const LOSSLESS: [&str; 7] = ["FLAC", "ALAC", "WAV", "AIFF", "APE", "WAVPACK", "PCM"];
/// Space under a grid tile's cover for its title and composer, and the gap between tiles.
pub(crate) const TILE_TEXT: f32 = 44.0;
/// Corner radius of a grid cover.
const TILE_RADIUS: f32 = 14.0;
pub(crate) const TILE_GAP: f32 = 20.0;
/// Most covers kept as textures. The least recently used are dropped beyond this.
const COVER_CAP: usize = 300;

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
    /// Covers by album index, with the tick they were last used on.
    covers: HashMap<usize, (u64, CoverSlot)>,
    /// Decodes covers off the UI thread. Started on first use.
    cover_worker: Option<Worker<usize>>,
    cover_tick: u64,
    /// Keys of favourite tracks (file paths) and albums (album names).
    favorite_tracks: HashSet<String>,
    favorite_albums: HashSet<String>,
    /// How the grid is ordered, and which albums it keeps.
    sort: AlbumSort,
    filter: AlbumFilter,
    /// Choices offered by the filter menus.
    genres: Vec<String>,
    years: Vec<i64>,
}

impl AlbumsView {
    /// All albums, in the order the grid shows them.
    pub fn albums(&self) -> &[AlbumSummary] {
        &self.albums
    }

    /// The cover of album `index`. It is decoded on first use, off the UI thread.
    pub fn cover(&mut self, ctx: &egui::Context, library: &Library, index: usize) -> Option<Cover> {
        self.cover_for(ctx, library, index)
    }

    /// The cover of album `index`, or `None` while it is being decoded or when it has none. The
    /// first request queues the decoding, and the cover appears on a later frame.
    fn cover_for(&mut self, ctx: &egui::Context, library: &Library, index: usize) -> Option<Cover> {
        self.poll_covers(ctx);
        self.cover_tick += 1;
        let tick = self.cover_tick;
        if let Some((used, slot)) = self.covers.get_mut(&index) {
            *used = tick;
            return match slot {
                CoverSlot::Ready(cover) => cover.clone(),
                CoverSlot::Loading => None,
            };
        }
        let name = self.albums.get(index)?.album.clone();
        let slot = match library.album_cover(name.as_deref()).ok().flatten() {
            None => CoverSlot::Ready(None),
            Some(bytes) => {
                let worker = self.cover_worker.get_or_insert_with(|| Worker::new(ctx));
                worker.submit(
                    index,
                    Box::new(move || covers::decode(&bytes, COVER_MAX_SIDE)),
                );
                CoverSlot::Loading
            }
        };
        self.covers.insert(index, (tick, slot));
        evict(&mut self.covers, COVER_CAP, |slot| {
            matches!(slot, CoverSlot::Ready(_))
        });
        None
    }

    /// Take in the covers the worker has decoded, and upload them as textures.
    fn poll_covers(&mut self, ctx: &egui::Context) {
        let Some(worker) = &self.cover_worker else {
            return;
        };
        for (index, decoded) in worker.finished() {
            let cover =
                decoded.map(|decoded| upload(ctx, &format!("album-cover-{index}"), decoded));
            if let Some((_, slot)) = self.covers.get_mut(&index) {
                *slot = CoverSlot::Ready(cover);
            }
        }
    }

    /// Show album `index` in its detail view.
    pub fn open(&mut self, library: &Library, index: usize) {
        if index < self.albums.len() {
            self.selected = Some(index);
            self.load_tracks(library);
        }
    }

    pub fn reload(&mut self, library: &Library) {
        self.covers.clear();
        self.favorite_tracks = library.favorites(FavoriteKind::Track).unwrap_or_default();
        self.favorite_albums = library.favorites(FavoriteKind::Album).unwrap_or_default();
        self.genres = library.genres().unwrap_or_default();
        self.years = library.years().unwrap_or_default();
        self.albums = library
            .albums_by(self.sort, &self.filter)
            .unwrap_or_else(|e| {
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
        queue: &mut Vec<MenuRequest>,
        playlists: &[PlaylistSummary],
    ) -> Option<PlayRequest> {
        if self.selected.is_none() {
            crate::fonts::page_title(ui, "Albums");
            self.toolbar(ui, library);
        }
        if self.albums.is_empty() {
            if self.filter_active() {
                ui.label("No albums match these filters.");
            } else {
                ui.label("No albums yet. Choose your library folder in Settings.");
            }
            return None;
        }
        let ctx = ui.ctx().clone();

        let mut play = None;
        let mut back = false;
        let mut open = None;
        match self.selected {
            None => open = self.grid(ui, &ctx, library),
            Some(i) => {
                let (request, go_back) = self.detail(ui, library, current, i, queue, playlists);
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

    /// Whether a genre or year filter is narrowing the grid.
    fn filter_active(&self) -> bool {
        self.filter.genre.is_some() || self.filter.year.is_some()
    }

    /// Sort and filter menus above the grid. Changing one reloads the grid.
    fn toolbar(&mut self, ui: &mut egui::Ui, library: &Library) {
        let (sort, genre, year) = (self.sort, self.filter.genre.clone(), self.filter.year);
        let mut sort_to = sort;
        let mut genre_to = genre.clone();
        let mut year_to = year;
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("album_sort")
                .selected_text(sort_label(sort_to))
                .show_ui(ui, |ui| {
                    for option in [AlbumSort::Name, AlbumSort::Year, AlbumSort::Added] {
                        ui.selectable_value(&mut sort_to, option, sort_label(option));
                    }
                });
            egui::ComboBox::from_id_salt("album_genre")
                .selected_text(genre_to.clone().unwrap_or_else(|| "All genres".into()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut genre_to, None, "All genres");
                    for name in &self.genres {
                        ui.selectable_value(&mut genre_to, Some(name.clone()), name.as_str());
                    }
                });
            egui::ComboBox::from_id_salt("album_year")
                .selected_text(year_to.map_or("All years".to_owned(), |y| y.to_string()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut year_to, None, "All years");
                    for &y in &self.years {
                        ui.selectable_value(&mut year_to, Some(y), y.to_string());
                    }
                });
        });
        ui.add_space(8.0);
        if sort_to != sort || genre_to != genre || year_to != year {
            self.sort = sort_to;
            self.filter = AlbumFilter {
                genre: genre_to,
                year: year_to,
            };
            self.reload(library);
        }
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
                            let cover = self.cover_for(ctx, library, i);
                            let album = &self.albums[i];
                            if album_tile(ui, cover.as_ref(), album).clicked() {
                                opened = Some(i);
                            }
                        }
                    });
                }
            });
        opened
    }

    /// One album, as in the design: back link, cover with title and details, the actions, then the
    /// track list. Returns a play request, and whether the user asked to go back to the grid.
    fn detail(
        &mut self,
        ui: &mut egui::Ui,
        library: &Library,
        current: Option<i64>,
        i: usize,
        queue: &mut Vec<MenuRequest>,
        playlists: &[PlaylistSummary],
    ) -> (Option<PlayRequest>, bool) {
        let mut play = None;
        let mut back = false;
        let ctx = ui.ctx().clone();
        let palette = AlbumPalette::of(ui);
        let album = self.albums[i].clone();
        let cover = self.cover_for(&ctx, library, i).map(|c| c.colour);
        let album_key = album.album.clone().unwrap_or_default();
        let favorite_album = self.favorite_albums.contains(&album_key);
        let mut toggle_album = false;
        let mut toggle_track: Option<(String, bool)> = None;

        // The page sits 36 px from the sidebar and 26 px from the top, as in the design.
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(12, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if album_back_button(ui).on_hover_text("All albums").clicked() {
                        back = true;
                    }
                    ui.add_space(14.0);
                    ui.label(egui::RichText::new("Albums").size(13.0).color(palette.weak));
                });
                ui.add_space(26.0);

                // Cover on the left; the title block sits at the bottom of the cover's height.
                let title = album
                    .album
                    .clone()
                    .unwrap_or_else(|| "Unknown album".into());
                let artist = album.artists.join(", ");
                let total_ms: i64 = self.tracks.iter().filter_map(|t| t.duration_ms).sum();
                let first = self.tracks.first();
                let codec = first.and_then(|t| t.codec.clone());
                let mut meta = Vec::new();
                if let Some(year) = album.year {
                    meta.push(year.to_string());
                }
                meta.push(format!("{} tracks", album.track_count));
                if total_ms > 0 {
                    meta.push(format::long_duration(total_ms));
                }
                if let Some(codec) = codec {
                    let lossless = LOSSLESS.iter().any(|c| codec.eq_ignore_ascii_case(c));
                    let kbps = first.and_then(|t| t.bitrate_kbps);
                    meta.push(match kbps {
                        _ if lossless => format!("{codec} lossless"),
                        Some(kbps) => format!("{codec} {kbps} kbps"),
                        None => codec,
                    });
                }
                let meta = meta.join(" · ");

                ui.horizontal(|ui| {
                    cover_image(ui, cover.as_ref(), COVER_DETAIL);
                    ui.add_space(28.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), COVER_DETAIL),
                        egui::Layout::bottom_up(egui::Align::Min),
                        |ui| {
                            // Bottom-up: the actions come first, so they sit lowest.
                            ui.spacing_mut().item_spacing.y = 10.0;
                            ui.horizontal(|ui| {
                                if pill(ui, &palette, "Play", PillIcon::Play, true).clicked() {
                                    play = Some(PlayRequest {
                                        tracks: self.tracks.clone(),
                                        index: 0,
                                    });
                                }
                                ui.add_space(10.0);
                                if pill(ui, &palette, "Shuffle", PillIcon::Shuffle, false).clicked()
                                {
                                    play = Some(PlayRequest {
                                        tracks: shuffled(self.tracks.clone()),
                                        index: 0,
                                    });
                                }
                                ui.add_space(10.0);
                                let tip = if favorite_album {
                                    "Remove from favourites"
                                } else {
                                    "Add to favourites"
                                };
                                if star_circle(ui, &palette, favorite_album)
                                    .on_hover_text(tip)
                                    .clicked()
                                {
                                    toggle_album = true;
                                }
                            });
                            ui.label(egui::RichText::new(meta).size(13.0).color(palette.weak));
                            ui.label(egui::RichText::new(artist).size(16.0).color(palette.text));
                            ui.label(
                                egui::RichText::new(title)
                                    .size(36.0)
                                    .family(crate::fonts::bold_family())
                                    .color(palette.text),
                            );
                            ui.label(egui::RichText::new("ALBUM").size(11.0).color(palette.weak));
                        },
                    );
                });

                // Track list: a header row, then the tracks.
                ui.add_space(26.0);
                let width = ui.available_width();
                let (header, _) =
                    ui.allocate_exact_size(egui::vec2(width, 30.0), egui::Sense::hover());
                let painter = ui.painter();
                let header_font = egui::FontId::proportional(11.0);
                painter.text(
                    egui::pos2(header.left() + 14.0, header.center().y),
                    egui::Align2::LEFT_CENTER,
                    "#",
                    header_font.clone(),
                    palette.weak,
                );
                painter.text(
                    egui::pos2(header.left() + 62.0, header.center().y),
                    egui::Align2::LEFT_CENTER,
                    "TITLE",
                    header_font.clone(),
                    palette.weak,
                );
                painter.text(
                    egui::pos2(header.right() - 58.0, header.center().y),
                    egui::Align2::RIGHT_CENTER,
                    "TIME",
                    header_font,
                    palette.weak,
                );
                painter.line_segment(
                    [
                        egui::pos2(header.left(), header.bottom()),
                        egui::pos2(header.right(), header.bottom()),
                    ],
                    egui::Stroke::new(1.0, palette.line),
                );

                let count = self.tracks.len();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show_rows(ui, TRACK_ROW_HEIGHT, count, |ui, range| {
                        for n in range {
                            let track = &self.tracks[n];
                            let is_current = current == Some(track.id);
                            let width = ui.available_width();
                            let (row, response) = ui.allocate_exact_size(
                                egui::vec2(width, TRACK_ROW_HEIGHT),
                                egui::Sense::click(),
                            );
                            let centre_y = row.center().y;
                            let key = track.path.to_string_lossy().into_owned();
                            let favorite = self.favorite_tracks.contains(&key);
                            let star = egui::Rect::from_center_size(
                                egui::pos2(row.right() - 28.0, centre_y),
                                egui::vec2(28.0, 28.0),
                            );
                            let star_response = ui.interact(
                                star,
                                ui.scope_id().with(("album_star", n)),
                                egui::Sense::click(),
                            );
                            let painter = ui.painter().clone();

                            if is_current {
                                painter.rect_filled(row, 8.0, palette.row);
                                equaliser(
                                    &painter,
                                    egui::pos2(row.left() + 14.0, centre_y + 8.0),
                                    palette.accent,
                                );
                            } else {
                                let number = track
                                    .track
                                    .map(|t| t.to_string())
                                    .unwrap_or_else(|| (n + 1).to_string());
                                painter.text(
                                    egui::pos2(row.left() + 14.0, centre_y),
                                    egui::Align2::LEFT_CENTER,
                                    number,
                                    egui::FontId::proportional(13.0),
                                    palette.weak,
                                );
                            }

                            let title_x = row.left() + 62.0;
                            let title_clip = egui::Rect::from_min_max(
                                egui::pos2(title_x, row.top()),
                                egui::pos2(row.right() - 148.0 - 8.0, row.bottom()),
                            );
                            let painter_clip = painter.with_clip_rect(title_clip);
                            let (title_font, title_colour) = if is_current {
                                (
                                    egui::FontId::new(14.0, crate::fonts::bold_family()),
                                    palette.accent,
                                )
                            } else {
                                (egui::FontId::proportional(14.0), palette.text)
                            };
                            painter_clip.text(
                                egui::pos2(title_x, centre_y - 9.0),
                                egui::Align2::LEFT_CENTER,
                                format::title(track),
                                title_font,
                                title_colour,
                            );
                            let track_artist = track
                                .artist
                                .as_deref()
                                .or(track.album_artist.as_deref())
                                .unwrap_or("");
                            painter_clip.text(
                                egui::pos2(title_x, centre_y + 9.0),
                                egui::Align2::LEFT_CENTER,
                                track_artist,
                                egui::FontId::proportional(12.0),
                                palette.weak,
                            );

                            painter.text(
                                egui::pos2(row.right() - 58.0, centre_y),
                                egui::Align2::RIGHT_CENTER,
                                format::duration(track.duration_ms),
                                egui::FontId::proportional(13.0),
                                palette.weak,
                            );
                            let star_colour = if favorite {
                                FAVORITE_COLOR
                            } else {
                                palette.weak
                            };
                            draw_star(&painter, star.center(), 16.0, star_colour, favorite);
                            if star_response.clicked() {
                                toggle_track = Some((key, !favorite));
                            }

                            if !is_current && n + 1 < count {
                                painter.line_segment(
                                    [
                                        egui::pos2(row.left(), row.bottom()),
                                        egui::pos2(row.right(), row.bottom()),
                                    ],
                                    egui::Stroke::new(1.0, palette.line),
                                );
                            }
                            // Double-click anywhere on the row to play from this track, except on the star.
                            if response.double_clicked() && !star_response.hovered() {
                                play = Some(PlayRequest {
                                    tracks: self.tracks.clone(),
                                    index: n,
                                });
                            }
                            response
                                .context_menu(|ui| queue::track_menu(ui, queue, track, playlists));
                        }
                    });
            });

        if toggle_album {
            let on = !favorite_album;
            if library
                .set_favorite(FavoriteKind::Album, &album_key, on)
                .is_ok()
            {
                if on {
                    self.favorite_albums.insert(album_key);
                } else {
                    self.favorite_albums.remove(&album_key);
                }
            }
        }
        if let Some((key, on)) = toggle_track
            && library.set_favorite(FavoriteKind::Track, &key, on).is_ok()
        {
            if on {
                self.favorite_tracks.insert(key);
            } else {
                self.favorite_tracks.remove(&key);
            }
        }
        (play, back)
    }
}

/// Draw a cover in grey, with its full-colour version faded in over it while `hovered`. The
/// fade takes 200 ms with an ease-in. The square in the middle of the cover is shown, rounded to
/// `radius`, so a radius of half the size makes a circle.
pub(crate) fn draw_cover(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    cover: &Cover,
    radius: f32,
    id: egui::Id,
    hovered: bool,
) {
    let hover = ui.ctx().animate_bool_with_time_and_easing(
        id,
        hovered,
        0.2,
        egui::emath::easing::quadratic_in,
    );
    let uv = square_uv(cover.colour.size_vec2());
    ui.put(
        rect,
        egui::Image::new(&cover.grayscaled)
            .uv(uv)
            .tint(egui::Color32::WHITE)
            .fit_to_exact_size(rect.size())
            .corner_radius(radius),
    );
    if hover > 0.0 {
        // Laying the colour over the grey at this fraction mixes the two pixel by pixel, as
        // CSS does when it changes the grayscale amount.
        let alpha = (hover * 255.0).round() as u8;
        ui.put(
            rect,
            egui::Image::new(&cover.colour)
                .uv(uv)
                .tint(egui::Color32::from_white_alpha(alpha))
                .fit_to_exact_size(rect.size())
                .corner_radius(radius),
        );
    }
}

/// The part of a texture that is its central square, in texture coordinates.
fn square_uv(size: egui::Vec2) -> egui::Rect {
    let side = size.x.min(size.y);
    let (x, y) = (
        (size.x - side) / 2.0 / size.x,
        (size.y - side) / 2.0 / size.y,
    );
    egui::Rect::from_min_max(egui::pos2(x, y), egui::pos2(1.0 - x, 1.0 - y))
}

/// Where a cover stands in the cache: being decoded, or done (`None` means there is no cover).
enum CoverSlot {
    Loading,
    Ready(Option<Cover>),
}

/// Draw a square cover, or a plain square when there is no cover.
fn cover_image(ui: &mut egui::Ui, texture: Option<&egui::TextureHandle>, size: f32) {
    match texture {
        Some(texture) => {
            ui.add(
                egui::Image::new(texture)
                    .fit_to_exact_size(egui::vec2(size, size))
                    .corner_radius(14.0),
            );
        }
        None => {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 14.0, ui.visuals().faint_bg_color);
        }
    }
}

/// A grid tile: square cover, then the album name and its composers. Text that doesn't fit is clipped.
pub(crate) fn album_tile(
    ui: &mut egui::Ui,
    cover: Option<&Cover>,
    album: &AlbumSummary,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(TILE_COVER, TILE_COVER + TILE_TEXT),
        egui::Sense::click(),
    );
    let cover_rect = egui::Rect::from_min_size(rect.min, egui::vec2(TILE_COVER, TILE_COVER));
    let painter = ui.painter_at(rect);
    match cover {
        Some(cover) => draw_cover(
            ui,
            cover_rect,
            cover,
            TILE_RADIUS,
            response.id.with("cover_hover"),
            response.hovered(),
        ),
        None => {
            painter.rect_filled(cover_rect, TILE_RADIUS, ui.visuals().faint_bg_color);
        }
    };
    if response.hovered() {
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
pub(crate) fn play_icon_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    ui.painter().rect_filled(rect, 16.0, visuals.weak_bg_fill);
    icons::paint(
        ui.painter(),
        rect.center(),
        icons::PLAY,
        16.0,
        visuals.fg_stroke.color,
    );
    response
}

/// Round button with a left chevron, for going back to the album grid.
pub(crate) fn back_icon_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    ui.painter().rect_filled(rect, 16.0, visuals.weak_bg_fill);
    icons::paint(
        ui.painter(),
        rect.center(),
        icons::CARET_LEFT,
        16.0,
        visuals.fg_stroke.color,
    );
    response
}

/// Round button with a shuffle icon (two crossing arrows). No text, so it needs no font glyph.
pub(crate) fn shuffle_icon_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    ui.painter().rect_filled(rect, 16.0, visuals.weak_bg_fill);
    icons::paint(
        ui.painter(),
        rect.center(),
        icons::SHUFFLE,
        16.0,
        visuals.fg_stroke.color,
    );
    response
}

/// The tracks in a random order (Fisher-Yates). The generator is seeded from the clock;
/// nothing here needs real randomness, so no extra crate is used.
pub(crate) fn shuffled(mut tracks: Vec<MediaItem>) -> Vec<MediaItem> {
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

/// A favourite star: the filled glyph when `filled`, the outline otherwise.
fn draw_star(
    painter: &egui::Painter,
    center: egui::Pos2,
    size: f32,
    color: egui::Color32,
    filled: bool,
) {
    if filled {
        icons::paint_filled(painter, center, icons::STAR, size, color);
    } else {
        icons::paint(painter, center, icons::STAR, size, color);
    }
}

/// Colours of the album page. Dark mode uses the design's values; light mode uses the theme's.
struct AlbumPalette {
    text: egui::Color32,
    weak: egui::Color32,
    accent: egui::Color32,
    line: egui::Color32,
    row: egui::Color32,
    fill: egui::Color32,
    on_fill: egui::Color32,
    outline: egui::Color32,
    back: egui::Color32,
}

impl AlbumPalette {
    fn of(ui: &egui::Ui) -> Self {
        let v = ui.visuals();
        if v.dark_mode {
            Self {
                text: egui::Color32::from_rgb(0xf2, 0xf2, 0xf4),
                weak: egui::Color32::from_rgb(0x9a, 0x9a, 0xa2),
                accent: egui::Color32::from_rgb(0x4d, 0xa3, 0xff),
                line: egui::Color32::from_rgb(0x2a, 0x2a, 0x2f),
                row: egui::Color32::from_rgb(0x1f, 0x1f, 0x23),
                fill: egui::Color32::from_rgb(0xf2, 0xf2, 0xf4),
                on_fill: egui::Color32::from_rgb(0x14, 0x14, 0x16),
                outline: egui::Color32::from_rgb(0x3a, 0x3a, 0x41),
                back: egui::Color32::from_rgb(0x2a, 0x2a, 0x2f),
            }
        } else {
            Self {
                text: v.text_color(),
                weak: v.weak_text_color(),
                accent: v.hyperlink_color,
                line: v.widgets.noninteractive.bg_stroke.color,
                row: v.faint_bg_color,
                fill: v.strong_text_color(),
                on_fill: v.panel_fill,
                outline: v.widgets.noninteractive.bg_stroke.color,
                back: v.widgets.inactive.weak_bg_fill,
            }
        }
    }
}

/// Icons drawn inside the album page's pill buttons.
#[derive(Clone, Copy)]
enum PillIcon {
    Play,
    Shuffle,
}

/// A pill button with an icon and a label. `filled` is the light primary style; otherwise it has
/// an outline.
fn pill(
    ui: &mut egui::Ui,
    palette: &AlbumPalette,
    label: &str,
    icon: PillIcon,
    filled: bool,
) -> egui::Response {
    let (pad, icon_w, gap) = match icon {
        PillIcon::Play => (22.0, 14.0, 8.0),
        PillIcon::Shuffle => (18.0, 16.0, 8.0),
    };
    let font = if filled {
        egui::FontId::new(14.0, crate::fonts::bold_family())
    } else {
        egui::FontId::proportional(14.0)
    };
    let colour = if filled {
        palette.on_fill
    } else {
        palette.text
    };
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font, colour);
    let width = pad + icon_w + gap + galley.size().x + pad;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 40.0), egui::Sense::click());
    let painter = ui.painter();
    if filled {
        painter.rect_filled(rect, 20.0, palette.fill);
    } else {
        painter.rect_stroke(
            rect,
            20.0,
            egui::Stroke::new(1.0, palette.outline),
            egui::StrokeKind::Inside,
        );
    }
    let c = rect.center();
    let x = rect.left() + pad;
    let glyph = match icon {
        PillIcon::Play => icons::PLAY,
        PillIcon::Shuffle => icons::SHUFFLE,
    };
    icons::paint(
        painter,
        egui::pos2(x + icon_w / 2.0, c.y),
        glyph,
        16.0,
        colour,
    );
    painter.galley(
        egui::pos2(x + icon_w + gap, c.y - galley.size().y / 2.0),
        galley,
        colour,
    );
    response
}

/// A 40 px round button with a star: filled gold when the album is a favourite.
fn star_circle(ui: &mut egui::Ui, palette: &AlbumPalette, on: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::click());
    let painter = ui.painter();
    painter.circle_stroke(rect.center(), 20.0, egui::Stroke::new(1.0, palette.outline));
    let colour = if on { FAVORITE_COLOR } else { palette.text };
    draw_star(painter, rect.center(), 18.0, colour, on);
    response
}

/// The back button of the album page: a 36 px round button with a chevron.
fn album_back_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
    let palette = AlbumPalette::of(ui);
    let painter = ui.painter();
    painter.circle_filled(rect.center(), 18.0, palette.back);
    let c = rect.center();
    icons::paint(painter, c, icons::CARET_LEFT, 18.0, palette.text);
    response
}

/// Three bars, bottom-aligned at `bottom`, marking the track that is playing.
fn equaliser(painter: &egui::Painter, bottom: egui::Pos2, colour: egui::Color32) {
    let centre = egui::pos2(bottom.x + 8.0, bottom.y - 8.0);
    icons::paint(painter, centre, icons::EQUALIZER, 16.0, colour);
}
