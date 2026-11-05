use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use eframe::egui;
use player_core::MediaItem;
use player_playback::{Player, PlayerState, RodioPlayer};

use crate::format;

/// What repeat does when a track or the queue ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repeat {
    Off,
    /// Repeat the whole queue.
    Queue,
    /// Repeat the current track.
    Track,
}

impl Repeat {
    fn cycle(self) -> Self {
        match self {
            Self::Off => Self::Queue,
            Self::Queue => Self::Track,
            Self::Track => Self::Off,
        }
    }
}

/// A track to add to the queue from elsewhere in the app.
pub enum QueueRequest {
    PlayNext(MediaItem),
    AddToQueue(MediaItem),
}

/// The queue the user is listening through, and the player that plays it.
pub struct Playback {
    /// `None` when no audio output device could be opened.
    player: Option<RodioPlayer>,
    queue: Vec<Arc<MediaItem>>,
    /// The queue in its natural order. Shuffle-off puts the upcoming tracks back in it.
    original: Vec<Arc<MediaItem>>,
    index: usize,
    shuffle: bool,
    repeat: Repeat,
    /// Output level from 0 to 1, applied on top of the system volume.
    volume: f32,
    muted: bool,
    /// Slider position in seconds. Follows the player unless the user is dragging.
    seek_secs: f64,
    dragging: bool,
    error: Option<String>,
    /// Measured widths of the centred rows, from the previous frame.
    row_widths: [f32; 3],
    /// Whether the queue panel is open. The bar's queue button toggles it.
    pub queue_open: bool,
}

/// What the bar asked for this frame. Applied after drawing.
enum Command {
    Toggle,
    Next,
    Previous,
    Seek(f64),
    SetVolume(f32),
    ToggleMute,
    ToggleShuffle,
    CycleRepeat,
    ToggleQueue,
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
            original: Vec::new(),
            index: 0,
            shuffle: false,
            repeat: Repeat::Off,
            volume: 1.0,
            muted: false,
            seek_secs: 0.0,
            dragging: false,
            error,
            row_widths: [0.0; 3],
            queue_open: false,
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

    pub fn state(&self) -> PlayerState {
        self.player
            .as_ref()
            .map_or(PlayerState::Idle, |p| p.state())
    }

    pub fn position(&self) -> Option<Duration> {
        self.player.as_ref().and_then(|p| p.position())
    }

    /// Total length: from the decoder if known, otherwise from the library.
    pub fn duration(&self) -> Option<Duration> {
        self.player.as_ref().and_then(|p| p.duration()).or_else(|| {
            self.current()
                .and_then(|t| t.duration_ms)
                .map(|ms| Duration::from_millis(ms.max(0) as u64))
        })
    }

    pub fn queue(&self) -> &[Arc<MediaItem>] {
        &self.queue
    }

    pub fn index(&self) -> usize {
        self.index
    }

    /// Replace the queue with `tracks` and start playing at `index`. With shuffle on, the
    /// tracks after `index` are shuffled.
    pub fn play_queue(&mut self, tracks: Vec<MediaItem>, index: usize) {
        let items: Vec<Arc<MediaItem>> = tracks.into_iter().map(Arc::new).collect();
        self.original = items.clone();
        self.queue = items;
        self.index = index.min(self.queue.len().saturating_sub(1));
        if self.shuffle {
            self.shuffle_upcoming();
        }
        self.start_current();
    }

    /// Put `item` straight after the current track.
    pub fn play_next(&mut self, item: MediaItem) {
        if self.queue.is_empty() {
            self.play_queue(vec![item], 0);
            return;
        }
        let item = Arc::new(item);
        self.queue.insert(self.index + 1, item.clone());
        // Keep the natural order in step, so turning shuffle off keeps this track in place.
        let current = self.queue[self.index].clone();
        let at = self
            .original
            .iter()
            .position(|o| Arc::ptr_eq(o, &current))
            .map_or(self.original.len(), |p| p + 1);
        self.original.insert(at, item);
    }

    /// Add `item` to the end of the queue.
    pub fn add_to_queue(&mut self, item: MediaItem) {
        if self.queue.is_empty() {
            self.play_queue(vec![item], 0);
        } else {
            let item = Arc::new(item);
            self.original.push(item.clone());
            self.queue.push(item);
        }
    }

    /// Remove the queue entry at `i`. The track that is playing can't be removed.
    pub fn remove_at(&mut self, i: usize) {
        if i >= self.queue.len() || i == self.index {
            return;
        }
        self.queue.remove(i);
        if i < self.index {
            self.index -= 1;
        }
        self.prune_original();
    }

    /// Move the upcoming entry at `from` so it sits just before the entry at `before`
    /// (`before` may be the queue length, to move it to the end). Only the queue changes.
    pub fn move_upcoming(&mut self, from: usize, before: usize) {
        let first = self.index + 1;
        let len = self.queue.len();
        if from < first || from >= len || before < first || before > len {
            return;
        }
        if before == from || before == from + 1 {
            return;
        }
        let before_item = self.queue.get(before).cloned();
        let item = self.queue.remove(from);
        let dest = if before > from { before - 1 } else { before };
        self.queue.insert(dest, item.clone());

        // Keep the natural order in step, so that turning shuffle off doesn't undo the move.
        if let Some(p) = self.original.iter().position(|o| Arc::ptr_eq(o, &item)) {
            let moved = self.original.remove(p);
            let dest = before_item
                .and_then(|b| self.original.iter().position(|o| Arc::ptr_eq(o, &b)))
                .unwrap_or(self.original.len());
            self.original.insert(dest, moved);
        }
    }

    /// Remove every entry after the current track. Only the queue changes.
    pub fn clear_upcoming(&mut self) {
        self.queue.truncate(self.index + 1);
        self.prune_original();
    }

    /// Drop entries from the natural order that are no longer in the queue.
    fn prune_original(&mut self) {
        let live: HashSet<*const MediaItem> = self.queue.iter().map(Arc::as_ptr).collect();
        self.original.retain(|o| live.contains(&Arc::as_ptr(o)));
    }

    /// Start playing the queue entry at `i`.
    pub fn jump_to(&mut self, i: usize) {
        if i < self.queue.len() {
            self.index = i;
            self.start_current();
        }
    }

    pub fn set_shuffle(&mut self, on: bool) {
        if on == self.shuffle {
            return;
        }
        self.shuffle = on;
        if on {
            self.shuffle_upcoming();
        } else {
            self.restore_order();
        }
    }

    pub fn cycle_repeat(&mut self) {
        self.repeat = self.repeat.cycle();
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        self.apply_volume();
    }

    /// Change the volume by `delta` (a fraction, so 0.05 is 5 percent).
    pub fn change_volume(&mut self, delta: f32) {
        self.set_volume(self.volume + delta);
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        self.apply_volume();
    }

    /// Move on when a track has finished, and say why if it stopped early.
    pub fn advance_if_finished(&mut self) {
        if self.state() != PlayerState::Ended {
            return;
        }
        let stopped_early = self.player.as_ref().and_then(|p| p.last_error());
        match self.repeat {
            // A track that stopped early isn't replayed, or a damaged file would loop.
            Repeat::Track if stopped_early.is_none() => self.start_current(),
            _ => self.next(),
        }
        if let Some(message) = stopped_early {
            self.error = Some(format!("Playback stopped early: {message}"));
        }
    }

    pub fn play(&mut self) {
        match self.state() {
            PlayerState::Playing => {}
            PlayerState::Paused => self.with_player(|p| p.play()),
            PlayerState::Ended | PlayerState::Idle => self.toggle(),
        }
    }

    pub fn pause(&mut self) {
        if self.is_playing() {
            self.with_player(|p| p.pause());
        }
    }

    pub fn toggle(&mut self) {
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

    pub fn next(&mut self) {
        if self.index + 1 < self.queue.len() {
            self.index += 1;
        } else if self.repeat == Repeat::Queue && !self.queue.is_empty() {
            self.index = 0;
        } else {
            return;
        }
        self.start_current();
    }

    /// Within the first few seconds, go to the previous track. Otherwise restart this one.
    pub fn previous(&mut self) {
        let past_start = self.position().is_some_and(|p| p > Duration::from_secs(3));
        if past_start || self.index == 0 {
            self.seek(0.0);
        } else {
            self.index -= 1;
            self.start_current();
        }
    }

    /// Move `delta` seconds forward (positive) or back (negative), within the track.
    pub fn seek_relative(&mut self, delta: f64) {
        let now = self.position().map_or(0.0, |p| p.as_secs_f64());
        let total = self.duration().map_or(f64::MAX, |d| d.as_secs_f64());
        self.seek((now + delta).clamp(0.0, total));
    }

    pub fn stop(&mut self) {
        self.with_player(|p| p.stop());
    }

    /// The track that is loaded, for the system's now-playing display.
    pub fn current_item(&self) -> Option<Arc<MediaItem>> {
        self.current().cloned()
    }

    fn current(&self) -> Option<&Arc<MediaItem>> {
        self.queue.get(self.index)
    }

    /// Shuffle the tracks after the current one.
    fn shuffle_upcoming(&mut self) {
        let start = self.index + 1;
        if start < self.queue.len() {
            shuffle(&mut self.queue[start..]);
        }
    }

    /// Put the tracks after the current one back in their natural order.
    fn restore_order(&mut self) {
        let mut tail = self.queue.split_off((self.index + 1).min(self.queue.len()));
        let original = &self.original;
        tail.sort_by_key(|item| {
            original
                .iter()
                .position(|o| Arc::ptr_eq(o, item))
                .unwrap_or(usize::MAX)
        });
        self.queue.extend(tail);
    }

    /// Apply the output level: zero when muted, otherwise the chosen volume.
    fn apply_volume(&mut self) {
        let level = if self.muted { 0.0 } else { self.volume };
        self.with_player(|p| p.set_volume(level));
    }

    pub fn seek(&mut self, secs: f64) {
        let target = Duration::from_secs_f64(secs.max(0.0));
        self.with_player(|p| p.seek(target));
        self.seek_secs = secs;
    }

    fn with_player(&mut self, f: impl FnOnce(&mut RodioPlayer) -> player_playback::Result<()>) {
        if let Some(player) = self.player.as_mut()
            && let Err(e) = f(player)
        {
            self.error = Some(e.to_string());
        }
    }

    /// Load and play the current queue item.
    fn start_current(&mut self) {
        let Some(item) = self.current().cloned() else {
            return;
        };
        // The output may not have been available at launch (e.g. headphones connected later).
        if self.player.is_none() {
            self.player = RodioPlayer::new().ok();
            self.apply_volume();
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

    /// Draw the playback bar, centred. Row 1: shuffle, transport and repeat, with the track name
    /// and artist. Row 2: seek slider, with mute and volume. Row 3: format details.
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
                        if mode_button(ui, ModeIcon::Shuffle, self.shuffle)
                            .on_hover_text("Shuffle")
                            .clicked()
                        {
                            command = Some(Command::ToggleShuffle);
                        }
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
                        if mode_button(
                            ui,
                            ModeIcon::Repeat(self.repeat),
                            self.repeat != Repeat::Off,
                        )
                        .on_hover_text("Repeat")
                        .clicked()
                        {
                            command = Some(Command::CycleRepeat);
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
                    let mut volume_level = self.volume;
                    let mut volume_to = None;
                    let width = ui.available_width().min(560.0);
                    centred_row(ui, &mut self.row_widths[1], |ui| {
                        ui.add_sized(
                            [48.0, 18.0],
                            egui::Label::new(
                                egui::RichText::new(format::clock(secs)).small().weak(),
                            ),
                        );
                        ui.spacing_mut().slider_width = width - 110.0 - 170.0;
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

                        ui.add_space(8.0);
                        let speaker = speaker_button(ui, self.muted);
                        if speaker
                            .on_hover_text(if self.muted { "Unmute" } else { "Mute" })
                            .clicked()
                        {
                            command = Some(Command::ToggleMute);
                        }
                        ui.spacing_mut().slider_width = 80.0;
                        let volume = ui
                            .add(egui::Slider::new(&mut volume_level, 0.0..=1.0).show_value(false));
                        if volume.changed() {
                            volume_to = Some(volume_level);
                        }
                        ui.add_space(4.0);
                        if queue_button(ui, self.queue_open)
                            .on_hover_text("Queue")
                            .clicked()
                        {
                            command = Some(Command::ToggleQueue);
                        }
                    });
                    self.seek_secs = secs;
                    self.dragging = dragging;
                    if let Some(target) = seek_to {
                        command = Some(Command::Seek(target));
                    }
                    if let Some(level) = volume_to {
                        command = Some(Command::SetVolume(level));
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
            Some(Command::SetVolume(level)) => self.set_volume(level),
            Some(Command::ToggleMute) => self.toggle_mute(),
            Some(Command::ToggleShuffle) => self.set_shuffle(!self.shuffle),
            Some(Command::CycleRepeat) => self.cycle_repeat(),
            Some(Command::ToggleQueue) => self.queue_open = !self.queue_open,
            None => {}
        }
    }
}

/// Playback keys. Space plays or pauses, Left and Right seek 10 seconds, Up and Down change the
/// volume, M mutes, and N and P go to the next and previous track. Keys are ignored while text
/// is being typed.
pub fn handle_keys(ctx: &egui::Context, playback: &mut Playback) {
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    let none = egui::Modifiers::NONE;
    ctx.input_mut(|input| {
        if input.consume_key(none, egui::Key::Space) {
            playback.toggle();
        }
        if input.consume_key(none, egui::Key::ArrowLeft) {
            playback.seek_relative(-10.0);
        }
        if input.consume_key(none, egui::Key::ArrowRight) {
            playback.seek_relative(10.0);
        }
        if input.consume_key(none, egui::Key::ArrowUp) {
            playback.change_volume(0.05);
        }
        if input.consume_key(none, egui::Key::ArrowDown) {
            playback.change_volume(-0.05);
        }
        if input.consume_key(none, egui::Key::M) {
            playback.toggle_mute();
        }
        if input.consume_key(none, egui::Key::N) {
            playback.next();
        }
        if input.consume_key(none, egui::Key::P) {
            playback.previous();
        }
    });
}

/// Shuffle `items` in place (Fisher-Yates). The generator is seeded from the clock; nothing
/// here needs real randomness, so no extra crate is used.
pub fn shuffle<T>(items: &mut [T]) {
    let mut state = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E37_79B9_7F4A_7C15)
        | 1;
    for i in (1..items.len()).rev() {
        // xorshift64
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let j = (state % (i as u64 + 1)) as usize;
        items.swap(i, j);
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

/// Icon for a mode button: shuffle, or repeat with its current mode.
#[derive(Clone, Copy)]
enum ModeIcon {
    Shuffle,
    Repeat(Repeat),
}

/// Small round button for shuffle and repeat. Its icon uses the accent colour when `active`.
fn mode_button(ui: &mut egui::Ui, icon: ModeIcon, active: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    let fill = visuals.weak_bg_fill;
    let color = if active {
        ui.visuals().hyperlink_color
    } else {
        visuals.fg_stroke.color
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 14.0, fill);
    let c = rect.center();
    let stroke = egui::Stroke::new(1.4, color);
    match icon {
        ModeIcon::Shuffle => {
            let (left, right) = (c.x - 6.0, c.x + 6.0);
            let (top, bottom) = (c.y - 5.0, c.y + 5.0);
            painter.line_segment([egui::pos2(left, bottom), egui::pos2(right, top)], stroke);
            painter.line_segment([egui::pos2(left, top), egui::pos2(right, bottom)], stroke);
        }
        ModeIcon::Repeat(mode) => {
            let body = egui::Rect::from_center_size(c, egui::vec2(13.0, 10.0));
            painter.rect_stroke(body, 3.0, stroke, egui::StrokeKind::Inside);
            if mode == Repeat::Track {
                painter.text(
                    c,
                    egui::Align2::CENTER_CENTER,
                    "1",
                    egui::FontId::proportional(9.0),
                    color,
                );
            }
        }
    }
    response
}

/// Round button with a list icon. It opens and closes the queue panel.
fn queue_button(ui: &mut egui::Ui, active: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    let color = if active {
        ui.visuals().hyperlink_color
    } else {
        visuals.fg_stroke.color
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 14.0, visuals.weak_bg_fill);
    let c = rect.center();
    for dy in [-5.0, 0.0, 5.0] {
        painter.circle_filled(egui::pos2(c.x - 5.0, c.y + dy), 1.2, color);
        painter.line_segment(
            [
                egui::pos2(c.x - 2.0, c.y + dy),
                egui::pos2(c.x + 6.0, c.y + dy),
            ],
            egui::Stroke::new(1.4, color),
        );
    }
    response
}

/// Speaker icon for mute: waves when sound is on, a cross when muted.
fn speaker_button(ui: &mut egui::Ui, muted: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    let color = visuals.fg_stroke.color;
    let painter = ui.painter();
    let c = rect.center();
    let stroke = egui::Stroke::new(1.4, color);
    painter.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(c.x - 7.0, c.y - 2.5),
            egui::pos2(c.x - 3.0, c.y - 2.5),
            egui::pos2(c.x + 1.0, c.y - 6.0),
            egui::pos2(c.x + 1.0, c.y + 6.0),
            egui::pos2(c.x - 3.0, c.y + 2.5),
            egui::pos2(c.x - 7.0, c.y + 2.5),
        ],
        color,
        egui::Stroke::NONE,
    ));
    if muted {
        painter.line_segment(
            [
                egui::pos2(c.x + 4.0, c.y - 4.0),
                egui::pos2(c.x + 9.0, c.y + 4.0),
            ],
            stroke,
        );
        painter.line_segment(
            [
                egui::pos2(c.x + 4.0, c.y + 4.0),
                egui::pos2(c.x + 9.0, c.y - 4.0),
            ],
            stroke,
        );
    } else {
        painter.line_segment(
            [
                egui::pos2(c.x + 4.0, c.y - 3.0),
                egui::pos2(c.x + 4.0, c.y + 3.0),
            ],
            stroke,
        );
        painter.line_segment(
            [
                egui::pos2(c.x + 7.0, c.y - 5.0),
                egui::pos2(c.x + 7.0, c.y + 5.0),
            ],
            stroke,
        );
    }
    response
}

/// Fixed width of the track name and artist column.
const INFO_WIDTH: f32 = 300.0;

/// Scroll speed of a line that is wider than the column, in points per second.
const SCROLL_SPEED: f64 = 40.0;

/// Gap between repeats of a scrolling line.
const SCROLL_GAP: f32 = 48.0;

/// One line of text in a fixed-width box. A short line is left-aligned. A line wider than the
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn item(id: i64) -> MediaItem {
        MediaItem {
            id,
            path: PathBuf::from(format!("/music/{id}.flac")),
            kind: player_core::MediaKind::Audio,
            size: 0,
            duration_ms: None,
            title: Some(format!("Track {id}")),
            artist: None,
            album: None,
            album_artist: None,
            composer: None,
            genre: None,
            codec: None,
            bit_depth: None,
            sample_rate: None,
            channels: None,
            bitrate_kbps: None,
            track: None,
            year: None,
        }
    }

    /// Ids of the queue, in order.
    fn ids(p: &Playback) -> Vec<i64> {
        p.queue().iter().map(|t| t.id).collect()
    }

    /// A playback with no audio output. The queue logic works without one.
    fn queue_of(n: i64) -> Playback {
        let mut p = Playback {
            player: None,
            queue: Vec::new(),
            original: Vec::new(),
            index: 0,
            shuffle: false,
            repeat: Repeat::Off,
            volume: 1.0,
            muted: false,
            seek_secs: 0.0,
            dragging: false,
            error: None,
            row_widths: [0.0; 3],
            queue_open: false,
        };
        p.play_queue((1..=n).map(item).collect(), 0);
        p
    }

    #[test]
    fn move_upcoming_places_the_entry_before_the_target() {
        let mut p = queue_of(5);
        p.move_upcoming(1, 3); // 2 goes before 4, which sits at index 3
        assert_eq!(ids(&p), vec![1, 3, 2, 4, 5]);
        p.move_upcoming(4, 1); // 5 goes right after the current track
        assert_eq!(ids(&p), vec![1, 5, 3, 2, 4]);
        p.move_upcoming(2, 5); // 3 goes to the end
        assert_eq!(ids(&p), vec![1, 5, 2, 4, 3]);
    }

    #[test]
    fn moves_that_change_nothing_and_moves_of_the_current_track_are_ignored() {
        let mut p = queue_of(3);
        p.move_upcoming(1, 1);
        p.move_upcoming(1, 2);
        p.move_upcoming(0, 2);
        assert_eq!(ids(&p), vec![1, 2, 3]);
    }

    #[test]
    fn remove_and_clear_only_touch_upcoming_entries() {
        let mut p = queue_of(4);
        p.remove_at(0); // the current track can't be removed
        p.remove_at(2);
        assert_eq!(ids(&p), vec![1, 2, 4]);
        p.clear_upcoming();
        assert_eq!(ids(&p), vec![1]);
    }

    #[test]
    fn play_next_and_add_keep_their_place_when_shuffle_is_turned_off() {
        let mut p = queue_of(3);
        p.play_next(item(10));
        p.add_to_queue(item(20));
        assert_eq!(ids(&p), vec![1, 10, 2, 3, 20]);
        p.set_shuffle(true);
        p.set_shuffle(false);
        assert_eq!(ids(&p), vec![1, 10, 2, 3, 20]);
    }

    #[test]
    fn manual_order_survives_a_shuffle_round_trip() {
        let mut p = queue_of(4);
        p.move_upcoming(3, 1); // 4 goes first after 1
        p.set_shuffle(true);
        p.set_shuffle(false);
        assert_eq!(ids(&p), vec![1, 4, 2, 3]);
    }

    #[test]
    fn shuffle_keeps_the_current_track_and_every_entry() {
        let mut p = queue_of(20);
        p.set_shuffle(true);
        let mut got = ids(&p);
        assert_eq!(got[0], 1);
        got.sort_unstable();
        assert_eq!(got, (1..=20).collect::<Vec<_>>());
    }
}
