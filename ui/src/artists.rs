//! Artists: a grid of artists, and one artist's albums. Each artist is listed under the albums
//! they appear on, so an album with several artists appears under each of them.

use std::collections::BTreeMap;

use eframe::egui;
use player_core::{AlbumSummary, Library};

use crate::albums::{AlbumsView, TILE_COVER, TILE_GAP, TILE_TEXT, album_tile, back_icon_button};

#[derive(Default)]
pub struct ArtistsView {
    /// Artist name, and the positions in the album grid of the albums they appear on.
    artists: Vec<(String, Vec<usize>)>,
    /// Index into `artists` of the artist being shown.
    selected: Option<usize>,
}

impl ArtistsView {
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

    /// Every artist, as a tile showing the cover of their first album.
    fn grid(&mut self, ui: &mut egui::Ui, albums: &mut AlbumsView, library: &Library) {
        ui.heading("Artists");
        ui.add_space(12.0);
        let ctx = ui.ctx().clone();
        let mut chosen = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(TILE_GAP, TILE_GAP);
                    for (n, (name, album_ids)) in self.artists.iter().enumerate() {
                        let cover = albums.cover(&ctx, library, album_ids[0]);
                        if artist_tile(ui, cover.as_ref(), name, album_ids.len()).clicked() {
                            chosen = Some(n);
                        }
                    }
                });
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
        ui.heading(&name);
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

/// An artist tile: square cover, then the name and how many albums they have.
fn artist_tile(
    ui: &mut egui::Ui,
    cover: Option<&egui::TextureHandle>,
    name: &str,
    album_count: usize,
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
        egui::pos2(rect.left(), text_top + 20.0),
        egui::Align2::LEFT_TOP,
        albums,
        egui::FontId::proportional(12.0),
        ui.visuals().weak_text_color(),
    );
    response
}
