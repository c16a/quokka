use std::path::{Path, PathBuf};

use eframe::egui;
use quokka_core::{LibraryKind, ReplayGainMode, Settings, Theme};

use crate::slider;
use crate::squircle::{self, Padding, SquircleButton};

/// Something the user asked for on the Settings screen.
pub enum Action {
    /// A new folder, and the kind of media it holds.
    AddFolder {
        path: PathBuf,
        kind: LibraryKind,
    },
    /// Give the folder at `index` in `Settings::library_roots` a kind.
    SetFolderKind {
        index: usize,
        kind: LibraryKind,
    },
    /// The index into `Settings::library_roots`.
    RemoveFolder(usize),
    RescanFolder(usize),
    RescanAll,
    SetTheme(Theme),
    SetReplayGain(ReplayGainMode),
    SetCrossfade(f32),
    SetLyricsOnline(bool),
    /// The TMDb API key the user typed. Empty removes the saved key.
    SetTmdbKey(String),
}

/// The kinds of folder, in the order the Library section lists them, with their names.
const KINDS: [(LibraryKind, &str); 2] = [
    (LibraryKind::Music, "Music"),
    (LibraryKind::Movies, "Movies"),
];

/// The colours of the Library section for the current theme.
struct LibraryColours {
    card: egui::Color32,
    row: egui::Color32,
    segment: egui::Color32,
    button: egui::Color32,
    text: egui::Color32,
    weak: egui::Color32,
    red: egui::Color32,
    /// The amber "Processing" flag: its text and its tinted background.
    amber: egui::Color32,
    amber_fill: egui::Color32,
}

impl LibraryColours {
    fn of(dark: bool) -> Self {
        if dark {
            Self {
                card: egui::Color32::from_rgb(0x1E, 0x1E, 0x22),
                row: egui::Color32::from_rgb(0x2A, 0x2A, 0x2F),
                segment: egui::Color32::from_rgb(0x2A, 0x2A, 0x2F),
                button: egui::Color32::from_rgb(0x3A, 0x3A, 0x41),
                text: egui::Color32::from_rgb(0xF2, 0xF2, 0xF2),
                weak: egui::Color32::from_rgb(0xA0, 0xA0, 0xA8),
                red: egui::Color32::from_rgb(0xFF, 0x8A, 0x80),
                amber: egui::Color32::from_rgb(0xFF, 0xB8, 0x4D),
                amber_fill: egui::Color32::from_rgba_unmultiplied(0xFF, 0xB8, 0x4D, 0x2A),
            }
        } else {
            Self {
                card: egui::Color32::WHITE,
                row: egui::Color32::from_rgb(0xF5, 0xF5, 0xF5),
                segment: egui::Color32::from_rgb(0xEE, 0xEE, 0xEE),
                button: egui::Color32::WHITE,
                text: egui::Color32::from_rgb(0x11, 0x11, 0x11),
                weak: egui::Color32::from_rgb(0x5C, 0x5C, 0x5C),
                red: egui::Color32::from_rgb(0xA3, 0x27, 0x1F),
                amber: egui::Color32::from_rgb(0x9A, 0x5B, 0x00),
                amber_fill: egui::Color32::from_rgba_unmultiplied(0xFF, 0xB8, 0x4D, 0x40),
            }
        }
    }
}

/// A white (or dark) rounded card holding one group of the Library section.
fn card(ui: &mut egui::Ui, colours: &LibraryColours, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(colours.card)
        .corner_radius(14.0)
        .inner_margin(egui::Margin::symmetric(20, 18))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui);
        });
}

/// The width below which a setting's control goes under its label instead of beside it.
const STACK_BELOW: f32 = 480.0;

/// A setting row: the label on the left and the control on the right. In a narrow card the control goes
/// under the label instead, so it cannot run past the card's left edge.
fn setting_row(
    ui: &mut egui::Ui,
    align: egui::Align,
    leading: impl FnOnce(&mut egui::Ui),
    trailing: impl FnOnce(&mut egui::Ui),
) {
    if ui.available_width() < STACK_BELOW {
        ui.vertical(|ui| {
            leading(ui);
            ui.add_space(10.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), trailing);
        });
    } else {
        ui.horizontal(|ui| {
            ui.vertical(leading);
            ui.with_layout(egui::Layout::right_to_left(align), trailing);
        });
    }
}

/// A row of options drawn as one pill-shaped segmented control. The chosen option is filled. Returns
/// the option the user picked, if it is a different one.
fn segmented<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    colours: &LibraryColours,
    options: &[(T, &str)],
    current: T,
) -> Option<T> {
    let mut picked = None;
    // Tighter pills in a narrow card, so the options fit beside or under the label.
    let padding = if ui.available_width() < 300.0 {
        12.0
    } else {
        22.0
    };
    egui::Frame::new()
        .fill(colours.segment)
        .corner_radius(999.0)
        .inner_margin(egui::Margin::symmetric(3, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().button_padding.x = padding;
                for &(value, label) in options {
                    let selected = value == current;
                    let (fill, text) = if selected {
                        (colours.text, colours.card)
                    } else {
                        (egui::Color32::TRANSPARENT, colours.text)
                    };
                    if pill(ui, label, fill, text, 32.0) && !selected {
                        picked = Some(value);
                    }
                }
            });
        });
    picked
}

/// An on/off switch. Returns true when it is clicked, so the caller can flip the setting.
fn switch(ui: &mut egui::Ui, colours: &LibraryColours, on: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(44.0, 26.0), egui::Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let (track, knob, knob_x) = if on {
        (colours.text, colours.card, rect.right() - 13.0)
    } else {
        (colours.segment, colours.text, rect.left() + 13.0)
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 13.0, track);
    painter.circle_filled(egui::pos2(knob_x, rect.center().y), 10.0, knob);
    response.clicked()
}

/// A small outlined label, such as a kind's name.
fn chip(ui: &mut egui::Ui, label: &str, text: egui::Color32, stroke: egui::Color32) {
    egui::Frame::new()
        .stroke(egui::Stroke::new(1.0, stroke))
        .corner_radius(999.0)
        .inner_margin(egui::Margin::symmetric(10, 3))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(label.to_uppercase())
                    .size(12.0)
                    .strong()
                    .color(text),
            );
        });
}

/// A folder row's button: 32 px tall with slightly rounded corners. Returns true when it is clicked.
fn row_button(ui: &mut egui::Ui, label: &str, fill: egui::Color32, text: egui::Color32) -> bool {
    squircle_label(ui, label, fill, text, Padding::symmetric(14.0, 6.0)).clicked()
}

/// A rounded button with a label, at least `height` points tall. Returns true when it is clicked.
fn pill(
    ui: &mut egui::Ui,
    label: &str,
    fill: egui::Color32,
    text: egui::Color32,
    height: f32,
) -> bool {
    let vertical = ((height - LABEL_LINE) / 2.0).max(0.0);
    squircle_label(ui, label, fill, text, Padding::symmetric(18.0, vertical)).clicked()
}

/// The height of one line of 14-point text, which the padding is measured from.
const LABEL_LINE: f32 = 18.0;

/// A squircle button with one line of 14-point text.
fn squircle_label(
    ui: &mut egui::Ui,
    label: &str,
    fill: egui::Color32,
    text: egui::Color32,
    padding: Padding,
) -> egui::Response {
    let text = egui::RichText::new(label).size(14.0).color(text);
    SquircleButton::new(egui::Label::new(text).extend())
        .padding(padding)
        .fill(fill)
        .show(ui)
}

/// The chip for a kind and the number of folders and files under it.
fn kind_header(
    ui: &mut egui::Ui,
    colours: &LibraryColours,
    label: &str,
    folders: usize,
    files: i64,
) {
    ui.horizontal(|ui| {
        chip(ui, label, colours.text, colours.text);
        let folders_text = if folders == 1 {
            "1 folder".to_owned()
        } else {
            format!("{folders} folders")
        };
        ui.label(
            egui::RichText::new(format!("{folders_text} · {files} files"))
                .size(13.0)
                .color(colours.weak),
        );
    });
}

/// One library folder: a folder icon, its path and its file count, then Rescan and Remove. A folder
/// with no kind yet also gets buttons to choose one.
fn folder_row(
    ui: &mut egui::Ui,
    colours: &LibraryColours,
    root: &Path,
    index: usize,
    files: i64,
    unset: bool,
) -> Option<Action> {
    let mut action = None;
    egui::Frame::new()
        .fill(colours.row)
        .corner_radius(10.0)
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                // The icon sits on the left. The buttons keep their room on the right, and the path takes what is
                // left between them. The icon comes first so the path, which fills its space, cannot push it out.
                let (icon, _) =
                    ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
                folder_icon(ui.painter(), icon, colours.text);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if row_button(ui, "Remove", egui::Color32::TRANSPARENT, colours.red) {
                        action = Some(Action::RemoveFolder(index));
                    }
                    if row_button(ui, "Rescan", colours.button, colours.text) {
                        action = Some(Action::RescanFolder(index));
                    }
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(root.display().to_string())
                                    .monospace()
                                    .size(14.0)
                                    .color(colours.text),
                            )
                            .truncate(),
                        );
                        ui.label(
                            egui::RichText::new(format!("{files} files"))
                                .size(13.0)
                                .color(colours.weak),
                        );
                    });
                });
            });
            if unset {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Holds:").size(13.0).color(colours.weak));
                    for (kind, label) in KINDS {
                        if pill(ui, label, colours.button, colours.text, 28.0) {
                            action = Some(Action::SetFolderKind { index, kind });
                        }
                    }
                });
            }
        });
    action
}

/// A folder outline, drawn in the same shape as the mockup's icon.
fn folder_icon(painter: &egui::Painter, rect: egui::Rect, colour: egui::Color32) {
    let scale = rect.width() / 24.0;
    let at = |x: f32, y: f32| rect.min + egui::vec2(x, y) * scale;
    let outline = [
        at(3.0, 7.0),
        at(5.0, 5.0),
        at(9.0, 5.0),
        at(11.0, 7.0),
        at(19.0, 7.0),
        at(21.0, 9.0),
        at(21.0, 17.0),
        at(19.0, 19.0),
        at(5.0, 19.0),
        at(3.0, 17.0),
    ];
    painter.add(egui::Shape::closed_line(
        outline.to_vec(),
        egui::Stroke::new(1.8 * scale, colour),
    ));
}

/// Settings screen. Returns the action the user chose this frame, if any.
/// `status` is the latest message from the app: the folder being scanned, or the results.
pub fn show(
    ui: &mut egui::Ui,
    settings: &Settings,
    busy: bool,
    status: &str,
    folder_files: &[i64],
    processing: bool,
) -> Option<Action> {
    let mut action = None;

    // The whole screen scrolls, so the sections below the Library section can be reached.
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
    // The padding is inside the scroll area, so the scrollbar sits at the window's edge.
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(32, 32))
        .show(ui, |ui| {
    crate::fonts::page_title(ui, "Settings");
    ui.add_space(12.0);
    let colours = LibraryColours::of(ui.visuals().dark_mode);
    let folders = settings.library_roots.len();
    let files: i64 = folder_files.iter().sum();
    let kind_id = ui.scope_id().with("library_add_kind");
    let mut kind = ui
        .data(|data| data.get_temp::<LibraryKind>(kind_id))
        .unwrap_or(LibraryKind::Music);

    ui.add_enabled_ui(!busy, |ui| {
        setting_row(
            ui,
            egui::Align::Center,
            |ui| {
                ui.heading("Library");
                let summary = match folders {
                    0 => "No folders yet.".to_owned(),
                    1 => format!("1 folder · {files} files"),
                    n => format!("{n} folders · {files} files"),
                };
                ui.label(egui::RichText::new(summary).size(14.0).color(colours.weak));
            },
            |ui| {
                if folders > 0 {
                    // Wider than the other buttons, so the label has room on each side.
                    let clicked = ui
                        .scope(|ui| {
                            ui.spacing_mut().button_padding.x = 22.0;
                            pill(ui, "Rescan all", colours.button, colours.text, 36.0)
                        })
                        .inner;
                    if clicked {
                        action = Some(Action::RescanAll);
                    }
                }
            },
        );
        if processing {
            ui.add_space(4.0);
            egui::Frame::new()
                .fill(colours.amber_fill)
                .corner_radius(999.0)
                .inner_margin(egui::Margin::symmetric(10, 3))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Processing")
                            .size(12.0)
                            .color(colours.amber),
                    );
                });
        }
        if !status.is_empty() {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(status).size(13.0).color(colours.weak));
        }
        ui.add_space(20.0);

        // One card per kind of folder. A folder with no kind yet gets its own card, listed after them.
        for (kind_value, label) in KINDS {
            let rows: Vec<usize> = (0..folders)
                .filter(|&i| settings.library_kind(&settings.library_roots[i]) == Some(kind_value))
                .collect();
            if rows.is_empty() {
                continue;
            }
            let rows_files: i64 = rows
                .iter()
                .map(|&i| folder_files.get(i).copied().unwrap_or(0))
                .sum();
            card(ui, &colours, |ui| {
                kind_header(ui, &colours, label, rows.len(), rows_files);
                ui.add_space(14.0);
                for i in rows {
                    if let Some(a) = folder_row(
                        ui,
                        &colours,
                        settings.library_roots[i].as_path(),
                        i,
                        folder_files.get(i).copied().unwrap_or(0),
                        false,
                    ) {
                        action = Some(a);
                    }
                    ui.add_space(8.0);
                }
            });
            ui.add_space(14.0);
        }

        let unset: Vec<usize> = (0..folders)
            .filter(|&i| settings.library_kind(&settings.library_roots[i]).is_none())
            .collect();
        if !unset.is_empty() {
            card(ui, &colours, |ui| {
                ui.horizontal(|ui| {
                    chip(ui, "Kind not set", colours.text, colours.text);
                    ui.label(
                        egui::RichText::new("Choose what each of these folders holds.")
                            .size(13.0)
                            .color(colours.weak),
                    );
                });
                ui.add_space(14.0);
                for i in unset {
                    if let Some(a) = folder_row(
                        ui,
                        &colours,
                        settings.library_roots[i].as_path(),
                        i,
                        folder_files.get(i).copied().unwrap_or(0),
                        true,
                    ) {
                        action = Some(a);
                    }
                    ui.add_space(8.0);
                }
            });
            ui.add_space(14.0);
        }

        // TV shows are not supported yet.
        card(ui, &colours, |ui| {
            ui.horizontal(|ui| {
                chip(ui, "TV Shows", colours.weak, colours.weak);
                ui.label(
                    egui::RichText::new("Coming soon")
                        .size(13.0)
                        .color(colours.weak),
                );
            });
        });
        ui.add_space(14.0);

        card(ui, &colours, |ui| {
            ui.label(egui::RichText::new("Add a folder").size(16.0).strong());
            ui.label(
                egui::RichText::new(
                    "Each folder holds one kind of media. Other kinds of file in it are skipped.",
                )
                .size(13.0)
                .color(colours.weak),
            );
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                // The same wider padding as "Rescan all", tightened when the window is narrow.
                ui.spacing_mut().button_padding.x = if ui.available_width() < 420.0 { 12.0 } else { 22.0 };
                egui::Frame::new()
                    .fill(colours.segment)
                    .corner_radius(999.0)
                    .inner_margin(egui::Margin::symmetric(3, 3))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (k, label) in KINDS {
                                let selected = kind == k;
                                let (fill, text) = if selected {
                                    (colours.text, colours.card)
                                } else {
                                    (egui::Color32::TRANSPARENT, colours.text)
                                };
                                if pill(ui, label, fill, text, 32.0) {
                                    kind = k;
                                }
                            }
                            ui.add_enabled_ui(false, |ui| {
                                squircle_label(
                                    ui,
                                    "TV Shows · Soon",
                                    egui::Color32::TRANSPARENT,
                                    colours.weak,
                                    Padding::symmetric(14.0, 6.0),
                                )
                            });
                        });
                    });
                ui.add_space(8.0);
                if pill(ui, "Choose folder…", colours.text, colours.card, 40.0)
                    && let Some(path) = rfd::FileDialog::new()
                        .set_title("Choose a folder for your library")
                        .pick_folder()
                {
                    action = Some(Action::AddFolder { path, kind });
                }
            });
        });
    });
    ui.data_mut(|data| data.insert_temp(kind_id, kind));

    ui.add_space(14.0);
    card(ui, &colours, |ui| {
        ui.label(egui::RichText::new("Appearance").size(16.0).strong());
        ui.add_space(14.0);
        setting_row(
            ui,
            egui::Align::Center,
            |ui| {
                ui.label(egui::RichText::new("Theme").size(14.0).color(colours.weak));
            },
            |ui| {
                // Listed in reverse: the row is right-to-left, so the first option ends up rightmost.
                let themes = [
                    (Theme::Dark, "Dark"),
                    (Theme::Light, "Light"),
                    (Theme::System, "System"),
                ];
                if let Some(theme) = segmented(ui, &colours, &themes, settings.theme) {
                    action = Some(Action::SetTheme(theme));
                }
            },
        );
    });

    ui.add_space(14.0);
    card(ui, &colours, |ui| {
        ui.label(egui::RichText::new("Playback").size(16.0).strong());
        ui.add_space(14.0);
        setting_row(
            ui,
            egui::Align::Min,
            |ui| {
                ui.set_max_width(ui.available_width().min(420.0));
                ui.label(egui::RichText::new("Loudness").size(14.0).color(colours.text));
                ui.label(egui::RichText::new("Levels tracks to the same loudness. Files with ReplayGain tags use them; other files are measured in the background as they play, and keep their own level until then.").size(13.0).color(colours.weak));
            },
            |ui| {
                // Listed in reverse, as above.
                let modes = [
                    (ReplayGainMode::Album, "Album"),
                    (ReplayGainMode::Track, "Track"),
                    (ReplayGainMode::Off, "Off"),
                ];
                if let Some(mode) = segmented(ui, &colours, &modes, settings.replay_gain) {
                    action = Some(Action::SetReplayGain(mode));
                }
            },
        );
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(16.0);
        ui.label(egui::RichText::new("Crossfade").size(14.0).color(colours.text));
        ui.label(egui::RichText::new("Each track fades into the next over this many seconds. Zero turns it off.").size(13.0).color(colours.weak));
        ui.add_space(10.0);
        let mut secs = f64::from(settings.crossfade_secs);
        let changed = ui
            .horizontal(|ui| {
                let changed = slider::show(ui, &mut secs, 0.0..=12.0, 0.5, 300.0).changed();
                ui.label(egui::RichText::new(format!("{secs:.1} s")).size(14.0).color(colours.weak));
                changed
            })
            .inner;
        if changed {
            action = Some(Action::SetCrossfade(secs as f32));
        }
    });

    ui.add_space(14.0);
    card(ui, &colours, |ui| {
        ui.label(egui::RichText::new("Lyrics").size(16.0).strong());
        ui.add_space(14.0);
        setting_row(
            ui,
            egui::Align::Center,
            |ui| {
                ui.label(egui::RichText::new("Look up synced lyrics on lrclib.net").size(14.0).color(colours.text));
            },
            |ui| {
                if switch(ui, &colours, settings.lyrics_online) {
                    action = Some(Action::SetLyricsOnline(!settings.lyrics_online));
                }
            },
        );
        ui.add_space(8.0);
        ui.label(egui::RichText::new("For tracks with no .lrc file beside them. Sends each track's title, artist, album and length to lrclib.net. Lyrics found are cached in the library. A .lrc file always takes priority.").size(13.0).color(colours.weak));
    });

    ui.add_space(14.0);
    card(ui, &colours, |ui| {
        ui.label(egui::RichText::new("Movie details").size(16.0).strong());
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Descriptions and cast for movies come from TMDb. Enter your own TMDb API key to use them. The key is saved in your settings file, readable only by your user account, and is sent only to TMDb.").size(13.0).color(colours.weak));
        ui.add_space(10.0);
        let draft_id = ui.scope_id().with("tmdb_key_draft");
        let mut draft: String = ui.data_mut(|d| d.get_temp(draft_id).unwrap_or_default());
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut draft)
                    .password(true)
                    .hint_text("TMDb API key")
                    .desired_width(280.0),
            );
            if squircle::text_button(ui, "Save key").clicked() {
                action = Some(Action::SetTmdbKey(draft.trim().to_owned()));
            }
            if settings.tmdb_api_key.is_some() && squircle::text_button(ui, "Remove key").clicked() {
                action = Some(Action::SetTmdbKey(String::new()));
            }
        });
        ui.data_mut(|d| d.insert_temp(draft_id, draft.clone()));
        if action.is_some() {
            ui.data_mut(|d| d.insert_temp(draft_id, String::new()));
        }
        let state = if settings.tmdb_api_key.is_some() {
            "A key is saved."
        } else {
            "No key saved. Movies show no descriptions or cast until you add one."
        };
        ui.add_space(6.0);
        ui.label(egui::RichText::new(state).size(13.0).color(colours.weak));
        ui.add_space(10.0);
        ui.label(egui::RichText::new("This product uses the TMDB API but is not endorsed or certified by TMDB.").size(12.0).color(colours.weak));
    });
        });
        });

    action
}
