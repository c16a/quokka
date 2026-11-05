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
}

/// Settings screen. Returns the action the user chose this frame, if any.
/// `status` is the latest message from the app: the folder being scanned, or the results.
pub fn show(ui: &mut egui::Ui, settings: &Settings, busy: bool, status: &str) -> Option<Action> {
    let mut action = None;

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
    ui.horizontal(|ui| {
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
    ui.horizontal(|ui| {
        ui.selectable_value(&mut mode, ReplayGainMode::Off, "Off");
        ui.selectable_value(&mut mode, ReplayGainMode::Track, "Track");
        ui.selectable_value(&mut mode, ReplayGainMode::Album, "Album");
    });
    if mode != settings.replay_gain {
        action = Some(Action::SetReplayGain(mode));
    }

    action
}
