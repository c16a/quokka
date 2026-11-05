use std::path::PathBuf;

use eframe::egui;
use player_core::{Settings, Theme};

/// Something the user asked for on the Settings screen.
pub enum Action {
    ChooseFolder(PathBuf),
    Rescan,
    SetTheme(Theme),
}

/// Settings screen. Returns the action the user chose this frame, if any.
/// `status` is the latest message from the app, such as a scan result or an error.
pub fn show(ui: &mut egui::Ui, settings: &Settings, busy: bool, status: &str) -> Option<Action> {
    let mut action = None;

    ui.heading("Library");
    if !status.is_empty() {
        ui.add_space(2.0);
        ui.weak(status);
    }
    ui.add_space(4.0);
    ui.label("Folder to scan for audio and video files:");
    match &settings.library_root {
        Some(root) => {
            ui.monospace(root.display().to_string());
        }
        None => {
            ui.weak("Not set");
        }
    }
    ui.add_space(12.0);

    ui.add_enabled_ui(!busy, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Choose folder…").clicked()
                && let Some(folder) = rfd::FileDialog::new()
                    .set_title("Choose your library folder")
                    .pick_folder()
            {
                action = Some(Action::ChooseFolder(folder));
            }
            if settings.library_root.is_some() && ui.button("Rescan").clicked() {
                action = Some(Action::Rescan);
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

    action
}
