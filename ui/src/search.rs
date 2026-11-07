use eframe::egui;
use quokka_core::{AlbumSummary, Library, MediaItem, PlaylistSummary, SongQuery, Station};

use crate::albums::{self, AlbumsView, PlayRequest, TILE_GAP};
use crate::format;
use crate::icons;
use crate::playback::MenuRequest;
use crate::playlists::{CARD_GAP, CARD_WIDTH, playlist_card};
use crate::queue;
use crate::radio;

/// Most songs listed for one search.
const SONG_LIMIT: usize = 200;
/// Most stations listed for one search.
const STATION_LIMIT: usize = 60;

/// What the search screen asked for this frame.
#[derive(Default)]
pub struct SearchOutcome {
    pub play: Option<PlayRequest>,
    pub open_album: Option<usize>,
    pub open_playlist: Option<i64>,
    /// A station chosen in the results: tune in to it, or play it.
    pub radio: Option<radio::RadioAction>,
}

/// The search screen: a field at the top, then the albums and songs whose names match.
#[derive(Default)]
pub struct SearchView {
    query: String,
    /// The query the results were built for. They are rebuilt only when it changes.
    searched: String,
    /// Positions in the album grid of albums whose name matches.
    album_hits: Vec<usize>,
    /// Playlists whose name matches.
    playlist_hits: Vec<PlaylistSummary>,
    /// Radio stations whose name, tags or country match.
    station_hits: Vec<Station>,
    songs: Vec<MediaItem>,
    focus_pending: bool,
}

impl SearchView {
    /// Put the cursor in the search field the next time the screen is drawn.
    pub fn focus(&mut self) {
        self.focus_pending = true;
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        albums: &mut AlbumsView,
        library: &Library,
        current: Option<i64>,
        queue: &mut Vec<MenuRequest>,
        playlists: &[PlaylistSummary],
    ) -> SearchOutcome {
        let ctx = ui.ctx().clone();
        let mut outcome = SearchOutcome::default();

        crate::fonts::page_title(ui, "Search");
        ui.add_space(8.0);
        egui::Frame::new()
            .fill(search_fill(ui))
            .corner_radius(10.0)
            .inner_margin(egui::Margin::symmetric(12, 9))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    search_icon(ui);
                    ui.add_space(6.0);
                    let field = crate::squircle::SquircleTextField::singleline(&mut self.query)
                        .hint_text("Search albums, songs, playlists and stations")
                        .desired_width(f32::INFINITY)
                        .show(ui);
                    if std::mem::take(&mut self.focus_pending) {
                        field.request_focus();
                    }
                });
            });
        ui.add_space(12.0);

        let query = self.query.trim().to_owned();
        if query != self.searched {
            self.refresh(albums, library, playlists, &query);
            self.searched = query.clone();
        }
        if query.is_empty() {
            ui.weak("Search for an album, a song, a playlist or a station. Filters: artist:, album:, genre:, year:, format:flac.");
            return outcome;
        }
        if self.station_hits.is_empty()
            && self.playlist_hits.is_empty()
            && self.album_hits.is_empty()
            && self.songs.is_empty()
        {
            ui.weak(format!("No results for \"{query}\"."));
            return outcome;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if !self.playlist_hits.is_empty() {
                    ui.heading("Playlists");
                    ui.add_space(8.0);
                    // Cards in a grid, as on the Playlists screen.
                    let columns = (((ui.available_width() + CARD_GAP) / (CARD_WIDTH + CARD_GAP))
                        .floor() as usize)
                        .max(1);
                    egui::Grid::new("search_playlist_grid")
                        .num_columns(columns)
                        .spacing([CARD_GAP, CARD_GAP])
                        .min_col_width(CARD_WIDTH)
                        .max_col_width(CARD_WIDTH)
                        .show(ui, |ui| {
                            for (n, playlist) in self.playlist_hits.iter().enumerate() {
                                if playlist_card(ui, playlist).clicked() {
                                    outcome.open_playlist = Some(playlist.id);
                                }
                                if (n + 1) % columns == 0 {
                                    ui.end_row();
                                }
                            }
                            ui.end_row();
                        });
                    ui.add_space(24.0);
                }

                if !self.station_hits.is_empty() {
                    ui.heading("Stations");
                    ui.add_space(8.0);
                    let tuned = library.tuned_stations().unwrap_or_default();
                    if let Some(action) = radio::station_cards(ui, &self.station_hits, &tuned) {
                        outcome.radio = Some(action);
                    }
                    ui.add_space(24.0);
                }

                if !self.album_hits.is_empty() {
                    ui.heading("Albums");
                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(TILE_GAP, TILE_GAP);
                        for &i in &self.album_hits {
                            let cover = albums.cover(&ctx, library, i);
                            let album = &albums.albums()[i];
                            if albums::album_tile(ui, cover.as_ref(), album).clicked() {
                                outcome.open_album = Some(i);
                            }
                        }
                    });
                    ui.add_space(24.0);
                }

                if !self.songs.is_empty() {
                    ui.heading("Songs");
                    ui.add_space(4.0);
                    for (n, song) in self.songs.iter().enumerate() {
                        // Reserved now, filled in once we know whether the row is hovered.
                        let background = ui.painter().add(egui::Shape::Noop);
                        let row = egui::Frame::new()
                            .inner_margin(egui::Margin::symmetric(8, 6))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        let title = format::title(song);
                                        if current == Some(song.id) {
                                            ui.label(crate::fonts::bold(title));
                                        } else {
                                            ui.label(title);
                                        }
                                        if let Some(artist) =
                                            song.artist.as_deref().or(song.album_artist.as_deref())
                                        {
                                            ui.weak(artist);
                                        }
                                    });
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.weak(format::duration(song.duration_ms));
                                            ui.add_space(16.0);
                                            ui.weak(song.album.as_deref().unwrap_or(""));
                                        },
                                    );
                                });
                            })
                            .response;
                        let response = ui.interact(
                            row.rect,
                            ui.scope_id().with(("song", n)),
                            egui::Sense::click(),
                        );
                        if response.hovered() {
                            ui.painter().set(
                                background,
                                egui::Shape::rect_filled(
                                    row.rect,
                                    6.0,
                                    ui.visuals().faint_bg_color,
                                ),
                            );
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if response.double_clicked() {
                            outcome.play = Some(play_from(library, song));
                        }
                        response.context_menu(|ui| queue::track_menu(ui, queue, song, playlists));
                    }
                }
            });
        outcome
    }

    /// Rebuild the results for `query`, which may contain filters such as `artist:`.
    fn refresh(
        &mut self,
        albums: &AlbumsView,
        library: &Library,
        playlists: &[PlaylistSummary],
        query: &str,
    ) {
        self.album_hits.clear();
        self.playlist_hits.clear();
        self.station_hits.clear();
        self.songs.clear();
        let parsed = SongQuery::parse(query);
        if parsed.is_empty() {
            return;
        }
        // Stations and playlists have a name, so they match the words typed and no other filter.
        if !parsed.text.is_empty()
            && parsed.artist.is_none()
            && parsed.album.is_none()
            && parsed.genre.is_none()
            && parsed.year.is_none()
            && parsed.format.is_none()
        {
            self.station_hits = library
                .search_stations(query.trim(), STATION_LIMIT)
                .unwrap_or_default();
        }
        if !parsed.text.is_empty()
            && parsed.artist.is_none()
            && parsed.album.is_none()
            && parsed.genre.is_none()
            && parsed.year.is_none()
            && parsed.format.is_none()
        {
            let needle = parsed.text.to_lowercase();
            self.playlist_hits = playlists
                .iter()
                .filter(|playlist| playlist.name.to_lowercase().contains(&needle))
                .cloned()
                .collect();
        }
        // Genre and format describe tracks, not albums, so they only filter songs.
        if !parsed.has_track_filters() {
            self.album_hits = albums
                .albums()
                .iter()
                .enumerate()
                .filter(|(_, album)| album_matches(album, &parsed))
                .map(|(i, _)| i)
                .collect();
        }
        self.songs = library
            .search_songs(&parsed, SONG_LIMIT)
            .unwrap_or_default();
    }
}

/// Whether an album passes every part of a search that is given.
fn album_matches(album: &AlbumSummary, query: &SongQuery) -> bool {
    let contains =
        |haystack: &str, needle: &str| haystack.to_lowercase().contains(&needle.to_lowercase());
    let name = album.album.as_deref().unwrap_or("");
    if !query.text.is_empty() && !contains(name, &query.text) {
        return false;
    }
    if let Some(album_name) = &query.album
        && !contains(name, album_name)
    {
        return false;
    }
    if let Some(artist) = &query.artist
        && !album.artists.iter().any(|a| contains(a, artist))
    {
        return false;
    }
    if let Some(year) = query.year
        && album.year != Some(year)
    {
        return false;
    }
    true
}

/// Play `song` from its album, so the rest of the album follows it.
fn play_from(library: &Library, song: &MediaItem) -> PlayRequest {
    let tracks = library
        .album_tracks(song.album.as_deref())
        .unwrap_or_default();
    match tracks.iter().position(|track| track.id == song.id) {
        Some(index) => PlayRequest { tracks, index },
        None => PlayRequest {
            tracks: vec![song.clone()],
            index: 0,
        },
    }
}

/// Background of the search field: a faint tint that works in both themes.
pub(crate) fn search_fill(ui: &egui::Ui) -> egui::Color32 {
    if ui.visuals().dark_mode {
        egui::Color32::from_white_alpha(18)
    } else {
        egui::Color32::from_black_alpha(12)
    }
}

/// A magnifying glass.
fn search_icon(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
    icons::paint(
        ui.painter(),
        rect.center(),
        icons::MAGNIFYING_GLASS,
        16.0,
        ui.visuals().weak_text_color(),
    );
}
