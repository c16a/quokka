//! Recently added and recently played albums, each as a row of album tiles.

use eframe::egui;
use player_core::{AlbumSummary, Library, Recent};

use crate::albums::{AlbumsView, TILE_GAP, album_tile};

/// Most albums listed in each row.
const LIMIT: usize = 24;

#[derive(Default)]
pub struct RecentView {
    /// Positions in the album grid of the albums added most recently.
    added: Vec<usize>,
    /// Positions in the album grid of the albums played most recently.
    played: Vec<usize>,
}

impl RecentView {
    /// Rebuild both lists from the library. Albums hidden by the grid's filters are left out.
    pub fn reload(&mut self, library: &Library, albums: &[AlbumSummary]) {
        self.added = positions(library, Recent::Added, albums);
        self.played = positions(library, Recent::Played, albums);
    }

    /// Draw both rows. Returns the album (its position in the grid) the user opened, if any.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        albums: &mut AlbumsView,
        library: &Library,
    ) -> Option<usize> {
        let ctx = ui.ctx().clone();
        let mut opened = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let rows = [
                    (
                        "Recently added",
                        &self.added,
                        "Albums you add to the library appear here.",
                    ),
                    (
                        "Recently played",
                        &self.played,
                        "Albums you play appear here.",
                    ),
                ];
                for (title, list, empty) in rows {
                    ui.heading(title);
                    ui.add_space(8.0);
                    if list.is_empty() {
                        ui.weak(empty);
                    } else {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(TILE_GAP, TILE_GAP);
                            for &i in list {
                                let cover = albums.cover(&ctx, library, i);
                                let album = &albums.albums()[i];
                                if album_tile(ui, cover.as_ref(), album).clicked() {
                                    opened = Some(i);
                                }
                            }
                        });
                    }
                    ui.add_space(24.0);
                }
            });
        opened
    }
}

/// Positions in the album grid of the albums in a recent list, most recent first.
fn positions(library: &Library, recent: Recent, albums: &[AlbumSummary]) -> Vec<usize> {
    library
        .recent_albums(recent, LIMIT)
        .unwrap_or_default()
        .iter()
        .filter_map(|name| {
            albums
                .iter()
                .position(|album| album.album.as_deref().unwrap_or("") == name)
        })
        .collect()
}
