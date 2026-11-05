use std::sync::Arc;
use std::time::Duration;

use eframe::egui;
use player_core::MediaItem;
use player_playback::{Player, PlayerState, RodioPlayer};

use crate::format;

/// The queue the user is listening through, and the player that plays it.
pub struct Playback {
    /// `None` when no audio output device could be opened.
    player: Option<RodioPlayer>,
    queue: Vec<Arc<MediaItem>>,
    index: usize,
    /// Slider position in seconds. Follows the player unless the user is dragging.
    seek_secs: f64,
    dragging: bool,
    error: Option<String>,
    /// Measured widths of the centred rows, from the previous frame.
    row_widths: [f32; 3],
}

/// What the bar asked for this frame. Applied after drawing.
enum Command {
    Toggle,
    Next,
    Previous,
    Seek(f64),
}

impl Playback {
    pub fn new() -> Self {
        let (player, error) = match RodioPlayer::new() {
            Ok(player) => (Some(player), None),
            Err(e) => (None, Some(format!("No audio output: {e}"))),
        };
        Self {
            player,
            queue: Vec::new(),
            index: 0,
            seek_secs: 0.0,
            dragging: false,
            error,
            row_widths: [0.0; 3],
        }
    }

    /// Id of the track that is loaded, if any.
    pub fn current_id(&self) -> Option<i64> {
        self.current().map(|t| t.id)
    }

    pub fn is_playing(&self) -> bool {
        self.state() == PlayerState::Playing
    }

    /// Whether a track is loaded. The playback bar is shown only then.
    pub fn has_current(&self) -> bool {
        self.current().is_some()
    }

    /// Replace the queue with `tracks` and start playing at `index`.
    pub fn play_queue(&mut self, tracks: Vec<MediaItem>, index: usize) {
        self.queue = tracks.into_iter().map(Arc::new).collect();
        self.index = index.min(self.queue.len().saturating_sub(1));
        self.start_current();
    }

    /// Move on when a track has finished, and say why if it stopped early.
    pub fn advance_if_finished(&mut self) {
        if self.state() != PlayerState::Ended {
            return;
        }
        let stopped_early = self.player.as_ref().and_then(|p| p.last_error());
        if self.index + 1 < self.queue.len() {
            self.next();
        }
        if let Some(message) = stopped_early {
            self.error = Some(format!("Playback stopped early: {message}"));
        }
    }

    fn current(&self) -> Option<&Arc<MediaItem>> {
        self.queue.get(self.index)
    }

    fn state(&self) -> PlayerState {
        self.player
            .as_ref()
            .map_or(PlayerState::Idle, |p| p.state())
    }

    fn position(&self) -> Option<Duration> {
        self.player.as_ref().and_then(|p| p.position())
    }

    /// Total length: from the decoder if known, otherwise from the library.
    fn duration(&self) -> Option<Duration> {
        self.player.as_ref().and_then(|p| p.duration()).or_else(|| {
            self.current()
                .and_then(|t| t.duration_ms)
                .map(|ms| Duration::from_millis(ms.max(0) as u64))
        })
    }

    /// Load and play the current queue item.
    fn start_current(&mut self) {
        let Some(item) = self.current().cloned() else {
            return;
        };
        // The output may not have been available at launch (e.g. headphones connected later).
        if self.player.is_none() {
            self.player = RodioPlayer::new().ok();
        }
        let Some(player) = self.player.as_mut() else {
            self.error = Some("No audio output device available".into());
            return;
        };
        self.error = match player.load(&item.path).and_then(|_| player.play()) {
            Ok(()) => None,
            Err(e) => Some(format!("Could not play {}: {e}", format::title(&item))),
        };
        self.dragging = false;
        self.seek_secs = 0.0;
    }

    fn with_player(&mut self, f: impl FnOnce(&mut RodioPlayer) -> player_playback::Result<()>) {
        if let Some(player) = self.player.as_mut()
            && let Err(e) = f(player)
        {
            self.error = Some(e.to_string());
        }
    }

    fn toggle(&mut self) {
        match self.state() {
            PlayerState::Playing => self.with_player(|p| p.pause()),
            PlayerState::Paused => self.with_player(|p| p.play()),
            // Playing again after the queue finished starts the album from the first track.
            PlayerState::Ended => {
                if !self.queue.is_empty() {
                    self.index = 0;
                    self.start_current();
                }
            }
            PlayerState::Idle => {
                if self.current().is_some() {
                    self.start_current();
                }
            }
        }
    }

    fn next(&mut self) {
        if self.index + 1 < self.queue.len() {
            self.index += 1;
            self.start_current();
        }
    }

    /// Within the first few seconds, go to the previous track. Otherwise restart this one.
    fn previous(&mut self) {
        let past_start = self.position().is_some_and(|p| p > Duration::from_secs(3));
        if past_start || self.index == 0 {
            self.seek(0.0);
        } else {
            self.index -= 1;
            self.start_current();
        }
    }

    fn seek(&mut self, secs: f64) {
        let target = Duration::from_secs_f64(secs.max(0.0));
        self.with_player(|p| p.seek(target));
        self.seek_secs = secs;
    }

    /// Draw the playback bar, centred. Row 1: transport buttons with the track name and
    /// artist. Row 2: seek slider. Row 3: format details.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let current = self.current().cloned();
        let state = self.state();
        let position = self.position().unwrap_or_default();
        let total = self
            .duration()
            .map_or(0.001, |d| d.as_secs_f64().max(0.001));

        if !self.dragging {
            self.seek_secs = position.as_secs_f64().min(total);
        }

        let animate = state == PlayerState::Playing;
        let mut command = None;
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    centred_row(ui, &mut self.row_widths[0], |ui| {
                        if transport_button(ui, Icon::Previous, 32.0)
                            .on_hover_text("Previous track")
                            .clicked()
                        {
                            command = Some(Command::Previous);
                        }
                        let (icon, tip) = if state == PlayerState::Playing {
                            (Icon::Pause, "Pause")
                        } else {
                            (Icon::Play, "Play")
                        };
                        if transport_button(ui, icon, 36.0)
                            .on_hover_text(tip)
                            .clicked()
                        {
                            command = Some(Command::Toggle);
                        }
                        if transport_button(ui, Icon::Next, 32.0)
                            .on_hover_text("Next track")
                            .clicked()
                        {
                            command = Some(Command::Next);
                        }
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            let (title_color, artist_color) = (
                                ui.visuals().strong_text_color(),
                                ui.visuals().weak_text_color(),
                            );
                            match &current {
                                Some(item) => {
                                    marquee(
                                        ui,
                                        INFO_WIDTH,
                                        &format::title(item),
                                        egui::FontId::proportional(14.0),
                                        title_color,
                                        animate,
                                    );
                                    let artist = item
                                        .artist
                                        .as_deref()
                                        .or(item.album_artist.as_deref())
                                        .unwrap_or_default();
                                    marquee(
                                        ui,
                                        INFO_WIDTH,
                                        artist,
                                        egui::FontId::proportional(12.0),
                                        artist_color,
                                        animate,
                                    );
                                }
                                None => {
                                    marquee(
                                        ui,
                                        INFO_WIDTH,
                                        "Nothing playing",
                                        egui::FontId::proportional(12.0),
                                        artist_color,
                                        animate,
                                    );
                                }
                            }
                        });
                    });

                    let mut secs = self.seek_secs;
                    let mut dragging = self.dragging;
                    let mut seek_to = None;
                    let width = ui.available_width().min(560.0);
                    centred_row(ui, &mut self.row_widths[1], |ui| {
                        ui.add_sized(
                            [48.0, 18.0],
                            egui::Label::new(
                                egui::RichText::new(format::clock(secs)).small().weak(),
                            ),
                        );
                        ui.spacing_mut().slider_width = width - 110.0;
                        let response = ui.add(
                            egui::Slider::new(&mut secs, 0.0..=total)
                                .show_value(false)
                                .trailing_fill(true),
                        );
                        ui.add_sized(
                            [48.0, 18.0],
                            egui::Label::new(
                                egui::RichText::new(format::clock(total)).small().weak(),
                            ),
                        );
                        dragging = response.dragged();
                        // Seek once the drag ends, or on a click (a change without a drag).
                        if response.drag_stopped() || (response.changed() && !response.dragged()) {
                            seek_to = Some(secs);
                        }
                    });
                    self.seek_secs = secs;
                    self.dragging = dragging;
                    if let Some(target) = seek_to {
                        command = Some(Command::Seek(target));
                    }

                    if let Some(item) = &current {
                        ui.add_space(4.0);
                        centred_row(ui, &mut self.row_widths[2], |ui| {
                            if let Some(codec) = item.codec.as_deref() {
                                chip(ui, codec);
                            }
                            if let Some(format) =
                                format::sample_format(item.bit_depth, item.sample_rate)
                            {
                                chip(ui, &format);
                            }
                            if let Some(kbps) = item.bitrate_kbps {
                                chip(ui, &format::kbps(kbps));
                            }
                            if let Some(channels) = item.channels {
                                chip(ui, &format::channels(channels));
                            }
                            if format::is_hi_res(item.bit_depth, item.sample_rate) {
                                hi_res_badge(ui);
                            }
                        });
                    }

                    ui.add_space(6.0);
                    if let Some(error) = self.error.as_deref() {
                        ui.colored_label(ui.visuals().error_fg_color, error);
                    }
                });
            });

        match command {
            Some(Command::Toggle) => self.toggle(),
            Some(Command::Next) => self.next(),
            Some(Command::Previous) => self.previous(),
            Some(Command::Seek(target)) => self.seek(target),
            None => {}
        }
    }
}

/// Outlined label for one format detail.
fn chip(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.small(text);
        });
}

/// Green "Hi-res" badge, with colours chosen for the current theme.
fn hi_res_badge(ui: &mut egui::Ui) {
    let (fill, text) = if ui.visuals().dark_mode {
        (
            egui::Color32::from_rgb(24, 58, 40),
            egui::Color32::from_rgb(140, 220, 170),
        )
    } else {
        (
            egui::Color32::from_rgb(222, 243, 228),
            egui::Color32::from_rgb(28, 98, 54),
        )
    };
    egui::Frame::new()
        .fill(fill)
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Hi-res").small().color(text));
        });
}

/// Lay out `add` in a horizontal row, centred in the available width. The row's width
/// comes from the previous frame, so the first frame may be a few pixels off.
fn centred_row(ui: &mut egui::Ui, width: &mut f32, add: impl FnOnce(&mut egui::Ui)) {
    let pad = ((ui.available_width() - *width) / 2.0).max(0.0);
    *width = ui
        .horizontal(|ui| {
            ui.add_space(pad);
            ui.scope(add).response.rect.width()
        })
        .inner;
}

/// Icon for a transport button.
#[derive(Clone, Copy)]
enum Icon {
    Previous,
    Play,
    Pause,
    Next,
}

/// Round or rounded button with a transport icon drawn by the painter, so no font glyphs are needed.
fn transport_button(ui: &mut egui::Ui, icon: Icon, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    let radius = match icon {
        Icon::Play | Icon::Pause => size / 2.0,
        Icon::Previous | Icon::Next => 6.0,
    };
    let fill = visuals.weak_bg_fill;
    let color = visuals.fg_stroke.color;
    let painter = ui.painter();
    painter.rect_filled(rect, radius, fill);

    let c = rect.center();
    let bar = |x0: f32, x1: f32| {
        egui::Rect::from_min_max(egui::pos2(x0, c.y - 6.0), egui::pos2(x1, c.y + 6.0))
    };
    let triangle = |tip: f32, base: f32| {
        egui::Shape::convex_polygon(
            vec![
                egui::pos2(base, c.y - 6.0),
                egui::pos2(base, c.y + 6.0),
                egui::pos2(tip, c.y),
            ],
            color,
            egui::Stroke::NONE,
        )
    };
    match icon {
        Icon::Previous => {
            painter.rect_filled(bar(c.x - 7.0, c.x - 5.0), 0.0, color);
            painter.add(triangle(c.x - 5.0, c.x + 5.0));
        }
        Icon::Next => {
            painter.rect_filled(bar(c.x + 5.0, c.x + 7.0), 0.0, color);
            painter.add(triangle(c.x + 5.0, c.x - 5.0));
        }
        Icon::Play => {
            painter.add(triangle(c.x + 6.0, c.x - 3.0));
        }
        Icon::Pause => {
            painter.rect_filled(bar(c.x - 4.0, c.x - 1.5), 0.0, color);
            painter.rect_filled(bar(c.x + 1.5, c.x + 4.0), 0.0, color);
        }
    }
    response
}

/// Fixed width of the track name and artist column.
const INFO_WIDTH: f32 = 300.0;

/// Scroll speed of a line that is wider than the column, in points per second.
const SCROLL_SPEED: f64 = 40.0;

/// Gap between repeats of a scrolling line.
const SCROLL_GAP: f32 = 48.0;

/// One line of text in a fixed-width box. A short line is centred. A line wider than the
/// box scrolls in a loop, and the window keeps redrawing while it does.
fn marquee(
    ui: &mut egui::Ui,
    width: f32,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    animate: bool,
) {
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, color);
    let text_width = galley.size().x;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(width, galley.size().y), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    if text_width <= width || !animate {
        let x = rect.left();
        painter.galley(egui::pos2(x, rect.top()), galley, color);
    } else {
        let period = text_width + SCROLL_GAP;
        let time = ui.input(|i| i.time);
        let offset = ((time * SCROLL_SPEED) % f64::from(period)) as f32;
        let x = rect.left() - offset;
        painter.galley(egui::pos2(x, rect.top()), galley.clone(), color);
        painter.galley(egui::pos2(x + period, rect.top()), galley, color);
        ui.ctx().request_repaint();
    }
}
