use std::collections::{HashMap, HashSet};

use eframe::egui;
use quokka_core::{
    AlbumFilter, AlbumSort, AlbumSummary, FavoriteKind, Library, MediaItem, PlaylistSummary,
};

use crate::covers::{self, COVER_MAX_SIDE, Cover, Worker, evict, upload};
use crate::format;
use crate::icons;
use crate::playback::MenuRequest;
use crate::queue;
use crate::squircle::{Padding, SquircleButton};

/// Height of a track row's content. Vertical padding comes from the row's frame.
const TRACK_ROW_HEIGHT: f32 = 52.0;
/// Gap between a track's title line and its artist line.
const TITLE_GAP: f32 = 2.0;

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
/// The album cover on a narrow page, and the width below which the page is laid out as a phone column.
const COVER_PHONE: f32 = 220.0;
const NARROW_DETAIL: f32 = 560.0;
/// Codecs that are lossless. Their details show the codec alone, with no bitrate.
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
    /// Whether an album's detail page is open rather than the grid.
    pub fn showing_detail(&self) -> bool {
        self.selected.is_some()
    }

    /// Open the album called `name`. If the grid's filters hide it, they are cleared first.
    /// Returns false when no album has that name.
    pub fn open_by_name(&mut self, library: &Library, name: &str) -> bool {
        let find = |albums: &[AlbumSummary]| {
            albums
                .iter()
                .position(|album| album.album.as_deref() == Some(name))
        };
        let index = match find(&self.albums) {
            Some(index) => Some(index),
            None if self.filter != AlbumFilter::default() => {
                self.filter = AlbumFilter::default();
                self.reload(library);
                find(&self.albums)
            }
            None => None,
        };
        match index {
            Some(index) => {
                self.open(library, index);
                true
            }
            None => false,
        }
    }

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
        let ctx = ui.ctx().clone();
        let mut play = None;
        let mut back = false;
        let mut open = None;
        match self.selected {
            None => {
                open = crate::page(ui, egui::Margin::symmetric(0, 0), |ui| {
                    crate::fonts::page_title(ui, "Albums");
                    self.toolbar(ui, library);
                    if self.albums.is_empty() {
                        if self.filter_active() {
                            ui.label("No albums match these filters.");
                        } else {
                            ui.label("No albums yet. Choose your library folder in Settings.");
                        }
                        return None;
                    }
                    self.grid(ui, &ctx, library)
                });
            }
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

    /// The album grid, as rows of tiles. It sits inside the page's scroll area. Returns the album the user
    /// clicked, if any.
    fn grid(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, library: &Library) -> Option<usize> {
        let columns = ((ui.available_width() + TILE_GAP) / (TILE_COVER + TILE_GAP))
            .floor()
            .max(1.0) as usize;
        let count = self.albums.len();
        let mut opened = None;
        for row in 0..count.div_ceil(columns) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = TILE_GAP;
                for column in 0..columns {
                    let i = row * columns + column;
                    if i >= count {
                        break;
                    }
                    let cover = self.cover_for(ctx, library, i);
                    let album = &self.albums[i];
                    if album_tile(ui, cover.as_ref(), album).clicked() {
                        opened = Some(i);
                    }
                }
            });
            ui.add_space(TILE_GAP);
        }
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

        // The page sits 32 px from the sidebar and from the top. The whole page scrolls, the header and
        // the tracks together.
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(32, 32))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if album_back_button(ui).on_hover_text("All albums").clicked() {
                                back = true;
                            }
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
                        // The genres of the album's tracks, each once, in the order they first appear.
                        let mut genres: Vec<&str> = Vec::new();
                        for genre in self.tracks.iter().filter_map(|t| t.genre.as_deref()) {
                            if !genres.contains(&genre) {
                                genres.push(genre);
                            }
                        }
                        if !genres.is_empty() {
                            meta.push(genres.join(", "));
                        }
                        if total_ms > 0 {
                            meta.push(format::long_duration(total_ms));
                        }
                        if let Some(codec) = codec {
                            let lossless = LOSSLESS.iter().any(|c| codec.eq_ignore_ascii_case(c));
                            let kbps = first.and_then(|t| t.bitrate_kbps);
                            meta.push(match kbps {
                                _ if lossless => codec,
                                Some(kbps) => format!("{codec} {kbps} kbps"),
                                None => codec,
                            });
                        }
                        let meta = meta.join(" · ");

                        let narrow = ui.available_width() < NARROW_DETAIL;
                        // The actions: Play, Shuffle and the favourite star.
                        let mut actions = |ui: &mut egui::Ui| {
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
                        };
                        if narrow {
                            // A narrow window: one centred column, with the cover, then the title and actions.
                            ui.vertical_centered(|ui| {
                                cover_image(ui, cover.as_ref(), COVER_PHONE);
                                ui.add_space(22.0);
                                ui.label(
                                    egui::RichText::new("ALBUM").size(11.0).color(palette.weak),
                                );
                                ui.label(
                                    egui::RichText::new(title)
                                        .size(24.0)
                                        .family(crate::fonts::bold_family())
                                        .color(palette.text),
                                );
                                ui.label(
                                    egui::RichText::new(artist).size(15.0).color(palette.text),
                                );
                                ui.label(egui::RichText::new(meta).size(13.0).color(palette.weak));
                                ui.add_space(16.0);
                                actions(ui);
                            });
                        } else {
                            ui.horizontal(|ui| {
                                cover_image(ui, cover.as_ref(), COVER_DETAIL);
                                ui.add_space(28.0);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(ui.available_width(), COVER_DETAIL),
                                    egui::Layout::bottom_up(egui::Align::Min),
                                    |ui| {
                                        // Bottom-up: the actions come first, so they sit lowest.
                                        ui.spacing_mut().item_spacing.y = 10.0;
                                        actions(ui);
                                        ui.label(
                                            egui::RichText::new(meta)
                                                .size(13.0)
                                                .color(palette.weak),
                                        );
                                        ui.label(
                                            egui::RichText::new(artist)
                                                .size(16.0)
                                                .color(palette.text),
                                        );
                                        ui.label(
                                            egui::RichText::new(title)
                                                .size(36.0)
                                                .family(crate::fonts::bold_family())
                                                .color(palette.text),
                                        );
                                        ui.label(
                                            egui::RichText::new("ALBUM")
                                                .size(11.0)
                                                .color(palette.weak),
                                        );
                                    },
                                );
                            });
                        }

                        // The tracks, one row each.
                        ui.add_space(26.0);
                        // The room at the right of a row for the duration and the star. A narrow page needs less.
                        let track_reserve = if narrow { 110.0 } else { 148.0 };
                        let count = self.tracks.len();
                        for n in 0..count {
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
                                egui::pos2(
                                    row.right() - if narrow { 22.0 } else { 28.0 },
                                    centre_y,
                                ),
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
                                    egui::pos2(
                                        row.left() + if narrow { 20.0 } else { 22.0 },
                                        centre_y,
                                    ),
                                    palette.accent,
                                );
                            } else {
                                let number = track
                                    .track
                                    .map(|t| t.to_string())
                                    .unwrap_or_else(|| (n + 1).to_string());
                                painter.text(
                                    egui::pos2(
                                        row.left() + if narrow { 16.0 } else { 14.0 },
                                        centre_y,
                                    ),
                                    egui::Align2::LEFT_CENTER,
                                    number,
                                    egui::FontId::proportional(13.0),
                                    palette.weak,
                                );
                            }

                            let title_x = row.left() + if narrow { 44.0 } else { 62.0 };
                            let title_clip = egui::Rect::from_min_max(
                                egui::pos2(title_x, row.top()),
                                egui::pos2(row.right() - track_reserve - 8.0, row.bottom()),
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
                            // The title and artist are one block, centred on the row.
                            let artist_font = egui::FontId::proportional(12.0);
                            let (title_h, artist_h) = ui.fonts_mut(|f| {
                                (f.row_height(&title_font), f.row_height(&artist_font))
                            });
                            let block_top = centre_y - (title_h + TITLE_GAP + artist_h) / 2.0;
                            painter_clip.text(
                                egui::pos2(title_x, block_top),
                                egui::Align2::LEFT_TOP,
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
                                egui::pos2(title_x, block_top + title_h + TITLE_GAP),
                                egui::Align2::LEFT_TOP,
                                track_artist,
                                artist_font,
                                palette.weak,
                            );

                            painter.text(
                                egui::pos2(
                                    row.right() - if narrow { 52.0 } else { 58.0 },
                                    centre_y,
                                ),
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

/// Largest tilt of a cover, in radians: 8 degrees, reached when the pointer is at the cover's edge.
const TILT_MAX: f32 = 0.1396;
/// How strong the highlight that follows the pointer is, from 0 to 1.
const SHINE_STRENGTH: f32 = 0.24;

/// The point `p` of a flat cover, tilted by `tilt` (rotation about the vertical and the horizontal
/// axis) and seen from `dist` away. The result is relative to the cover's centre.
fn project(p: egui::Vec2, tilt: egui::Vec2, dist: f32) -> egui::Vec2 {
    let (sa, ca) = tilt.x.sin_cos();
    let (sb, cb) = tilt.y.sin_cos();
    // Tilt about the horizontal axis, then about the vertical one. The flat cover starts at z = 0.
    let y1 = p.y * cb;
    let z1 = p.y * sb;
    let x2 = p.x * ca + z1 * sa;
    let z2 = -p.x * sa + z1 * ca;
    let scale = dist / (dist - z2);
    egui::vec2(x2 * scale, y1 * scale)
}

/// The outline of a rounded rectangle with half-size `half` and corner `radius`, as `per_corner`
/// points on each corner, going round once.
fn rounded_outline(half: egui::Vec2, radius: f32, per_corner: usize) -> Vec<egui::Vec2> {
    let r = radius.min(half.x).min(half.y);
    let corners: [(f32, f32, f32); 4] = [
        (half.x - r, half.y - r, 0.0),
        (-half.x + r, half.y - r, 90.0),
        (-half.x + r, -half.y + r, 180.0),
        (half.x - r, -half.y + r, 270.0),
    ];
    let mut points = Vec::with_capacity(4 * per_corner);
    for (cx, cy, start) in corners {
        for k in 0..per_corner {
            let a = (start + 90.0 * k as f32 / per_corner as f32).to_radians();
            points.push(egui::vec2(cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    points
}

/// Paint a cover tilted toward the pointer, with a highlight that follows the pointer. The tilt and the
/// highlight ease in on hover and back out when the pointer leaves.
fn tilted_cover(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: egui::Rect,
    texture: &egui::TextureHandle,
    response: &egui::Response,
) {
    let ctx = ui.ctx();
    let hovered = response.hovered();
    let hover = ctx.animate_bool_with_time_and_easing(
        response.id.with("cover_hover"),
        hovered,
        0.2,
        egui::emath::easing::quadratic_in,
    );

    // The tilt aims at the pointer, measured from the cover's centre, and eases toward it each frame.
    let target = match response.hover_pos() {
        Some(pos) if hovered => {
            let local = (pos - rect.center()) / (rect.size() / 2.0);
            let (x, y) = (local.x.clamp(-1.0, 1.0), local.y.clamp(-1.0, 1.0));
            egui::vec2(-x * TILT_MAX, y * TILT_MAX)
        }
        _ => egui::Vec2::ZERO,
    };
    let dt = ui.input(|i| i.stable_dt).min(0.1);
    let ease = 1.0 - (-dt * 14.0).exp();
    let tilt_id = response.id.with("cover_tilt");
    let tilt = ctx.data_mut(|data| {
        let tilt: &mut egui::Vec2 = data.get_temp_mut_or_default(tilt_id);
        *tilt += (target - *tilt) * ease;
        *tilt
    });
    if hovered || hover > 0.0 || tilt.length() > 1e-4 {
        ctx.request_repaint();
    }

    let half = rect.size() / 2.0;
    let dist = rect.width() * 4.0;
    let centre = rect.center();
    let uv_rect = square_uv(texture.size_vec2());
    let outline = rounded_outline(half, TILE_RADIUS, 16);
    let screen: Vec<egui::Pos2> = outline
        .iter()
        .map(|&p| centre + project(p, tilt, dist))
        .collect();
    let uv_of = |p: egui::Vec2| {
        let t = p / (half * 2.0) + egui::vec2(0.5, 0.5);
        uv_rect.min + t * uv_rect.size()
    };

    // A fan of triangles from the centre to the outline, so the tilted shape keeps its rounded corners.
    let fan = |texture: egui::TextureId, colour_at: &dyn Fn(egui::Pos2) -> egui::Color32| {
        let mut mesh = egui::Mesh::with_texture(texture);
        mesh.vertices.push(egui::epaint::Vertex {
            pos: centre,
            uv: uv_rect.center(),
            color: colour_at(centre),
        });
        for (local, pos) in outline.iter().zip(&screen) {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: *pos,
                uv: uv_of(*local),
                color: colour_at(*pos),
            });
        }
        let n = outline.len() as u32;
        for i in 0..n {
            mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
        }
        egui::Shape::mesh(mesh)
    };

    painter.add(fan(texture.id(), &|_| egui::Color32::WHITE));

    // The highlight: white, brightest at the pointer and fading out across the cover.
    if hover > 0.0 {
        let pointer = response.hover_pos().unwrap_or(centre);
        let reach = rect.width() * 0.8;
        painter.add(fan(egui::TextureId::default(), &|pos| {
            let fade = (1.0 - (pos - pointer).length() / reach).clamp(0.0, 1.0);
            let alpha = fade * fade * SHINE_STRENGTH * hover;
            egui::Color32::from_white_alpha((alpha * 255.0).round() as u8)
        }));
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
    fade_cover(ui, cover, id, hovered, |ui, texture, uv, tint| {
        ui.put(
            rect,
            egui::Image::new(texture)
                .uv(uv)
                .tint(tint)
                .fit_to_exact_size(rect.size())
                .corner_radius(radius),
        );
    });
}

/// Draw an album cover in grey, fading to its full colour on hover, shaped as a squircle with
/// continuous corners of `radius`. The fade is the same as `draw_cover`'s.
fn draw_squircle_cover(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    cover: &Cover,
    radius: f32,
    id: egui::Id,
    hovered: bool,
) {
    fade_cover(ui, cover, id, hovered, |ui, texture, uv, tint| {
        paint_squircle(ui.painter(), rect, texture.id(), uv, tint, radius);
    });
}

/// The hover fade shared by the cover shapes. `draw` paints one layer: the grey cover at full
/// strength, then the colour cover over it at the hover fraction.
fn fade_cover(
    ui: &mut egui::Ui,
    cover: &Cover,
    id: egui::Id,
    hovered: bool,
    mut draw: impl FnMut(&mut egui::Ui, &egui::TextureHandle, egui::Rect, egui::Color32),
) {
    let hover = ui.ctx().animate_bool_with_time_and_easing(
        id,
        hovered,
        0.2,
        egui::emath::easing::quadratic_in,
    );
    let uv = square_uv(cover.colour.size_vec2());
    draw(ui, &cover.grayscaled, uv, egui::Color32::WHITE);
    if hover > 0.0 {
        // Laying the colour over the grey at this fraction mixes the two pixel by pixel, as
        // CSS does when it changes the grayscale amount.
        let alpha = (hover * 255.0).round() as u8;
        draw(
            ui,
            &cover.colour,
            uv,
            egui::Color32::from_white_alpha(alpha),
        );
    }
}

/// Paint `texture` (or a flat `tint` when it is the default white texture) over `rect` in the shape of
/// a squircle with corners of `radius`. `uv` is the part of the texture that covers the rectangle.
fn paint_squircle(
    painter: &egui::Painter,
    rect: egui::Rect,
    texture: egui::TextureId,
    uv: egui::Rect,
    tint: egui::Color32,
    radius: f32,
) {
    let centre = rect.center();
    let half = rect.size() / 2.0;
    let outline = crate::squircle::outline(half.x, half.y, radius);
    let mut mesh = egui::Mesh::with_texture(texture);
    mesh.vertices.push(egui::epaint::Vertex {
        pos: centre,
        uv: uv.center(),
        color: tint,
    });
    for offset in &outline {
        let t = *offset / (half * 2.0) + egui::vec2(0.5, 0.5);
        mesh.vertices.push(egui::epaint::Vertex {
            pos: centre + *offset,
            uv: uv.min + t * uv.size(),
            color: tint,
        });
    }
    // A fan of triangles from the centre, as the tilted covers use. egui's convex-polygon fill
    // draws a wedge across the page for these outlines.
    let n = outline.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    painter.add(egui::Shape::mesh(mesh));
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

/// Draw a square cover with a soft shadow beneath it, or a plain square when there is no cover. The
/// cover tilts toward the pointer. The shadow is darker in light mode, and stronger in dark mode so it
/// shows against the dark background.
fn cover_image(ui: &mut egui::Ui, texture: Option<&egui::TextureHandle>, size: f32) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    // A wide, faint shadow that sits a little below the cover, so it looks lifted off the page.
    let shadow = if ui.visuals().dark_mode {
        egui::epaint::Shadow {
            offset: [0, 14],
            blur: 44,
            spread: 0,
            color: egui::Color32::from_black_alpha(130),
        }
    } else {
        egui::epaint::Shadow {
            offset: [0, 12],
            blur: 40,
            spread: 0,
            color: egui::Color32::from_black_alpha(40),
        }
    };
    ui.painter().add(shadow.as_shape(rect, 14.0));
    match texture {
        Some(texture) => tilted_cover(ui, ui.painter(), rect, texture, &response),
        None => {
            ui.painter()
                .rect_filled(rect, 14.0, ui.visuals().faint_bg_color);
        }
    }
}

/// Gap between a scrolling line and its repeat, in points.
const SCROLL_GAP: f32 = 40.0;
/// How fast a line that does not fit the tile scrolls, in points per second.
const SCROLL_SPEED: f32 = 30.0;

/// Paint one line of a tile at `top`, in the tile whose area is `rect`. A line wider than the tile
/// scrolls left in a loop, by `offset` points. Otherwise it starts at the tile's left edge, and the
/// tile's painter clips what does not fit.
pub(crate) fn tile_line(
    painter: &egui::Painter,
    rect: egui::Rect,
    top: f32,
    text: String,
    (font, colour): (egui::FontId, egui::Color32),
    offset: f32,
) {
    let galley = painter.layout_no_wrap(text, font, colour);
    let width = galley.size().x;
    let mut x = rect.left();
    if width > rect.width() {
        let cycle = width + SCROLL_GAP;
        x -= offset.rem_euclid(cycle);
        // The repeat sits one cycle behind, so the line loops without a gap.
        painter.galley(egui::pos2(x + cycle, top), galley.clone(), colour);
    }
    painter.galley(egui::pos2(x, top), galley, colour);
}

/// The horizontal scroll of a tile's long line, in points. It moves on while the tile is hovered, and
/// slides back to the start when the pointer leaves. It is kept per tile, under `id`.
pub(crate) fn hover_scroll(ui: &egui::Ui, id: egui::Id, hovered: bool) -> f32 {
    let step = ui.input(|i| i.stable_dt).min(0.1) * SCROLL_SPEED;
    let offset = ui.ctx().data_mut(|data| {
        let offset: &mut f32 = data.get_temp_mut_or_default(id);
        *offset = if hovered {
            *offset + step
        } else {
            (*offset - step).max(0.0)
        };
        *offset
    });
    if hovered || offset > 0.0 {
        ui.ctx().request_repaint();
    }
    offset
}

/// A grid tile: square cover, then the album name and its composers. Text that doesn't fit is clipped,
/// unless the tile is hovered, when it scrolls.
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
        Some(cover) => draw_squircle_cover(
            ui,
            cover_rect,
            cover,
            TILE_RADIUS,
            response.id.with("cover_hover"),
            response.hovered(),
        ),
        None => {
            paint_squircle(
                &painter,
                cover_rect,
                egui::TextureId::default(),
                egui::Rect::ZERO,
                ui.visuals().faint_bg_color,
                TILE_RADIUS,
            );
        }
    };
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let text_top = cover_rect.bottom() + 6.0;
    let offset = hover_scroll(ui, response.id.with("scroll_offset"), response.hovered());
    let name_colour = ui.visuals().text_color();
    let composer_colour = ui.visuals().weak_text_color();
    tile_line(
        &painter,
        rect,
        text_top,
        album.album.as_deref().unwrap_or("Unknown album").to_owned(),
        (egui::FontId::proportional(14.0), name_colour),
        offset,
    );
    if !album.composers.is_empty() {
        tile_line(
            &painter,
            rect,
            text_top + 20.0,
            album.composers.join(", "),
            (egui::FontId::proportional(12.0), composer_colour),
            offset,
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

/// The background of a back button: its resting colour, which eases over 0.2 seconds into a more
/// prominent one while the button is hovered. The prominent colour is lighter in dark mode and darker
/// in light mode.
fn back_background(ui: &egui::Ui, response: &egui::Response, rest: egui::Color32) -> egui::Color32 {
    let target = if ui.visuals().dark_mode {
        rest.lerp_to_gamma(egui::Color32::WHITE, 0.14)
    } else {
        rest.lerp_to_gamma(egui::Color32::BLACK, 0.12)
    };
    let hover = ui.ctx().animate_bool_with_time_and_easing(
        response.id.with("back_hover"),
        response.hovered(),
        0.2,
        egui::emath::easing::quadratic_in,
    );
    rest.lerp_to_gamma(target, hover)
}

/// Round button with a left chevron, for going back to the album grid.
pub(crate) fn back_icon_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    let fill = back_background(ui, &response, ui.visuals().widgets.inactive.weak_bg_fill);
    ui.painter().rect_filled(rect, 16.0, fill);
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
    let padding = match icon {
        PillIcon::Play => Padding::symmetric(22.0, 10.0),
        PillIcon::Shuffle => Padding::symmetric(18.0, 10.0),
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
    let glyph = match icon {
        PillIcon::Play => icons::PLAY,
        PillIcon::Shuffle => icons::SHUFFLE,
    };
    let text = label.to_owned();
    let content = move |ui: &mut egui::Ui| {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
            icons::paint(ui.painter(), rect.center(), glyph, 16.0, colour);
            ui.add_space(8.0);
            ui.add(egui::Label::new(egui::RichText::new(text).font(font).color(colour)).extend())
        })
        .response
    };
    let button = SquircleButton::new(content).padding(padding);
    if filled {
        button.fill(palette.fill).show(ui)
    } else {
        button
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::new(1.0, palette.outline))
            .show(ui)
    }
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
    let fill = back_background(ui, &response, palette.back);
    let painter = ui.painter();
    painter.circle_filled(rect.center(), 18.0, fill);
    let c = rect.center();
    icons::paint(painter, c, icons::CARET_LEFT, 18.0, palette.text);
    response
}

/// The equaliser icon, centred on `centre`, marking the track that is playing.
fn equaliser(painter: &egui::Painter, centre: egui::Pos2, colour: egui::Color32) {
    icons::paint(painter, centre, icons::EQUALIZER, 16.0, colour);
}
