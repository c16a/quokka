//! Internet radio: the station now on air, the stations the user has tuned in to, and a search
//! of the station index.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::icons;
use eframe::egui::{self, Sense};
use player_core::{Library, Station};

/// What the user asked for on the Radio tab this frame.
pub enum RadioAction {
    /// Start playing a tuned-in station.
    Play(Station),
    /// Play or pause the station on air.
    TogglePlayback,
    /// Add a found station to the tuned-in list, and start it playing.
    TuneInAndPlay(Station),
    /// Remove a station from the tuned-in list. Holds its uuid.
    TuneOut(String),
    /// Refresh the station index from radio-browser.info.
    Sync,
}

/// Most stations loaded for one search. The filters narrow these.
const RESULT_LIMIT: usize = 120;
/// Width of a search result card, in points.
const CARD_WIDTH: f32 = 260.0;
/// Height of a search result card, in points.
const CARD_HEIGHT: f32 = 66.0;
/// Stations per row of cards.
const CARDS_PER_ROW: usize = 3;
/// Colours for station tiles. Each station keeps the same one.
const TILE_COLOURS: [egui::Color32; 6] = [
    egui::Color32::from_rgb(154, 52, 18),
    egui::Color32::from_rgb(30, 58, 138),
    egui::Color32::from_rgb(76, 29, 149),
    egui::Color32::from_rgb(54, 83, 20),
    egui::Color32::from_rgb(15, 118, 110),
    egui::Color32::from_rgb(190, 24, 93),
];

#[derive(Default)]
pub struct RadioView {
    tuned: Vec<Station>,
    query: String,
    /// The query the results were built for.
    searched: Option<String>,
    results: Vec<Station>,
    /// Filters applied to the results. `None` and 0 mean "any".
    country: Option<String>,
    codec: Option<String>,
    min_bitrate: i64,
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

    /// Draw the tab. `now_playing` is the station on air, if the output is live radio, and
    /// `syncing` says whether an index sync is running.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        library: &Library,
        now_playing: Option<&Station>,
        paused: bool,
        syncing: bool,
    ) -> Option<RadioAction> {
        let mut action = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(24, 20))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("Radio")
                                        .size(34.0)
                                        .family(crate::fonts::bold_family()),
                                );
                                let status = if self.count == 0 {
                                    "No stations yet.".to_owned()
                                } else {
                                    format!(
                                        "{} stations in the directory · synced {}",
                                        self.count,
                                        age(self.last_sync)
                                    )
                                };
                                ui.weak(status);
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_enabled_ui(!syncing, |ui| {
                                        if pill(ui, "Sync now").clicked() {
                                            action = Some(RadioAction::Sync);
                                        }
                                    });
                                    if syncing {
                                        ui.spinner();
                                        ui.weak("Syncing…");
                                    }
                                },
                            );
                        });
                        ui.add_space(18.0);

                        if let Some(station) = now_playing {
                            if let Some(act) = on_air(ui, station, paused) {
                                action = Some(act);
                            }
                            ui.add_space(22.0);
                        }

                        ui.horizontal(|ui| {
                            ui.label(crate::fonts::bold("Your stations"));
                            ui.weak(format!("{} tuned in", self.tuned.len()));
                        });
                        ui.add_space(8.0);
                        if self.tuned.is_empty() {
                            ui.weak("Find a station below and tune in. It will appear here.");
                        }
                        let playing = now_playing.map(|s| s.uuid.as_str());
                        for row in self.tuned.chunks(CARDS_PER_ROW) {
                            ui.columns(CARDS_PER_ROW, |columns| {
                                for (column, station) in columns.iter_mut().zip(row) {
                                    let on_air = playing == Some(station.uuid.as_str());
                                    if let Some(act) = station_card(column, station, on_air) {
                                        action = Some(act);
                                    }
                                }
                            });
                            ui.add_space(12.0);
                        }

                        ui.add_space(22.0);
                        ui.label(crate::fonts::bold("Find stations"));
                        ui.add_space(8.0);
                        if search_field(ui, &mut self.query) {
                            self.refresh(library);
                        }
                        ui.add_space(10.0);
                        if let Some(act) = self.filters(ui) {
                            action = Some(act);
                        }
                        ui.add_space(10.0);

                        let visible = self.visible();
                        if visible.is_empty() {
                            if self.count == 0 {
                                ui.weak("Sync the station list to search it.");
                            } else {
                                ui.weak("No stations match.");
                            }
                        }
                        if !visible.is_empty()
                            && let Some(act) = self.results_grid(ui, &visible)
                        {
                            action = Some(act);
                        }
                    });
            });
        action
    }

    /// The country, codec and bitrate menus. They list what the loaded results contain.
    fn filters(&mut self, ui: &mut egui::Ui) -> Option<RadioAction> {
        let mut countries: Vec<&str> = self
            .results
            .iter()
            .map(|s| s.country.as_str())
            .filter(|c| !c.is_empty())
            .collect();
        countries.sort_unstable();
        countries.dedup();
        let mut codecs: Vec<&str> = self
            .results
            .iter()
            .map(|s| s.codec.as_str())
            .filter(|c| !c.is_empty())
            .collect();
        codecs.sort_unstable();
        codecs.dedup();

        let mut country = self.country.clone();
        let mut codec = self.codec.clone();
        let mut min_bitrate = self.min_bitrate;
        ui.horizontal(|ui| {
            ui.spacing_mut().button_padding = egui::vec2(12.0, 6.0);
            let widgets = &mut ui.style_mut().visuals.widgets;
            for state in [
                &mut widgets.inactive,
                &mut widgets.hovered,
                &mut widgets.active,
            ] {
                state.corner_radius = egui::CornerRadius::same(255);
            }
            egui::ComboBox::from_id_salt("radio_country")
                .selected_text(country.clone().unwrap_or_else(|| "Country: any".into()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut country, None, "Any country");
                    for name in &countries {
                        ui.selectable_value(&mut country, Some((*name).to_owned()), *name);
                    }
                });
            egui::ComboBox::from_id_salt("radio_codec")
                .selected_text(codec.clone().unwrap_or_else(|| "Codec: any".into()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut codec, None, "Any codec");
                    for name in &codecs {
                        ui.selectable_value(&mut codec, Some((*name).to_owned()), *name);
                    }
                });
            let bitrate_label = if min_bitrate == 0 {
                "Bitrate: any".to_owned()
            } else {
                format!("Bitrate: {min_bitrate}+ kbps")
            };
            egui::ComboBox::from_id_salt("radio_bitrate")
                .selected_text(bitrate_label)
                .show_ui(ui, |ui| {
                    for (value, label) in [
                        (0, "Any bitrate"),
                        (64, "64+ kbps"),
                        (128, "128+ kbps"),
                        (256, "256+ kbps"),
                    ] {
                        ui.selectable_value(&mut min_bitrate, value, label);
                    }
                });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.weak("Most popular first");
            });
        });
        self.country = country;
        self.codec = codec;
        self.min_bitrate = min_bitrate;
        None
    }

    /// The loaded results that pass the filters.
    fn visible(&self) -> Vec<&Station> {
        self.results
            .iter()
            .filter(|s| self.country.as_ref().is_none_or(|c| &s.country == c))
            .filter(|s| self.codec.as_ref().is_none_or(|c| &s.codec == c))
            .filter(|s| s.bitrate >= self.min_bitrate)
            .collect()
    }

    /// The results as a grid of cards. Each card shows the station's tile, name and details.
    /// Hovering a card turns the tile into a play button, which tunes in to the station and
    /// starts it. The grid has as many columns as the width allows.
    fn results_grid(&self, ui: &mut egui::Ui, visible: &[&Station]) -> Option<RadioAction> {
        const GAP: f32 = 12.0;
        let mut action = None;
        // As many fixed-width columns as the window fits.
        let columns = (((ui.available_width() + GAP) / (CARD_WIDTH + GAP)).floor() as usize).max(1);
        egui::Grid::new("radio_results_grid")
            .num_columns(columns)
            .spacing([GAP, GAP])
            .min_col_width(CARD_WIDTH)
            .max_col_width(CARD_WIDTH)
            .show(ui, |ui| {
                for (n, station) in visible.iter().enumerate() {
                    let tuned = self.tuned.iter().any(|t| t.uuid == station.uuid);
                    if let Some(act) = result_card(ui, station, tuned) {
                        action = Some(act);
                    }
                    if (n + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
                ui.end_row();
            });
        action
    }
}

/// The station on air: its tile, name, whether it is live, a level meter, and play or pause.
fn on_air(ui: &mut egui::Ui, station: &Station, paused: bool) -> Option<RadioAction> {
    let mut action = None;
    let accent = ui.visuals().selection.bg_fill;
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .stroke(egui::Stroke::new(1.0, accent))
        .corner_radius(14.0)
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                tile(ui, station, 60.0);
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    let (label, colour) = if paused {
                        ("PAUSED", ui.visuals().weak_text_color())
                    } else {
                        ("LIVE NOW", egui::Color32::from_rgb(110, 231, 183))
                    };
                    ui.colored_label(
                        colour,
                        egui::RichText::new(label)
                            .small()
                            .family(crate::fonts::bold_family()),
                    );
                    ui.heading(&station.name);
                    ui.weak(station_details(station));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let icon = if paused { Icon::Play } else { Icon::Pause };
                    let tip = if paused { "Play" } else { "Pause" };
                    if round_button(ui, icon, 44.0).on_hover_text(tip).clicked() {
                        action = Some(RadioAction::TogglePlayback);
                    }
                    ui.add_space(16.0);
                    level_meter(ui, !paused);
                });
            });
        });
    action
}

/// A card for a tuned-in station: tile, name, details, and a play button. The station on air is
/// outlined.
fn station_card(ui: &mut egui::Ui, station: &Station, on_air: bool) -> Option<RadioAction> {
    let mut action = None;
    let stroke = if on_air {
        egui::Stroke::new(1.5, ui.visuals().selection.bg_fill)
    } else {
        ui.visuals().widgets.noninteractive.bg_stroke
    };
    let response = egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .stroke(stroke)
        .corner_radius(12.0)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                tile(ui, station, 42.0);
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width() - 44.0);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(&station.name).family(crate::fonts::bold_family()),
                        )
                        .truncate(),
                    );
                    ui.add(
                        egui::Label::new(egui::RichText::new(station_details(station)).weak())
                            .truncate(),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if round_button(ui, Icon::Play, 32.0)
                        .on_hover_text("Play")
                        .clicked()
                    {
                        action = Some(RadioAction::Play(station.clone()));
                    }
                });
            });
        })
        .response;
    response.context_menu(|ui| {
        if ui.button("Tune out").clicked() {
            action = Some(RadioAction::TuneOut(station.uuid.clone()));
            ui.close();
        }
    });
    action
}

/// Rounded search field with a magnifier.
fn search_field(ui: &mut egui::Ui, query: &mut String) -> bool {
    let fill = if ui.visuals().dark_mode {
        egui::Color32::from_white_alpha(18)
    } else {
        egui::Color32::from_black_alpha(12)
    };
    let mut changed = false;
    egui::Frame::new()
        .fill(fill)
        .corner_radius(10.0)
        .inner_margin(egui::Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                icons::paint(
                    ui.painter(),
                    rect.center(),
                    icons::MAGNIFYING_GLASS,
                    16.0,
                    ui.visuals().weak_text_color(),
                );
                ui.add_space(6.0);
                let field = ui.add(
                    egui::TextEdit::singleline(query)
                        .frame(egui::Frame::NONE)
                        .hint_text("Search by name, country, language or genre")
                        .margin(egui::Margin::symmetric(10, 6))
                        .align(egui::Align2::LEFT_CENTER)
                        .desired_width(f32::INFINITY),
                );
                changed = field.changed();
            });
        });
    changed
}

/// A square tile with the station's initials on its colour.
fn tile(ui: &mut egui::Ui, station: &Station, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
    paint_tile(ui.painter(), rect, station);
}

/// Paint a station's tile into `rect`.
fn paint_tile(painter: &egui::Painter, rect: egui::Rect, station: &Station) {
    let colour = TILE_COLOURS[colour_index(&station.name)];
    painter.rect_filled(rect, rect.width() * 0.22, colour);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        initials(&station.name),
        egui::FontId::proportional(rect.width() * 0.36),
        egui::Color32::WHITE,
    );
}

/// The first letters of the first two words, in capitals.
fn initials(name: &str) -> String {
    let letters: String = name
        .split_whitespace()
        .filter_map(|word| word.chars().find(|c| c.is_alphanumeric()))
        .take(2)
        .flat_map(|c| c.to_uppercase())
        .collect();
    if letters.is_empty() {
        "?".into()
    } else {
        letters
    }
}

/// A stable colour for a station, from its name.
fn colour_index(name: &str) -> usize {
    name.bytes().fold(0usize, |hash, b| {
        hash.wrapping_mul(31).wrapping_add(b as usize)
    }) % TILE_COLOURS.len()
}

/// "Country · codec · bitrate", leaving out what the station doesn't list.
fn station_details(station: &Station) -> String {
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
        "Internet radio".into()
    } else {
        parts.join(" · ")
    }
}

#[derive(Clone, Copy)]
enum Icon {
    Play,
    Pause,
}

/// A round button with a play triangle or two pause bars.
fn round_button(ui: &mut egui::Ui, icon: Icon, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    let painter = ui.painter();
    painter.circle_filled(rect.center(), size / 2.0, visuals.weak_bg_fill);
    let glyph = match icon {
        Icon::Play => icons::PLAY,
        Icon::Pause => icons::PAUSE,
    };
    icons::paint(
        painter,
        rect.center(),
        glyph,
        size * 0.5,
        visuals.fg_stroke.color,
    );
    response
}

/// Five bars that move while a station is live, and stay low when it isn't.
fn level_meter(ui: &mut egui::Ui, live: bool) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 28.0), egui::Sense::hover());
    let time = ui.input(|i| i.time);
    let colour = ui.visuals().selection.bg_fill;
    for i in 0..5 {
        let phase = i as f64 * 1.3;
        let level = if live {
            0.5 + 0.5 * (time * 6.0 + phase).sin()
        } else {
            0.1
        };
        let height = 6.0 + 20.0 * level as f32;
        let x = rect.left() + 2.0 + i as f32 * 6.0;
        let bar = egui::Rect::from_min_max(
            egui::pos2(x, rect.bottom() - height),
            egui::pos2(x + 3.0, rect.bottom()),
        );
        ui.painter().rect_filled(bar, 2.0, colour);
    }
    if live {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(80));
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

/// A filled accent button, as used for actions such as Sync now.
fn pill(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(egui::Color32::WHITE))
            .fill(ui.visuals().selection.bg_fill)
            .stroke(egui::Stroke::NONE)
            .corner_radius(6.0)
            .min_size(egui::vec2(0.0, 30.0)),
    )
}

/// A search result: the station's tile, name and details. Hovering turns the tile into a play
/// button.
fn result_card(ui: &mut egui::Ui, station: &Station, tuned: bool) -> Option<RadioAction> {
    let mut action = None;
    let mut tile_rect = egui::Rect::NOTHING;
    let response = egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(12.0)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            // Every card is the same width; the grid wraps them to fit the window.
            ui.set_width(CARD_WIDTH - 24.0);
            // Every card is the same height, so the cards in a row line up.
            ui.set_min_height(CARD_HEIGHT - 24.0);
            // The content is centred vertically within the card's fixed height.
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), CARD_HEIGHT - 24.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(42.0, 42.0), Sense::hover());
                    tile_rect = rect;
                    paint_tile(ui.painter(), rect, station);
                    ui.add_space(10.0);
                    // The text is centred vertically against the tile, within the card's height.
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), CARD_HEIGHT - 24.0),
                        egui::Layout::top_down(egui::Align::Min)
                            .with_main_align(egui::Align::Center),
                        |ui| {
                            // Nudged down: the line boxes sit high against the tile.
                            ui.add_space(4.0);
                            ui.add(
                                egui::Label::new(egui::RichText::new(&station.name).strong())
                                    .truncate(),
                            );
                            let mut details = card_details(station);
                            if tuned {
                                details.push_str("  ·  Tuned in");
                            }
                            ui.add(
                                egui::Label::new(egui::RichText::new(details).weak()).truncate(),
                            );
                        },
                    );
                },
            );
        })
        .response;

    if ui.rect_contains_pointer(response.rect) {
        let accent = ui.visuals().selection.bg_fill;
        let click = ui.interact(
            tile_rect,
            ui.scope_id().with(("result_play", &station.uuid)),
            Sense::click(),
        );
        ui.painter().rect_filled(tile_rect, 10.0, accent);
        icons::paint(
            ui.painter(),
            tile_rect.center(),
            icons::PLAY,
            18.0,
            egui::Color32::WHITE,
        );
        if click.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if click.clicked() {
            action = Some(RadioAction::TuneInAndPlay(station.clone()));
        }
    }
    action
}

/// Country code (or name), codec and bitrate for a result card.
fn card_details(station: &Station) -> String {
    let mut parts = Vec::new();
    if !station.country_code.is_empty() {
        parts.push(station.country_code.clone());
    } else if !station.country.is_empty() {
        parts.push(station.country.clone());
    }
    if !station.codec.is_empty() {
        parts.push(station.codec.clone());
    }
    if station.bitrate > 0 {
        parts.push(format!("{} kbps", station.bitrate));
    }
    if parts.is_empty() {
        "Internet radio".into()
    } else {
        parts.join("  ·  ")
    }
}
