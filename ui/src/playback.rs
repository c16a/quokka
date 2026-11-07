use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use eframe::egui;
use player_core::RepeatMode as Repeat;
use player_core::Station;
use player_core::{MediaItem, MediaKind, ReplayGainMode, Session};
use player_playback::{
    Player, PlayerState, RodioPlayer, SymphoniaSource, VideoPlayer, open_stream,
    output_device_names,
};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

use crate::format;
use crate::icons;
use crate::slider;

/// A track to add to the queue from elsewhere in the app.
pub enum MenuRequest {
    PlayNext(MediaItem),
    AddToQueue(MediaItem),
    /// Show a track in the track info panel.
    ShowInfo(MediaItem),
    /// Append a track to a saved playlist.
    AddToPlaylist {
        playlist: i64,
        media_id: i64,
    },
}

/// The side panel open beside the main view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Panel {
    #[default]
    Closed,
    Queue,
    Lyrics,
    Info,
}

/// The queue the user is listening through, and the player that plays it.
const RADIO_RECONNECTS: u32 = 5;

pub struct Playback {
    /// `None` when no audio output device could be opened.
    player: Option<RodioPlayer>,
    /// The libmpv output for video. Created the first time a video plays.
    video: Option<VideoPlayer>,
    /// Whether the loaded track plays through `video` instead of `player`.
    video_loaded: bool,
    queue: Vec<Arc<MediaItem>>,
    /// The queue in its natural order. Shuffle-off puts the upcoming tracks back in it.
    original: Vec<Arc<MediaItem>>,
    index: usize,
    shuffle: bool,
    repeat: Repeat,
    /// Output level from 0 to 1, applied on top of the system volume.
    volume: f32,
    muted: bool,
    /// Playback speed and crossfade length, as last applied from the settings.
    speed: f32,
    crossfade: f32,
    /// The width the buttons on the right of the bar took, measured when they were drawn.
    right_width: f32,
    /// A speed chosen in the bar, for the app to save to the settings.
    speed_change: Option<f32>,
    /// The album whose name was clicked in the bar, for the app to open.
    album_request: Option<String>,
    /// The library database, where measured loudness is kept.
    library: Option<PathBuf>,
    /// Files asked to be measured, so each is asked once.
    measuring: HashSet<PathBuf>,
    /// The background measurer: the files to measure, and the ones it has finished.
    measure_tx: Option<Sender<PathBuf>>,
    measured_rx: Option<Receiver<PathBuf>>,
    /// Slider position in seconds. Follows the player unless the user is dragging.
    seek_secs: f64,
    dragging: bool,
    error: Option<String>,
    /// Measured widths of the centred rows, from the previous frame.
    row_widths: [f32; 3],
    /// Whether the queue panel is open. The bar's queue button toggles it.
    pub panel: Panel,
    replay_gain: ReplayGainMode,
    /// The output chosen for this session. `None` follows the system default.
    output: Option<String>,
    /// Output names offered in the bar's menu, refreshed when the menu is opened.
    outputs: Vec<String>,
    /// Level factor from the current track's ReplayGain tags. 1.0 when there is none.
    gain: f32,
    /// The queue index of the track armed to follow the current one, if any.
    armed: Option<usize>,
    /// The station being played, when the output is live radio rather than the queue.
    radio: Option<Station>,
    /// A stream being connected to on a background thread.
    radio_loading: Option<Receiver<Result<SymphoniaSource, String>>>,
    /// Whether the user paused the station. A paused station is not reconnected.
    radio_paused: bool,
    /// Reconnects since the user last started the station.
    radio_reconnects: u32,
    /// Whether the output list above the bar is open.
    output_menu: bool,
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
    TogglePanel(Panel),
}

impl Playback {
    pub fn new() -> Self {
        let (player, error) = match RodioPlayer::new() {
            Ok(player) => (Some(player), None),
            Err(e) => (None, Some(format!("No audio output: {e}"))),
        };
        Self {
            player,
            video: None,
            video_loaded: false,
            queue: Vec::new(),
            original: Vec::new(),
            index: 0,
            shuffle: false,
            repeat: Repeat::Off,
            volume: 1.0,
            muted: false,
            speed: 1.0,
            crossfade: 0.0,
            speed_change: None,
            album_request: None,
            right_width: 300.0,
            library: None,
            measuring: HashSet::new(),
            measure_tx: None,
            measured_rx: None,
            seek_secs: 0.0,
            dragging: false,
            error,
            row_widths: [0.0; 3],
            panel: Panel::Closed,
            replay_gain: ReplayGainMode::Off,
            output: None,
            outputs: output_device_names(),
            gain: 1.0,
            armed: None,
            radio: None,
            radio_loading: None,
            radio_paused: false,
            radio_reconnects: 0,
            output_menu: false,
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
        self.active().map_or(PlayerState::Idle, |p| p.state())
    }

    pub fn position(&self) -> Option<Duration> {
        self.active().and_then(|p| p.position())
    }

    /// Total length: from the decoder if known, otherwise from the library.
    pub fn duration(&self) -> Option<Duration> {
        self.active().and_then(|p| p.duration()).or_else(|| {
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

    /// Tune in to a station: connect to its stream and play it. Replaces what was playing.
    pub fn tune(&mut self, station: Station) {
        self.leave_video();
        self.with_player(|p| p.stop());
        self.radio = Some(station);
        self.radio_paused = false;
        self.radio_reconnects = 0;
        self.error = None;
        self.connect_radio();
    }

    /// Whether the bar has something to show: a track, or a station.
    pub fn has_bar(&self) -> bool {
        self.has_current() || self.radio.is_some()
    }

    /// The station playing, if the output is live radio.
    pub fn radio_station(&self) -> Option<&Station> {
        self.radio.as_ref()
    }

    /// Stop the station and forget it.
    pub fn clear_radio(&mut self) {
        if self.radio.is_some() {
            self.with_player(|p| p.stop());
        }
        self.radio = None;
        self.radio_loading = None;
        self.radio_paused = false;
        self.radio_reconnects = 0;
    }

    /// Pause or resume a station. Pausing stops the stream, so resuming joins it live rather
    /// than playing on from where it was paused.
    fn radio_toggle(&mut self) {
        if self.radio_paused {
            self.radio_paused = false;
            self.radio_reconnects = 0;
            self.error = None;
            self.connect_radio();
        } else {
            self.radio_paused = true;
            self.radio_loading = None;
            self.with_player(|p| p.stop());
        }
    }

    /// Connect to the current station's stream on a background thread.
    fn connect_radio(&mut self) {
        let Some(station) = &self.radio else {
            return;
        };
        let url = station.url.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(open_stream(&url));
        });
        self.radio_loading = Some(rx);
    }

    /// Start playing a stream once it has connected.
    fn poll_radio(&mut self) {
        let Some(rx) = self.radio_loading.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(source)) => {
                self.leave_video();
                if self.player.is_none() {
                    self.player = RodioPlayer::new().ok();
                    self.apply_volume();
                }
                let outcome = match self.player.as_mut() {
                    Some(player) => player.load_stream(source).and_then(|_| player.play()),
                    None => Err(player_playback::PlaybackError::Backend(
                        "no audio output device available".into(),
                    )),
                };
                self.error = outcome
                    .err()
                    .map(|e| format!("Could not play the station: {e}"));
            }
            Ok(Err(e)) => {
                self.radio_paused = true;
                self.error = Some(format!("Could not connect to the station: {e}"));
            }
            Err(TryRecvError::Empty) => self.radio_loading = Some(rx),
            Err(TryRecvError::Disconnected) => {
                self.radio_paused = true;
                self.error = Some("Could not connect to the station.".into());
            }
        }
    }

    /// A live stream that ends (the server dropped it) is reconnected, a few times over.
    fn radio_ended(&mut self) {
        if self.radio_paused || self.radio_loading.is_some() || self.state() != PlayerState::Ended {
            return;
        }
        self.radio_reconnects += 1;
        if self.radio_reconnects > RADIO_RECONNECTS {
            self.radio_paused = true;
            self.error =
                Some("The station keeps dropping the connection. Press play to try again.".into());
            return;
        }
        self.connect_radio();
    }

    /// The bar for a station: play or pause, the station's name, and whether it is live.
    fn show_radio(&mut self, ui: &mut egui::Ui) {
        let Some(station) = self.radio.clone() else {
            return;
        };
        let connecting = self.radio_loading.is_some();
        let playing = self.is_playing();
        let mut toggle = false;
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    centred_row(ui, &mut self.row_widths[0], |ui| {
                        let icon = if playing { Icon::Pause } else { Icon::Play };
                        let tip = if playing { "Pause" } else { "Play" };
                        if transport_button(ui, icon, 36.0)
                            .on_hover_text(tip)
                            .clicked()
                        {
                            toggle = true;
                        }
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            ui.label(crate::fonts::bold(&station.name));
                            let status = if connecting {
                                "Connecting…"
                            } else if self.radio_paused {
                                "Paused"
                            } else {
                                "Live"
                            };
                            ui.weak(status);
                        });
                    });
                    if let Some(error) = self.error.as_deref() {
                        ui.add_space(6.0);
                        ui.colored_label(ui.visuals().error_fg_color, error);
                    }
                });
            });
        if toggle {
            self.radio_toggle();
        }
    }

    /// Replace the queue with `tracks` and start playing at `index`. With shuffle on, the
    /// tracks after `index` are shuffled.
    pub fn play_queue(&mut self, tracks: Vec<MediaItem>, index: usize) {
        self.clear_radio();
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
        self.clear_radio();
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

    /// The playback state to save on exit, so it can resume later.
    pub fn session(&self) -> Session {
        Session {
            queue: self.queue.iter().map(|item| item.path.clone()).collect(),
            index: self.index,
            position_secs: self.position().map_or(0.0, |p| p.as_secs_f64()),
            shuffle: self.shuffle,
            repeat: self.repeat,
        }
    }

    /// Put back a saved queue at `index`, paused at `position` seconds into the track.
    pub fn restore(
        &mut self,
        items: Vec<MediaItem>,
        index: usize,
        position: f64,
        shuffle: bool,
        repeat: Repeat,
    ) {
        self.queue = items.into_iter().map(Arc::new).collect();
        self.original = self.queue.clone();
        self.index = index.min(self.queue.len().saturating_sub(1));
        self.shuffle = shuffle;
        self.repeat = repeat;
        self.start(false);
        if position > 0.0 {
            self.seek(position);
        }
    }

    /// Switch the audio output. The current track carries on from the same position, in the
    /// same state (playing, paused or finished).
    pub fn set_output(&mut self, device: Option<String>) {
        if device == self.output && self.player.is_some() {
            return;
        }
        let position = self.position();
        let state = self.state();
        let had_track = self.current().is_some();
        match RodioPlayer::open(device.as_deref()) {
            Ok(player) => {
                // The old player is dropped here, which stops its audio.
                self.player = Some(player);
                self.output = device;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Could not switch output: {e}"));
                return;
            }
        }
        if had_track {
            self.start_current();
            if state != PlayerState::Ended
                && let Some(at) = position
            {
                self.seek(at.as_secs_f64());
            }
            if state != PlayerState::Playing {
                self.pause();
            }
        }
    }

    /// Choose how tracks are levelled with their ReplayGain tags. Applies to the current track too.
    pub fn set_replay_gain(&mut self, mode: ReplayGainMode) {
        self.replay_gain = mode;
        self.gain = match self.current().map(|item| item.path.clone()) {
            Some(path) => self.track_gain(&path),
            None => 1.0,
        };
        self.apply_volume();
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
        self.sync_gapless();
        self.poll_radio();
        if self.radio.is_some() {
            self.radio_ended();
            return;
        }
        if self.state() != PlayerState::Ended {
            return;
        }
        let stopped_early = self.active().and_then(|p| p.last_error());
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
        if self.radio.is_some() {
            if self.radio_paused {
                self.radio_toggle();
            }
            return;
        }
        match self.state() {
            PlayerState::Playing => {}
            PlayerState::Paused => self.with_player(|p| p.play()),
            PlayerState::Ended | PlayerState::Idle => self.toggle(),
        }
    }

    pub fn pause(&mut self) {
        if self.radio.is_some() {
            if !self.radio_paused {
                self.radio_toggle();
            }
            return;
        }
        if self.is_playing() {
            self.with_player(|p| p.pause());
        }
    }

    pub fn toggle(&mut self) {
        if self.radio.is_some() {
            self.radio_toggle();
            return;
        }
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
        if self.radio.is_some() {
            return;
        }
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
        if self.radio.is_some() {
            return;
        }
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

    /// Keep the output one track ahead, so the next track follows without a gap. Called every
    /// frame. When the output has moved on to the armed track, the queue position follows it.
    fn sync_gapless(&mut self) {
        self.poll_measured();
        if self.radio.is_some() || self.video_loaded {
            return;
        }
        let started = match self.player.as_mut() {
            Some(player) => player.take_started().is_some(),
            None => false,
        };
        if started {
            if let Some(index) = self.armed.take() {
                self.index = index;
            }
            self.seek_secs = 0.0;
            self.gain = match self.current().map(|item| item.path.clone()) {
                Some(path) => self.track_gain(&path),
                None => 1.0,
            };
            self.apply_volume();
        }

        let want = self.next_index();
        if want == self.armed {
            return;
        }
        // Only audio is armed for gapless playback. A video needs its own output, so it is
        // opened when it is reached.
        let path = want
            .and_then(|i| self.queue.get(i))
            .filter(|item| item.kind == MediaKind::Audio)
            .map(|item| item.path.clone());
        if let Some(player) = self.player.as_mut() {
            match path {
                Some(path) => {
                    // A track that can't be opened is simply not followed; the normal path then
                    // reports the error when it is reached.
                    let _ = player.arm_next(&path);
                }
                None => player.disarm_next(),
            }
        }
        // Recorded even when opening failed, so the same track isn't retried every frame.
        self.armed = want;
    }

    /// The queue index that plays after the current one, following the repeat mode.
    fn next_index(&self) -> Option<usize> {
        self.current()?;
        match self.repeat {
            Repeat::Track => Some(self.index),
            Repeat::Queue if self.index + 1 >= self.queue.len() => Some(0),
            _ => (self.index + 1 < self.queue.len()).then_some(self.index + 1),
        }
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

    /// Set the library database, where measured loudness is kept.
    pub fn set_library(&mut self, db: PathBuf) {
        self.library = Some(db);
    }

    /// The level change for a track. Its ReplayGain tags come first. A file without them is levelled
    /// by its measured loudness, and a file not measured yet plays at its own level until it is.
    fn track_gain(&mut self, path: &Path) -> f32 {
        if self.replay_gain == ReplayGainMode::Off {
            return 1.0;
        }
        if let Some(gain) = player_core::tag_gain(path, self.replay_gain) {
            return gain;
        }
        if let Some((lufs, peak)) = self.stored_loudness(path) {
            return player_core::loudness_factor(lufs, peak);
        }
        self.request_measure(path);
        1.0
    }

    /// The loudness stored for a file, if it has been measured.
    fn stored_loudness(&self, path: &Path) -> Option<(f64, f64)> {
        let db = self.library.as_ref()?;
        player_core::Library::open(db)
            .ok()?
            .loudness(path)
            .ok()
            .flatten()
    }

    /// Ask the background measurer for a file's loudness, once per file.
    fn request_measure(&mut self, path: &Path) {
        if self.measuring.contains(path) {
            return;
        }
        let Some(db) = self.library.clone() else {
            return;
        };
        if self.measure_tx.is_none() {
            let (requests, done) = spawn_measurer(db);
            self.measure_tx = Some(requests);
            self.measured_rx = Some(done);
        }
        self.measuring.insert(path.to_path_buf());
        if let Some(tx) = &self.measure_tx {
            let _ = tx.send(path.to_path_buf());
        }
    }

    /// Apply the loudness of a file the background measurer has finished, if it is the track
    /// playing now.
    fn poll_measured(&mut self) {
        let Some(done) = &self.measured_rx else {
            return;
        };
        let finished: Vec<PathBuf> = done.try_iter().collect();
        for path in finished {
            if self.current().is_some_and(|item| item.path == path) {
                self.gain = self.track_gain(&path);
                self.apply_volume();
            }
        }
    }

    /// Apply the playback speed and crossfade length from the settings. They reach the player only
    /// when they differ from what was applied last.
    pub fn set_tempo(&mut self, speed: f32, crossfade_secs: f32) {
        if speed == self.speed && crossfade_secs == self.crossfade {
            return;
        }
        self.speed = speed;
        self.crossfade = crossfade_secs;
        self.with_player(|p| {
            p.set_speed(speed)?;
            p.set_crossfade(crossfade_secs)
        });
    }

    /// A speed the user chose in the bar since the last call, if any.
    pub fn take_speed_change(&mut self) -> Option<f32> {
        self.speed_change.take()
    }

    /// The album whose name the user clicked in the bar, if any.
    pub fn take_album_request(&mut self) -> Option<String> {
        self.album_request.take()
    }

    /// Apply the output level: zero when muted, otherwise the chosen volume.
    fn apply_volume(&mut self) {
        let level = if self.muted {
            0.0
        } else {
            self.volume * self.gain
        };
        self.with_player(|p| p.set_volume(level));
    }

    pub fn seek(&mut self, secs: f64) {
        if self.radio.is_some() {
            return;
        }
        let target = Duration::from_secs_f64(secs.max(0.0));
        self.with_player(|p| p.seek(target));
        self.seek_secs = secs;
    }

    /// The output that plays the loaded track: libmpv for video, rodio for audio.
    fn active(&self) -> Option<&dyn Player> {
        if self.video_loaded {
            self.video.as_ref().map(|p| p as &dyn Player)
        } else {
            self.player.as_ref().map(|p| p as &dyn Player)
        }
    }

    fn active_mut(&mut self) -> Option<&mut dyn Player> {
        if self.video_loaded {
            self.video.as_mut().map(|p| p as &mut dyn Player)
        } else {
            self.player.as_mut().map(|p| p as &mut dyn Player)
        }
    }

    fn with_player(&mut self, f: impl FnOnce(&mut dyn Player) -> player_playback::Result<()>) {
        let result = match self.active_mut() {
            Some(player) => f(player),
            None => return,
        };
        if let Err(e) = result {
            self.error = Some(e.to_string());
        }
    }

    /// Stop libmpv if a video was playing, so the audio output can take over.
    fn leave_video(&mut self) {
        if !self.video_loaded {
            return;
        }
        self.video_loaded = false;
        if let Some(video) = self.video.as_mut() {
            let _ = video.stop();
        }
    }

    /// Play a video through libmpv, in its own window. The audio output is released meanwhile.
    fn start_video(&mut self, item: &MediaItem, autoplay: bool) {
        if let Some(player) = self.player.as_mut() {
            let _ = player.stop();
        }
        // Loading a video opens its window. A video that is only queued, such as one restored from
        // the last session, waits until play is pressed, so the app doesn't open a window at launch.
        if !autoplay {
            self.leave_video();
            self.error = None;
            self.gain = self.track_gain(&item.path);
            self.apply_volume();
            self.dragging = false;
            self.seek_secs = 0.0;
            return;
        }
        if self.video.is_none() {
            match VideoPlayer::new() {
                Ok(video) => self.video = Some(video),
                Err(e) => {
                    self.error = Some(format!("Video needs libmpv: {e}"));
                    return;
                }
            }
        }
        self.video_loaded = true;
        let Some(video) = self.video.as_mut() else {
            return;
        };
        let loaded = video
            .load(&item.path)
            .and_then(|_| video.set_speed(self.speed))
            .and_then(|_| if autoplay { video.play() } else { Ok(()) });
        self.error = match loaded {
            Ok(()) => None,
            Err(e) => Some(format!("Could not play {}: {e}", format::title(item))),
        };
        let path = item.path.clone();
        self.gain = self.track_gain(&path);
        self.apply_volume();
        self.dragging = false;
        self.seek_secs = 0.0;
    }

    /// Load and play the current queue item.
    fn start_current(&mut self) {
        self.start(true);
    }

    /// Load the current queue item. It plays when `autoplay`, otherwise it waits paused.
    fn start(&mut self, autoplay: bool) {
        let Some(item) = self.current().cloned() else {
            return;
        };
        self.armed = None;
        if item.kind == MediaKind::Video {
            self.start_video(&item, autoplay);
            return;
        }
        self.leave_video();
        // The output may not have been available at launch (e.g. headphones connected later).
        if self.player.is_none() {
            self.player = RodioPlayer::new().ok();
            self.apply_volume();
        }
        let Some(player) = self.player.as_mut() else {
            self.error = Some("No audio output device available".into());
            return;
        };
        let loaded = player.load(&item.path);
        self.error = match loaded.and_then(|_| if autoplay { player.play() } else { Ok(()) }) {
            Ok(()) => None,
            Err(e) => Some(format!("Could not play {}: {e}", format::title(&item))),
        };
        let path = item.path.clone();
        self.gain = self.track_gain(&path);
        self.apply_volume();
        self.dragging = false;
        self.seek_secs = 0.0;
    }

    /// The playback bar, in this order: shuffle, previous, play or pause, next, repeat, the
    /// track's cover and name, then (after a space) information, lyrics, queue and sound. A seek
    /// slider runs along the top edge, with the elapsed and total time at its ends.
    /// Draw the bar. In `compact` mode, which is for narrow windows, it shows only the track, play or
    /// pause, and next.
    pub fn show(&mut self, ui: &mut egui::Ui, compact: bool) {
        if self.radio.is_some() {
            self.show_radio(ui);
            return;
        }
        let state = self.state();
        let position = self.position().unwrap_or_default();
        let total = self
            .duration()
            .map_or(0.001, |d| d.as_secs_f64().max(0.001));
        if !self.dragging {
            self.seek_secs = position.as_secs_f64().min(total);
        }
        let current = self.current().cloned();
        let (track_text, track_dim) = track_text_colours(ui.visuals().dark_mode);
        if compact {
            let command = show_compact(ui, state, current.as_deref(), track_text, track_dim);
            self.run(command);
            return;
        }

        let mut command = None;
        let mut choice: Option<Option<String>> = None;
        let mut refresh = false;
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(24, 6))
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    // The transport: shuffle, previous, play or pause, next, repeat. It is centred on
                    // the cover's height, so it lines up with the cover.
                    ui.allocate_ui_with_layout(
                        egui::vec2(TRANSPORT_WIDTH, COVER_HEIGHT),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
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
                        },
                    );
                    ui.add_space(12.0);

                    // The cover, the title, and the seek bar beneath them. They fill the space up to
                    // the buttons on the right.
                    let column = (ui.available_width() - self.right_width - 8.0).max(120.0);
                    ui.vertical(|ui| {
                        ui.set_width(column);
                            // Title and artist, centred vertically on the 48px row height. They fill the column.
                            let title_h =
                                ui.fonts_mut(|f| f.row_height(&egui::FontId::proportional(14.0)));
                            let artist_h =
                                ui.fonts_mut(|f| f.row_height(&egui::FontId::proportional(12.0)));
                            let block = title_h + artist_h + ui.spacing().item_spacing.y;
                            let top = ((48.0 - block) / 2.0).max(0.0);
                            ui.allocate_ui_with_layout(
                                egui::vec2(column, 48.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.add_space(top);
                                    ui.vertical(|ui| match &current {
                                        Some(item) => {
                                            ui.add(
                                                egui::Label::new(
                                                    crate::fonts::bold(format::title(item))
                                                        .size(14.0)
                                                        .color(track_text),
                                                )
                                                .truncate(),
                                            );
                                            let artist = item
                                                .artist
                                                .as_deref()
                                                .or(item.album_artist.as_deref())
                                                .unwrap_or_default();
                                            // Artist, then " - " and the album name. The artist is plain text. The
                                            // album is a link to its album page, fully black while hovered.
                                            let album = item.album.as_deref().filter(|a| !a.is_empty());
                                            let font = egui::FontId::proportional(12.0);
                                            let album_width = album.map_or(0.0, |album| {
                                                ui.painter()
                                                    .layout_no_wrap(
                                                        format!(" - {album}"),
                                                        font.clone(),
                                                        track_dim,
                                                    )
                                                    .size()
                                                    .x
                                            });
                                            let artist_width =
                                                (ui.available_width() - album_width).max(0.0);
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 0.0;
                                                ui.scope(|ui| {
                                                    ui.set_max_width(artist_width);
                                                    ui.add(
                                                        egui::Label::new(
                                                            egui::RichText::new(artist)
                                                                .size(12.0)
                                                                .color(track_dim),
                                                        )
                                                        .truncate(),
                                                    );
                                                });
                                                if let Some(album) = album {
                                                    ui.label(
                                                        egui::RichText::new(" - ")
                                                            .size(12.0)
                                                            .color(track_dim),
                                                    );
                                                    // The album name is cut short at the row's edge, then painted
                                                    // in its hover colour once the pointer's position is known.
                                                    let mut job = egui::text::LayoutJob::simple_singleline(
                                                        album.to_owned(),
                                                        font,
                                                        track_dim,
                                                    );
                                                    job.wrap.max_width = ui.available_width().max(0.0);
                                                    job.wrap.max_rows = 1;
                                                    job.wrap.break_anywhere = true;
                                                    job.wrap.overflow_character = Some('\u{2026}');
                                                    let galley = ui.painter().layout_job(job);
                                                    let (rect, link) = ui.allocate_exact_size(
                                                        galley.size(),
                                                        egui::Sense::click(),
                                                    );
                                                    let link = link.on_hover_cursor(
                                                        egui::CursorIcon::PointingHand,
                                                    );
                                                    let colour = if link.hovered() {
                                                        track_text
                                                    } else {
                                                        track_dim
                                                    };
                                                    ui.painter().galley_with_override_text_color(
                                                        rect.min, galley, colour,
                                                    );
                                                    if link.clicked() {
                                                        self.album_request = Some(album.to_owned());
                                                    }
                                                }
                                            });
                                        }
                                        None => {
                                            ui.weak("Nothing playing");
                                        }
                                    });
                                },
                            );

                        ui.add_space(2.0);
                        // The seek bar spans the column: from the cover's left edge to the title's right
                        // edge. The two times sit above it, at its ends.
                        let mut secs = self.seek_secs;
                        let mut seek_to = None;
                        // One row: the elapsed time at the cover's left edge, the rail between, and the total
                        // time at the title's right edge. The slider fills the space between the two labels,
                        // and each label's line of text is centred on the rail.
                        let font = egui::TextStyle::Small.resolve(ui.style());
                        let weak = ui.visuals().weak_text_color();
                        let start = ui
                            .painter()
                            .layout_no_wrap(format::clock(secs), font.clone(), weak);
                        let end = ui
                            .painter()
                            .layout_no_wrap(format::clock(total), font, weak);
                        let gap = 8.0;
                        let (row, _) =
                            ui.allocate_exact_size(egui::vec2(column, 16.0), egui::Sense::hover());
                        let slider_left = row.left() + start.size().x + gap;
                        let slider_right = row.right() - end.size().x - gap;
                        // The seek bar takes the row's width between the two labels.
                        let slider = egui::Rect::from_min_max(
                            egui::pos2(slider_left, row.top()),
                            egui::pos2(slider_right, row.bottom()),
                        );
                        let response =
                            slider::show_in(ui, slider, "seek", &mut secs, 0.0..=total, 0.0);
                        let centre_y = slider.center().y;
                        // The labels sit on the same centre line as the rail and the knob.
                        ui.painter().galley(
                            egui::pos2(row.left(), centre_y - start.size().y / 2.0),
                            start,
                            weak,
                        );
                        ui.painter().galley(
                            egui::pos2(row.right() - end.size().x, centre_y - end.size().y / 2.0),
                            end,
                            weak,
                        );
                        self.dragging = response.dragged();
                        if response.drag_stopped() || response.clicked() {
                            seek_to = Some(secs);
                        }
                        self.seek_secs = secs;
                        if let Some(target) = seek_to {
                            command = Some(Command::Seek(target));
                        }
                    });

                    // Right: information, lyrics, queue, then sound, centred on the cover's height.
                    let mut right_used = 0.0;
                    let mut left_edge = 0.0;
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), COVER_HEIGHT),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let mut level = f64::from(self.volume);
                            let volume = slider::show(ui, &mut level, 0.0..=1.0, 0.0, 80.0);
                            self.volume = level as f32;
                            if volume.changed() {
                                command = Some(Command::SetVolume(self.volume));
                            }
                            if speaker_button(ui, self.muted)
                                .on_hover_text(if self.muted { "Unmute" } else { "Mute" })
                                .clicked()
                            {
                                command = Some(Command::ToggleMute);
                            }
                            // Output: a button that opens a list of audio outputs.
                            let output = output_button(ui, self.output_menu).on_hover_text("Output");
                            if output.clicked() {
                                self.output_menu = !self.output_menu;
                                // Refresh the device list whenever the list opens.
                                refresh |= self.output_menu;
                            }
                            if self.output_menu {
                                let height = 44.0 + 28.0 * self.outputs.len() as f32;
                                let pos = egui::pos2(
                                    output.rect.right() - 240.0,
                                    output.rect.top() - height - 8.0,
                                );
                                let area = egui::Area::new(ui.scope_id().with("output_menu"))
                                    .order(egui::Order::Foreground)
                                    .fixed_pos(pos)
                                    .show(ui.ctx(), |ui| {
                                        egui::Frame::popup(ui.style()).show(ui, |ui| {
                                            ui.set_width(240.0);
                                            ui.weak("Output");
                                            if ui
                                                .selectable_label(
                                                    self.output.is_none(),
                                                    "System default",
                                                )
                                                .clicked()
                                            {
                                                choice = Some(None);
                                            }
                                            for name in &self.outputs {
                                                if ui
                                                    .selectable_label(
                                                        self.output.as_deref() == Some(name),
                                                        name,
                                                    )
                                                    .clicked()
                                                {
                                                    choice = Some(Some(name.clone()));
                                                }
                                            }
                                        });
                                    });
                                if choice.is_some() {
                                    self.output_menu = false;
                                }
                                // A click anywhere else closes the list.
                                let pressed = ui.input(|i| i.pointer.any_pressed());
                                if pressed && !area.response.contains_pointer() && !output.hovered() {
                                    self.output_menu = false;
                                }
                            }
                            if panel_button(ui, Panel::Queue, self.panel == Panel::Queue)
                                .on_hover_text("Up next")
                                .clicked()
                            {
                                command = Some(Command::TogglePanel(Panel::Queue));
                            }
                            if panel_button(ui, Panel::Lyrics, self.panel == Panel::Lyrics)
                                .on_hover_text("Lyrics")
                                .clicked()
                            {
                                command = Some(Command::TogglePanel(Panel::Lyrics));
                            }
                            // Playback speed: each click steps to the next preset.
                            let speed_label = speed_text(self.speed);
                            if speed_button(ui, &speed_label)
                                .on_hover_text("Playback speed. Click to change. Speeds other than 1× also change the pitch.")
                                .clicked()
                            {
                                self.speed_change = Some(next_speed(self.speed));
                            }
                            let info = panel_button(ui, Panel::Info, self.panel == Panel::Info)
                                .on_hover_text("Track info");
                            left_edge = info.rect.left();
                            if info.clicked() {
                                command = Some(Command::TogglePanel(Panel::Info));
                            }
                        right_used = ui.max_rect().right() - left_edge;
                        },
                );
                self.right_width = right_used;
                });
                if let Some(error) = self.error.as_deref() {
                    ui.add_space(6.0);
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
            });

        if refresh {
            self.outputs = output_device_names();
        }
        if let Some(device) = choice {
            self.set_output(device);
        }
        self.run(command);
    }

    fn run(&mut self, command: Option<Command>) {
        match command {
            Some(Command::Toggle) => self.toggle(),
            Some(Command::Next) => self.next(),
            Some(Command::Previous) => self.previous(),
            Some(Command::Seek(target)) => self.seek(target),
            Some(Command::SetVolume(level)) => self.set_volume(level),
            Some(Command::ToggleMute) => self.toggle_mute(),
            Some(Command::ToggleShuffle) => self.set_shuffle(!self.shuffle),
            Some(Command::CycleRepeat) => self.cycle_repeat(),
            Some(Command::TogglePanel(panel)) => {
                self.panel = if self.panel == panel {
                    Panel::Closed
                } else {
                    panel
                };
            }
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

/// The bar in a narrow window: the track, then play or pause and next, in one row. Returns the command
/// the user chose, if any.
fn show_compact(
    ui: &mut egui::Ui,
    state: PlayerState,
    current: Option<&MediaItem>,
    track_text: egui::Color32,
    track_dim: egui::Color32,
) -> Option<Command> {
    const ROW: f32 = 36.0;
    const GAP: f32 = 8.0;
    let mut command = None;
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(24, 12))
        .show(ui, |ui| {
            // Each item gets its own rectangle, centred on the row's line, so the items share one centre line.
            let (row, _) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), ROW), egui::Sense::hover());
            let centre_y = row.center().y;
            let next = egui::Rect::from_min_size(
                egui::pos2(row.right() - 32.0, centre_y - 16.0),
                egui::vec2(32.0, 32.0),
            );
            let play = egui::Rect::from_min_size(
                egui::pos2(next.left() - GAP - 36.0, centre_y - 18.0),
                egui::vec2(36.0, 36.0),
            );
            let title_h = ui.fonts_mut(|f| f.row_height(&egui::FontId::proportional(14.0)));
            let artist_h = ui.fonts_mut(|f| f.row_height(&egui::FontId::proportional(12.0)));
            let block = title_h + artist_h + 2.0;
            let text = egui::Rect::from_min_size(
                egui::pos2(row.left(), centre_y - block / 2.0),
                egui::vec2((play.left() - GAP - row.left()).max(0.0), block),
            );

            if ui
                .put(next, |ui: &mut egui::Ui| {
                    transport_button(ui, Icon::Next, 32.0).on_hover_text("Next track")
                })
                .clicked()
            {
                command = Some(Command::Next);
            }
            let (icon, tip) = if state == PlayerState::Playing {
                (Icon::Pause, "Pause")
            } else {
                (Icon::Play, "Play")
            };
            if ui
                .put(play, |ui: &mut egui::Ui| {
                    transport_button(ui, icon, 36.0).on_hover_text(tip)
                })
                .clicked()
            {
                command = Some(Command::Toggle);
            }
            ui.put(text, |ui: &mut egui::Ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    match current {
                        Some(item) => {
                            let artist = item
                                .artist
                                .as_deref()
                                .or(item.album_artist.as_deref())
                                .unwrap_or_default();
                            ui.add(
                                egui::Label::new(
                                    crate::fonts::bold(format::title(item))
                                        .size(14.0)
                                        .color(track_text),
                                )
                                .truncate(),
                            );
                            // Artist, then " - " and the album name, as in the wide bar. The line is cut short at the edge.
                            let line = match item.album.as_deref().filter(|a| !a.is_empty()) {
                                Some(album) => format!("{artist} - {album}"),
                                None => artist.to_owned(),
                            };
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(line).size(12.0).color(track_dim),
                                )
                                .truncate(),
                            );
                        }
                        None => {
                            ui.weak("Nothing playing");
                        }
                    }
                })
                .response
            });
        });
    command
}

/// A faint background that eases in over 200 ms while the pointer is over a button, and eases out again
/// when it leaves.
fn hover_fill(ui: &egui::Ui, rect: egui::Rect, radius: f32, response: &egui::Response) {
    let amount = ui.ctx().animate_bool_with_time_and_easing(
        response.id.with("hover_fill"),
        response.hovered(),
        0.2,
        egui::emath::easing::quadratic_in,
    );
    if amount > 0.0 {
        let colour = ui
            .visuals()
            .widgets
            .hovered
            .weak_bg_fill
            .linear_multiply(amount);
        ui.painter().rect_filled(rect, radius, colour);
    }
}

/// Round or rounded button with a transport icon drawn by the painter, so no font glyphs are needed.
fn transport_button(ui: &mut egui::Ui, icon: Icon, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    hover_fill(
        ui,
        rect,
        if matches!(icon, Icon::Play | Icon::Pause) {
            size / 2.0
        } else {
            6.0
        },
        &response,
    );
    let color = visuals.fg_stroke.color;
    let painter = ui.painter();

    let glyph = match icon {
        Icon::Previous => icons::SKIP_BACK,
        Icon::Next => icons::SKIP_FORWARD,
        Icon::Play => icons::PLAY,
        Icon::Pause => icons::PAUSE,
    };
    icons::paint(painter, rect.center(), glyph, size * 0.5, color);
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
    hover_fill(ui, rect, 14.0, &response);
    let color = if active {
        ui.visuals().hyperlink_color
    } else {
        visuals.fg_stroke.color
    };
    let painter = ui.painter();
    let glyph = match icon {
        ModeIcon::Shuffle => icons::SHUFFLE,
        ModeIcon::Repeat(Repeat::Track) => icons::REPEAT_ONCE,
        ModeIcon::Repeat(_) => icons::REPEAT,
    };
    icons::paint(painter, rect.center(), glyph, 16.0, color);
    response
}

/// A background thread that measures the files it is sent, and stores each result in the library at
/// `db`. It sends each file back once it is stored, so the player can apply the loudness.
fn spawn_measurer(db: PathBuf) -> (Sender<PathBuf>, Receiver<PathBuf>) {
    let (requests, queued) = mpsc::channel::<PathBuf>();
    let (done_tx, done) = mpsc::channel::<PathBuf>();
    thread::spawn(move || {
        // Ends when the player is dropped, which closes the request channel.
        for path in queued {
            if let Some(measured) = player_playback::measure_loudness(&path)
                && let Ok(library) = player_core::Library::open(&db)
            {
                let _ = library.set_loudness(&path, measured.lufs, measured.peak);
            }
            if done_tx.send(path).is_err() {
                break;
            }
        }
    });
    (requests, done)
}

/// The cover's size, which the bar's controls centre on.
/// The track info text colours for a theme: the full colour for the title and a hovered album, and the
/// same colour at 64% opacity for the artist, the album and the separator between them.
fn track_text_colours(dark: bool) -> (egui::Color32, egui::Color32) {
    if dark {
        (egui::Color32::WHITE, egui::Color32::from_white_alpha(163))
    } else {
        (egui::Color32::BLACK, egui::Color32::from_black_alpha(163))
    }
}

const COVER_HEIGHT: f32 = 48.0;
/// Width of the transport: five buttons and the gaps between them.
const TRANSPORT_WIDTH: f32 = 188.0;

/// The playback speeds the bar's speed button steps through.
const SPEEDS: [f32; 5] = [0.75, 1.0, 1.25, 1.5, 2.0];

/// The speed after `speed` in `SPEEDS`, wrapping back to the first.
fn next_speed(speed: f32) -> f32 {
    let current = SPEEDS
        .iter()
        .position(|s| (s - speed).abs() < 0.01)
        .unwrap_or(1);
    SPEEDS[(current + 1) % SPEEDS.len()]
}

/// A speed as the bar shows it: `1×`, `1.25×`.
fn speed_text(speed: f32) -> String {
    let text = format!("{speed:.2}");
    format!("{}×", text.trim_end_matches('0').trim_end_matches('.'))
}

/// A small pill button showing the playback speed.
fn speed_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(40.0, 28.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    hover_fill(ui, rect, 14.0, &response);
    let colour = visuals.fg_stroke.color;
    let painter = ui.painter();
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(12.0),
        colour,
    );
    response
}

/// Round button with an icon for a side panel. It is tinted when that panel is open.
fn panel_button(ui: &mut egui::Ui, panel: Panel, active: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    hover_fill(ui, rect, 14.0, &response);
    let color = if active {
        ui.visuals().hyperlink_color
    } else {
        visuals.fg_stroke.color
    };
    let painter = ui.painter();
    let glyph = match panel {
        Panel::Queue => icons::QUEUE,
        Panel::Lyrics => icons::QUOTES,
        Panel::Info => icons::INFO,
        Panel::Closed => "",
    };
    icons::paint(painter, rect.center(), glyph, 16.0, color);
    response
}

/// Round button with an output icon: a screen with a pointer below it. Tinted while the output
/// list is open.
fn output_button(ui: &mut egui::Ui, active: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    hover_fill(ui, rect, 14.0, &response);
    let colour = if active {
        ui.visuals().hyperlink_color
    } else {
        visuals.fg_stroke.color
    };
    let painter = ui.painter();
    icons::paint(painter, rect.center(), icons::AIRPLAY, 16.0, colour);
    response
}

/// Speaker icon for mute: waves when sound is on, a cross when muted.
fn speaker_button(ui: &mut egui::Ui, muted: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    hover_fill(ui, rect, 11.0, &response);
    let color = visuals.fg_stroke.color;
    let painter = ui.painter();
    let glyph = if muted {
        icons::SPEAKER_X
    } else {
        icons::SPEAKER_HIGH
    };
    icons::paint(painter, rect.center(), glyph, 16.0, color);
    response
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
            video_codec: None,
            width: None,
            height: None,
            video_bit_depth: None,
            video_chroma: None,
            video_fps: None,
            video_fps_variable: false,
            video_aspect: None,
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
            video: None,
            video_loaded: false,
            queue: Vec::new(),
            original: Vec::new(),
            index: 0,
            shuffle: false,
            repeat: Repeat::Off,
            volume: 1.0,
            muted: false,
            speed: 1.0,
            crossfade: 0.0,
            speed_change: None,
            album_request: None,
            right_width: 300.0,
            library: None,
            measuring: HashSet::new(),
            measure_tx: None,
            measured_rx: None,
            seek_secs: 0.0,
            dragging: false,
            error: None,
            row_widths: [0.0; 3],
            panel: Panel::Closed,
            replay_gain: ReplayGainMode::Off,
            output: None,
            outputs: Vec::new(),
            gain: 1.0,
            armed: None,
            radio: None,
            radio_loading: None,
            radio_paused: false,
            radio_reconnects: 0,
            output_menu: false,
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
