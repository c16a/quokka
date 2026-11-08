//! Artists: a grid of artists, and one artist's albums. Each artist is listed under the albums
//! they appear on, so an album with several artists appears under each of them.
//!
//! An artist's tile shows a photo from Wikidata when there is one, in a circle, and otherwise the
//! cover of their first album. Photos are fetched on a background thread.

use crate::i18n;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use eframe::egui;
use quokka_core::{AlbumSummary, Library};

use crate::albums::{
    AlbumsView, TILE_COVER, TILE_GAP, TILE_TEXT, album_tile, back_icon_button, draw_cover,
};
use crate::covers::{self, COVER_MAX_SIDE, Cover, Decoded, evict, upload};

#[derive(Default)]
pub struct ArtistsView {
    /// Artist name, and the positions in the album grid of the albums they appear on.
    artists: Vec<(String, Vec<usize>)>,
    /// Index into `artists` of the artist being shown.
    selected: Option<usize>,
    photos: Photos,
}

impl ArtistsView {
    /// An artist view that caches its photos in `photo_dir`.
    pub fn new(photo_dir: PathBuf) -> Self {
        Self {
            photos: Photos {
                dir: photo_dir,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Rebuild the artist list from the album grid.
    pub fn reload(&mut self, albums: &[AlbumSummary]) {
        let mut by_name: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, album) in albums.iter().enumerate() {
            for artist in &album.artists {
                by_name.entry(artist.clone()).or_default().push(i);
            }
        }
        let mut artists: Vec<_> = by_name.into_iter().collect();
        artists.sort_by_key(|(name, _)| name.to_lowercase());
        self.artists = artists;
        if self.selected.is_some_and(|i| i >= self.artists.len()) {
            self.selected = None;
        }
    }

    /// Draw the view. Returns the album (its position in the album grid) the user opened, if any.
    /// Whether one artist's page is open rather than the grid.
    pub fn showing_detail(&self) -> bool {
        self.selected.is_some()
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        albums: &mut AlbumsView,
        library: &Library,
    ) -> Option<usize> {
        if self.artists.is_empty() {
            ui.label(i18n::no_artists_yet_choose_your_library());
            return None;
        }
        match self.selected {
            None => {
                crate::page(ui, egui::Margin::symmetric(32, 32), |ui| {
                    self.grid(ui, albums, library)
                });
                None
            }
            Some(i) => self.detail(ui, albums, library, i),
        }
    }

    /// Every artist, as a tile showing their photo, or the cover of their first album.
    fn grid(&mut self, ui: &mut egui::Ui, albums: &mut AlbumsView, library: &Library) {
        crate::fonts::page_title(ui, i18n::artists());
        ui.add_space(12.0);
        let ctx = ui.ctx().clone();
        self.photos.poll(&ctx);
        let mut chosen = None;
        {
            let width = ui.available_width();
            let columns = ((width + TILE_GAP) / (TILE_COVER + TILE_GAP))
                .floor()
                .max(1.0) as usize;
            let count = self.artists.len();
            let artists = &self.artists;
            let photos = &mut self.photos;
            for row in 0..count.div_ceil(columns) {
                // A row off screen is only reserved, so the grid keeps its height and no photo is
                // requested for its artists.
                let rect = egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::vec2(width, TILE_COVER + TILE_TEXT),
                );
                if !ui.is_rect_visible(rect) {
                    ui.allocate_exact_size(rect.size(), egui::Sense::hover());
                    ui.add_space(TILE_GAP);
                    continue;
                }
                let start = row * columns;
                let end = (start + columns).min(count);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = TILE_GAP;
                    for (k, (name, album_ids)) in artists[start..end].iter().enumerate() {
                        let n = start + k;
                        let cover = match photos.get(&ctx, name) {
                            PhotoState::Photo(photo) => Some(photo),
                            PhotoState::Loading => None,
                            PhotoState::Missing => albums.cover(&ctx, library, album_ids[0]),
                        };
                        if artist_tile(ui, cover.as_ref(), name, album_ids.len()).clicked() {
                            chosen = Some(n);
                        }
                    }
                });
                ui.add_space(TILE_GAP);
            }
            let credits: Vec<&str> = self
                .artists
                .iter()
                .filter(|(name, _)| self.photos.shows(name))
                .filter_map(|(name, _)| self.photos.credits.get(name).map(String::as_str))
                .collect();
            if !credits.is_empty() {
                ui.add_space(16.0);
                ui.small(i18n::fill(
                    i18n::photos_credits(),
                    &[credits.join("  ·  ").as_str()],
                ));
            }
        }
        if chosen.is_some() {
            self.selected = chosen;
        }
    }

    /// One artist: back button, name, then the albums they appear on.
    fn detail(
        &mut self,
        ui: &mut egui::Ui,
        albums: &mut AlbumsView,
        library: &Library,
        i: usize,
    ) -> Option<usize> {
        let ctx = ui.ctx().clone();
        let (name, album_ids) = self.artists[i].clone();
        if back_icon_button(ui)
            .on_hover_text(i18n::all_artists())
            .clicked()
        {
            self.selected = None;
            return None;
        }
        ui.add_space(12.0);
        crate::fonts::page_title(ui, &name);
        ui.weak(i18n::album_count(album_ids.len()));
        ui.add_space(12.0);

        let mut opened = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(TILE_GAP, TILE_GAP);
                    for &album_index in &album_ids {
                        let cover = albums.cover(&ctx, library, album_index);
                        let album = &albums.albums()[album_index];
                        if album_tile(ui, cover.as_ref(), album).clicked() {
                            opened = Some(album_index);
                        }
                    }
                });
            });
        opened
    }
}

/// How an artist's photo stands: being fetched, there, or not on Wikidata.
#[derive(Clone)]
enum PhotoState {
    Loading,
    Photo(Cover),
    Missing,
}

/// A photo, decoded, with its credit. `None` means there is no photo.
type PhotoResult = Option<(Decoded, String)>;

/// Most artist photos kept as textures. Each is two square textures.
const PHOTO_CAP: usize = 200;

/// Artist photos, fetched from Wikidata on one background thread. The thread also decodes each
/// photo, so the UI thread only uploads it. The least recently used photos are dropped beyond
/// `PHOTO_CAP`; a dropped photo is read again from the disk cache when it is next shown.
#[derive(Default)]
struct Photos {
    /// Folder the photos are cached in.
    dir: PathBuf,
    /// Each artist's photo state, with the tick it was last used on.
    loaded: HashMap<String, (u64, PhotoState)>,
    tick: u64,
    /// Credit of each artist's photo, for the line under the grid.
    credits: HashMap<String, String>,
    requests: Option<Sender<String>>,
    results: Option<Receiver<(String, PhotoResult)>>,
}

impl Photos {
    /// Take in the photos the worker has finished since the last frame, and drop the least
    /// recently used beyond the cap.
    fn poll(&mut self, ctx: &egui::Context) {
        let Some(results) = &self.results else {
            return;
        };
        let finished: Vec<_> = results.try_iter().collect();
        for (name, result) in finished {
            let state = match result {
                Some((decoded, credit)) => {
                    self.credits.insert(name.clone(), credit);
                    PhotoState::Photo(upload(ctx, &format!("artist-photo-{name}"), decoded))
                }
                None => PhotoState::Missing,
            };
            self.loaded.insert(name, (self.tick, state));
        }
        evict(&mut self.loaded, PHOTO_CAP, |state| {
            !matches!(state, PhotoState::Loading)
        });
    }

    /// The artist's photo state. The first time an artist is seen, its photo is asked for.
    fn get(&mut self, ctx: &egui::Context, name: &str) -> PhotoState {
        self.tick += 1;
        let tick = self.tick;
        if let Some((used, state)) = self.loaded.get_mut(name) {
            *used = tick;
            return state.clone();
        }
        self.loaded
            .insert(name.to_owned(), (tick, PhotoState::Loading));
        self.ensure_worker(ctx);
        if let Some(requests) = &self.requests {
            let _ = requests.send(name.to_owned());
        }
        PhotoState::Loading
    }

    /// Whether the artist's photo is loaded, so its credit belongs under the grid.
    fn shows(&self, name: &str) -> bool {
        matches!(self.loaded.get(name), Some((_, PhotoState::Photo(_))))
    }

    /// Start the worker thread the first time a photo is wanted.
    fn ensure_worker(&mut self, ctx: &egui::Context) {
        if self.requests.is_some() {
            return;
        }
        let (request_tx, request_rx) = mpsc::channel::<String>();
        let (result_tx, result_rx) = mpsc::channel();
        let dir = self.dir.clone();
        let ctx = ctx.clone();
        thread::spawn(move || {
            // Ends when the view is dropped, which closes the request channel.
            for name in request_rx {
                let result = quokka_core::fetch_artist_photo(&name, &dir).and_then(|photo| {
                    covers::decode(&photo.bytes, COVER_MAX_SIDE).map(|d| (d, photo.credit))
                });
                if result_tx.send((name, result)).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
        });
        self.requests = Some(request_tx);
        self.results = Some(result_rx);
    }
}

/// An artist tile: a circle with the photo or cover, then the name and how many albums they have.
fn artist_tile(
    ui: &mut egui::Ui,
    cover: Option<&Cover>,
    name: &str,
    album_count: usize,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(TILE_COVER, TILE_COVER + TILE_TEXT),
        egui::Sense::click(),
    );
    let circle = egui::Rect::from_min_size(rect.min, egui::vec2(TILE_COVER, TILE_COVER));
    let painter = ui.painter_at(rect);
    match cover {
        Some(cover) => draw_cover(
            ui,
            circle,
            cover,
            TILE_COVER / 2.0,
            response.id.with("artist_hover"),
            response.hovered(),
        ),
        None => {
            painter.circle_filled(
                circle.center(),
                TILE_COVER / 2.0,
                ui.visuals().faint_bg_color,
            );
        }
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let text_top = circle.bottom() + 6.0;
    let centre_x = rect.center().x;
    painter.text(
        egui::pos2(centre_x, text_top),
        egui::Align2::CENTER_TOP,
        name,
        egui::FontId::proportional(14.0),
        ui.visuals().text_color(),
    );
    let albums = i18n::album_count(album_count);
    painter.text(
        egui::pos2(centre_x, text_top + 20.0),
        egui::Align2::CENTER_TOP,
        albums,
        egui::FontId::proportional(12.0),
        ui.visuals().weak_text_color(),
    );
    response
}
