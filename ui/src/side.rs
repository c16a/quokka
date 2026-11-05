//! The lyrics and track info panels, beside the main view. Both read the playing file's tags
//! once per track.

use std::path::Path;
use std::process::Command;

use eframe::egui;
use player_core::{MediaItem, TrackDetails, track_details};

use crate::format;

#[derive(Default)]
pub struct SidePanels {
    /// The track the cached details belong to, and the details read from its file.
    track: Option<i64>,
    details: Option<TrackDetails>,
}

impl SidePanels {
    /// The details of `item`, read from its file the first time they are asked for.
    fn details(&mut self, item: &MediaItem) -> Option<&TrackDetails> {
        if self.track != Some(item.id) {
            self.track = Some(item.id);
            self.details = track_details(&item.path);
        }
        self.details.as_ref()
    }

    /// The lyrics of the playing track, as the file's tags give them.
    pub fn lyrics(&mut self, ui: &mut egui::Ui, item: Option<&MediaItem>) {
        heading(ui, "Lyrics");
        let Some(item) = item else {
            indented(ui, |ui| {
                ui.weak("Nothing playing.");
            });
            return;
        };
        let lyrics = self.details(item).and_then(|d| d.lyrics.clone());
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match lyrics {
                Some(text) => indented(ui, |ui| {
                    ui.label(text);
                }),
                None => indented(ui, |ui| {
                    ui.weak("This file has no lyrics in its tags.");
                }),
            });
    }

    /// Everything the file says about itself, and a button to show it in the file manager.
    pub fn info(&mut self, ui: &mut egui::Ui, item: Option<&MediaItem>) {
        heading(ui, "Track info");
        let Some(item) = item else {
            indented(ui, |ui| {
                ui.weak("Nothing playing.");
            });
            return;
        };
        let details = self.details(item);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                indented(ui, |ui| {
                    egui::Grid::new("track_info")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            row(ui, "Title", &format::title(item));
                            row(
                                ui,
                                "Artist",
                                item.artist
                                    .as_deref()
                                    .or(item.album_artist.as_deref())
                                    .unwrap_or("—"),
                            );
                            row(ui, "Album", item.album.as_deref().unwrap_or("—"));
                            row(ui, "Codec", item.codec.as_deref().unwrap_or("—"));
                            row(ui, "Duration", &format::duration(item.duration_ms));
                            row(ui, "Size", &format::size(item.size));
                            if let Some(kbps) = item.bitrate_kbps {
                                row(ui, "Bitrate", &format::kbps(kbps));
                            }
                            if let Some(sample) =
                                format::sample_format(item.bit_depth, item.sample_rate)
                            {
                                row(ui, "Format", &sample);
                            }
                            if let Some(channels) = item.channels {
                                row(ui, "Channels", &format::channels(channels));
                            }
                            if let Some(details) = details {
                                row(ui, "Pictures", &details.pictures.to_string());
                            }
                            row(ui, "File", &item.path.display().to_string());
                        });
                    ui.add_space(10.0);
                    if ui.button("Reveal in Finder").clicked() {
                        reveal(&item.path);
                    }
                });
                if let Some(details) = details {
                    ui.add_space(16.0);
                    indented(ui, |ui| {
                        ui.strong("Tags");
                        ui.add_space(4.0);
                        egui::Grid::new("track_tags")
                            .num_columns(2)
                            .spacing([12.0, 4.0])
                            .show(ui, |ui| {
                                for (name, value) in &details.tags {
                                    ui.weak(name);
                                    ui.add(egui::Label::new(value).wrap());
                                    ui.end_row();
                                }
                            });
                    });
                }
            });
    }
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.heading(text);
    });
    ui.add_space(8.0);
}

/// Content with the panel's side margin.
fn indented(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(14, 0))
        .show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            add(ui);
        });
}

/// One label and its value, as a grid row. Long values wrap.
fn row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.weak(label);
    ui.add(egui::Label::new(value).wrap());
    ui.end_row();
}

/// Select `path` in the file manager. Falls back to opening its folder where the file manager
/// can't select a file.
fn reveal(path: &Path) {
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg("-R").arg(path).spawn();
    #[cfg(target_os = "windows")]
    let result = Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let result = Command::new("xdg-open")
        .arg(path.parent().unwrap_or(path))
        .spawn();
    if let Err(e) = result {
        eprintln!(
            "player: could not show {} in the file manager: {e}",
            path.display()
        );
    }
}
