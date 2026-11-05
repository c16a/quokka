//! Artists: a grid of artists, and one artist's albums. Each artist is listed under the albums
//! they appear on, so an album with several artists appears under each of them.
//!
//! An artist's tile shows a photo from Wikidata when there is one, in a circle, and otherwise the
//! cover of their first album. Photos are fetched on a background thread.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use eframe::egui;
use player_core::{AlbumSummary, ArtistPhoto, Library};

use crate::albums::{
    AlbumsView, Cover, TILE_COVER, TILE_GAP, TILE_TEXT, album_tile, back_icon_button,
    cover_from_bytes, draw_cover,
};

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
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        albums: &mut AlbumsView,
        library: &Library,
    ) -> Option<usize> {
        if self.artists.is_empty() {
            ui.label("No artists yet. Choose your library folder in Settings.");
            return None;
        }
        match self.selected {
            None => {
                self.grid(ui, albums, library);
                None
            }
            Some(i) => self.detail(ui, albums, library, i),
        }
    }

    /// Every artist, as a tile showing their photo, or the cover of their first album.
    fn grid(&mut self, ui: &mut egui::Ui, albums: &mut AlbumsView, library: &Library) {
        crate::fonts::page_title(ui, "Artists");
        ui.add_space(12.0);
        let ctx = ui.ctx().clone();
        self.photos.poll(&ctx);
        let mut chosen = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(TILE_GAP, TILE_GAP);
                    for (n, (name, album_ids)) in self.artists.iter().enumerate() {
                        let cover = match self.photos.get(&ctx, name) {
                            Some(photo) => Some(photo),
                            None => albums.cover(&ctx, library, album_ids[0]),
                        };
                        if artist_tile(ui, cover.as_ref(), name, album_ids.len()).clicked() {
                            chosen = Some(n);
                        }
                    }
                });
                let credits: Vec<&str> = self
                    .artists
                    .iter()
                    .filter_map(|(name, _)| self.photos.credits.get(name).map(String::as_str))
                    .collect();
                if !credits.is_empty() {
                    ui.add_space(16.0);
                    ui.small(format!("Photos: {}", credits.join("  ·  ")));
                }
            });
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
        if back_icon_button(ui).on_hover_text("All artists").clicked() {
            self.selected = None;
            return None;
        }
        ui.add_space(12.0);
        crate::fonts::page_title(ui, &name);
        ui.weak(format!("{} albums", album_ids.len()));
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

/// Artist photos, fetched from Wikidata on one background thread. Each photo is decoded once and
/// kept. An artist with no photo is kept as `None`, so it is not asked for again.
#[derive(Default)]
struct Photos {
    /// Folder the photos are cached in.
    dir: PathBuf,
    loaded: HashMap<String, Option<Cover>>,
    /// Artists that have been asked for and not yet answered.
    pending: HashSet<String>,
    /// Credit of each artist's photo, for the line under the grid.
    credits: HashMap<String, String>,
    requests: Option<Sender<String>>,
    results: Option<Receiver<(String, Option<ArtistPhoto>)>>,
}

impl Photos {
    /// Take in the photos the worker has finished since the last frame.
    fn poll(&mut self, ctx: &egui::Context) {
        let Some(results) = &self.results else {
            return;
        };
        let finished: Vec<_> = results.try_iter().collect();
        for (name, photo) in finished {
            self.pending.remove(&name);
            let cover = photo.as_ref().and_then(|photo| {
                cover_from_bytes(ctx, &format!("artist-photo-{name}"), &photo.bytes)
            });
            if cover.is_some()
                && let Some(photo) = &photo
            {
                self.credits.insert(name.clone(), photo.credit.clone());
            }
            self.loaded.insert(name, cover);
        }
    }

    /// The artist's photo, if it has loaded. The first time an artist is seen, its photo is asked
    /// for, and it appears on a later frame.
    fn get(&mut self, ctx: &egui::Context, name: &str) -> Option<Cover> {
        if let Some(entry) = self.loaded.get(name) {
            return entry.clone();
        }
        if self.pending.insert(name.to_owned()) {
            self.ensure_worker(ctx);
            if let Some(requests) = &self.requests {
                let _ = requests.send(name.to_owned());
            }
        }
        None
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
                let bytes = player_core::fetch_artist_photo(&name, &dir);
                if result_tx.send((name, bytes)).is_err() {
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
    let albums = if album_count == 1 {
        "1 album".to_owned()
    } else {
        format!("{album_count} albums")
    };
    painter.text(
        egui::pos2(centre_x, text_top + 20.0),
        egui::Align2::CENTER_TOP,
        albums,
        egui::FontId::proportional(12.0),
        ui.visuals().weak_text_color(),
    );
    response
}
