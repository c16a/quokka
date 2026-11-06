use std::path::PathBuf;

use eframe::egui;
use player_core::{ReplayGainMode, Settings, Theme};

/// Something the user asked for on the Settings screen.
pub enum Action {
    AddFolder(PathBuf),
    /// The index into `Settings::library_roots`.
    RemoveFolder(usize),
    RescanFolder(usize),
    RescanAll,
    SetTheme(Theme),
    SetReplayGain(ReplayGainMode),
    SetCrossfade(f32),
    SetLyricsOnline(bool),
}

/// Settings screen. Returns the action the user chose this frame, if any.
/// `status` is the latest message from the app: the folder being scanned, or the results.
pub fn show(ui: &mut egui::Ui, settings: &Settings, busy: bool, status: &str) -> Option<Action> {
    let mut action = None;

    crate::fonts::page_title(ui, "Settings");
    ui.add_space(12.0);
    ui.heading("Library");
    if !status.is_empty() {
        ui.add_space(2.0);
        ui.weak(status);
    }
    ui.add_space(4.0);
    ui.label("Folders scanned for audio and video files:");
    ui.add_space(4.0);

    ui.add_enabled_ui(!busy, |ui| {
        if settings.library_roots.is_empty() {
            ui.weak("No folders yet.");
        }
        for (i, root) in settings.library_roots.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.monospace(root.display().to_string());
                if ui.button("Rescan").clicked() {
                    action = Some(Action::RescanFolder(i));
                }
                if ui.button("Remove").clicked() {
                    action = Some(Action::RemoveFolder(i));
                }
            });
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Add folder…").clicked()
                && let Some(folder) = rfd::FileDialog::new()
                    .set_title("Choose a folder for your library")
                    .pick_folder()
            {
                action = Some(Action::AddFolder(folder));
            }
            if !settings.library_roots.is_empty() && ui.button("Rescan all").clicked() {
                action = Some(Action::RescanAll);
            }
        });
    });

    ui.add_space(24.0);
    ui.heading("Appearance");
    ui.add_space(4.0);
    ui.label("Theme:");
    let mut theme = settings.theme;
    egui::ComboBox::from_id_salt("theme")
        .selected_text(match theme {
            Theme::System => "System",
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut theme, Theme::System, "System");
            ui.selectable_value(&mut theme, Theme::Light, "Light");
            ui.selectable_value(&mut theme, Theme::Dark, "Dark");
        });
    if theme != settings.theme {
        action = Some(Action::SetTheme(theme));
    }

    ui.add_space(24.0);
    ui.heading("Playback");
    ui.add_space(4.0);
    ui.label("ReplayGain, which levels tracks using the loudness tags in each file:");
    ui.add_space(4.0);
    let mut mode = settings.replay_gain;
    egui::ComboBox::from_id_salt("replay_gain")
        .selected_text(match mode {
            ReplayGainMode::Off => "Off",
            ReplayGainMode::Track => "Track",
            ReplayGainMode::Album => "Album",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut mode, ReplayGainMode::Off, "Off");
            ui.selectable_value(&mut mode, ReplayGainMode::Track, "Track");
            ui.selectable_value(&mut mode, ReplayGainMode::Album, "Album");
        });
    if mode != settings.replay_gain {
        action = Some(Action::SetReplayGain(mode));
    }

    ui.add_space(24.0);
    ui.label(
        "Crossfade: each track fades into the next over this many seconds. Zero turns it off.",
    );
    ui.add_space(4.0);
    let mut secs = settings.crossfade_secs;
    if ui
        .add(
            egui::Slider::new(&mut secs, 0.0..=12.0)
                .step_by(0.5)
                .suffix(" s"),
        )
        .changed()
    {
        action = Some(Action::SetCrossfade(secs));
    }

    ui.add_space(24.0);
    ui.heading("Lyrics");
    ui.add_space(4.0);
    ui.label("Look up synced lyrics on lrclib.net:");
    ui.add_space(4.0);
    let mut online = settings.lyrics_online;
    egui::ComboBox::from_id_salt("lyrics_online")
        .selected_text(if online { "On" } else { "Off" })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut online, false, "Off");
            ui.selectable_value(&mut online, true, "On");
        });
    if online != settings.lyrics_online {
        action = Some(Action::SetLyricsOnline(online));
    }
    ui.weak("For tracks with no .lrc file beside them. Sends each track's title, artist, album and length to lrclib.net. Lyrics found are cached in the library. A .lrc file always takes priority.");

    action
}
