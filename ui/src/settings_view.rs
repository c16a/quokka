use std::path::PathBuf;

use eframe::egui;
use player_core::Settings;

/// Something the user asked for on the Settings screen.
pub enum Action {
    ChooseFolder(PathBuf),
    Rescan,
}

/// Library folder picker. Returns the action the user chose this frame, if any.
pub fn show(ui: &mut egui::Ui, settings: &Settings, busy: bool) -> Option<Action> {
    let mut action = None;

    ui.heading("Library");
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

    action
}
