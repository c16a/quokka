//! Internet radio: the stations the user has tuned in to, and a search of the station index.

use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;
use player_core::{Library, Station};

/// What the user asked for on the Radio tab this frame.
pub enum RadioAction {
    /// Start playing a tuned-in station.
    Play(Station),
    /// Add a found station to the tuned-in list.
    TuneIn(Station),
    /// Remove a station from the tuned-in list. Holds its uuid.
    TuneOut(String),
    /// Refresh the station index from radio-browser.info.
    Sync,
}

/// Most stations listed for one search.
const RESULT_LIMIT: usize = 60;

#[derive(Default)]
pub struct RadioView {
    tuned: Vec<Station>,
    query: String,
    /// The query the results were built for.
    searched: Option<String>,
    results: Vec<Station>,
    count: i64,
    /// When the index was last synced, in seconds since the Unix epoch.
    last_sync: Option<i64>,
}

impl RadioView {
    /// Reload the tuned-in list, the index size and the sync time, and search again.
    pub fn reload(&mut self, library: &Library) {
        self.tuned = library.tuned_stations().unwrap_or_default();
        self.count = library.station_count().unwrap_or(0);
        self.last_sync = library
            .meta("stations_synced")
            .ok()
            .flatten()
            .and_then(|value| value.parse().ok());
        self.searched = None;
        self.refresh(library);
    }

    fn refresh(&mut self, library: &Library) {
        let query = self.query.trim().to_owned();
        if self.searched.as_deref() == Some(query.as_str()) {
            return;
        }
        self.results = library
            .search_stations(&query, RESULT_LIMIT)
            .unwrap_or_default();
        self.searched = Some(query);
    }

    /// Draw the tab. `playing` is the uuid of the station on air, and `syncing` says whether an
    /// index sync is running.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        library: &Library,
        playing: Option<&str>,
        syncing: bool,
    ) -> Option<RadioAction> {
        let mut action = None;
        ui.heading("Radio");
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let status = if self.count == 0 {
                "No stations yet.".to_owned()
            } else {
                format!("{} stations · synced {}", self.count, age(self.last_sync))
            };
            ui.weak(status);
            ui.add_enabled_ui(!syncing, |ui| {
                if ui.button("Sync now").clicked() {
                    action = Some(RadioAction::Sync);
                }
            });
            if syncing {
                ui.spinner();
                ui.weak("Syncing…");
            }
        });
        ui.add_space(14.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.strong("Tuned in");
                ui.add_space(6.0);
                if self.tuned.is_empty() {
                    ui.weak("Search below and tune in to a station. It will appear here.");
                }
                for station in &self.tuned {
                    let on_air = playing == Some(station.uuid.as_str());
                    let response = ui.add_sized(
                        [ui.available_width(), 40.0],
                        egui::Button::new(label(station, on_air)).corner_radius(6.0),
                    );
                    if response.clicked() {
                        action = Some(RadioAction::Play(station.clone()));
                    }
                    response.context_menu(|ui| {
                        if ui.button("Tune out").clicked() {
                            action = Some(RadioAction::TuneOut(station.uuid.clone()));
                            ui.close();
                        }
                    });
                }

                ui.add_space(22.0);
                ui.strong("Find stations");
                ui.add_space(6.0);
                let field = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Name, country, genre or language")
                        .desired_width(360.0),
                );
                if field.changed() {
                    self.refresh(library);
                }
                ui.add_space(8.0);
                if self.results.is_empty() {
                    if self.count == 0 {
                        ui.weak("Sync the station list to search it.");
                    } else {
                        ui.weak("No stations match.");
                    }
                }
                for station in &self.results {
                    ui.horizontal(|ui| {
                        ui.label(station_text(station));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if self.tuned.iter().any(|t| t.uuid == station.uuid) {
                                ui.weak("Tuned in");
                            } else if ui.button("Tune in").clicked() {
                                action = Some(RadioAction::TuneIn(station.clone()));
                            }
                        });
                    });
                }
            });
        action
    }
}

/// A station's name, and what it plays, for a button.
fn label(station: &Station, on_air: bool) -> String {
    let prefix = if on_air { "▶  " } else { "" };
    format!("{prefix}{}", station_text(station))
}

/// A station's name with its country, codec and bitrate.
fn station_text(station: &Station) -> String {
    let mut parts = Vec::new();
    if !station.country.is_empty() {
        parts.push(station.country.clone());
    }
    if !station.codec.is_empty() {
        parts.push(station.codec.clone());
    }
    if station.bitrate > 0 {
        parts.push(format!("{} kbps", station.bitrate));
    }
    if parts.is_empty() {
        station.name.clone()
    } else {
        format!("{}   ·   {}", station.name, parts.join(" · "))
    }
}

/// How long ago the index was synced, in words.
fn age(last: Option<i64>) -> String {
    let Some(last) = last else {
        return "never".into();
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(last);
    let minutes = (now - last).max(0) / 60;
    match minutes {
        0..=1 => "just now".into(),
        2..=59 => format!("{minutes} minutes ago"),
        60..=1439 => format!("{} hours ago", minutes / 60),
        _ => format!("{} days ago", minutes / 1440),
    }
}
