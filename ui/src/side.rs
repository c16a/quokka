//! The lyrics and track info panels, beside the main view.

use crate::i18n::{self, Text};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use eframe::egui;
use quokka_core::{
    Library, LyricLine, Lyrics, LyricsQuery, MediaItem, RemoteLyrics, TrackDetails, best_lyrics,
    fetch_lyrics, lyrics_from_text, sidecar_lyrics, track_details,
};

use crate::format;
use crate::squircle;

/// What the lyrics panel asked for this frame.
#[derive(Default)]
pub struct LyricsOutcome {
    /// A synced line was clicked: seek to its time.
    pub seek: Option<Duration>,
    /// The user chose to turn on lookups on lrclib.net.
    pub enable: bool,
}

/// Where the lookup for the current track stands.
#[derive(Default, Clone, PartialEq, Eq)]
enum Lookup {
    #[default]
    Idle,
    Pending,
    /// Finished. The cache holds the result, which may be that nothing was found.
    Done,
    Failed(Text),
}

/// What a lookup sends back: the track it was for, and the result.
type LookupResult = (i64, Result<Option<RemoteLyrics>, String>);

#[derive(Default)]
pub struct SidePanels {
    /// The track the state below belongs to.
    track: Option<i64>,
    details: Option<TrackDetails>,
    /// A `.lrc` file beside the track. It overrides every other source.
    sidecar: Option<Lyrics>,
    /// The cached lookup: `None` never looked up, `Some(None)` looked up and nothing found,
    /// `Some(Some(lyrics))` found.
    cached: Option<Option<Lyrics>>,
    lookup: Lookup,
    results: Option<Receiver<LookupResult>>,
    /// The synced line last scrolled into view.
    scrolled: Option<usize>,
    /// The value of each tag line as it is being edited. Same order as `details.tags`.
    drafts: Vec<String>,
    /// Why the last save or revert failed, if it did.
    tag_error: Option<String>,
    /// Set when tags were written, so the library views reload.
    library_changed: bool,
    /// The track's library row, read again after a save.
    refreshed: Option<MediaItem>,
    /// Whether the track has a tag snapshot to revert to. Read on load, then kept up to date by saves and reverts.
    can_revert: bool,
}

/// What the user asked for in the tags section this frame.
enum TagAction {
    Save(Vec<String>),
    Revert,
}

/// The editable values of a track's tags, as they start out.
fn draft_values(details: Option<&TrackDetails>) -> Vec<String> {
    details
        .map(|d| d.tags.iter().map(|t| t.value.clone()).collect())
        .unwrap_or_default()
}

impl SidePanels {
    /// Load what the panels need when the playing track changes.
    fn load(&mut self, item: &MediaItem, library: &Library) {
        if self.track == Some(item.id) {
            return;
        }
        self.track = Some(item.id);
        self.details = track_details(&item.path);
        self.drafts = draft_values(self.details.as_ref());
        self.tag_error = None;
        self.refreshed = None;
        self.can_revert = library.has_tag_snapshot(&item.path).unwrap_or(false);
        self.sidecar = sidecar_lyrics(&item.path);
        self.cached = library.cached_lyrics(item.id).unwrap_or(None);
        self.lookup = Lookup::Idle;
        self.results = None;
        self.scrolled = None;
    }

    /// The lyrics to show now: the `.lrc` file if there is one, otherwise the best of the file's
    /// tags and the cached lookup.
    fn current(&self) -> Option<Lyrics> {
        if let Some(sidecar) = &self.sidecar {
            return Some(sidecar.clone());
        }
        let embedded = self
            .details
            .as_ref()
            .and_then(|d| d.lyrics.as_deref())
            .and_then(lyrics_from_text);
        let cached = self.cached.clone().flatten();
        best_lyrics(embedded.into_iter().chain(cached))
    }

    /// Store a finished lookup and remember its outcome. Results for another track are dropped.
    fn collect(&mut self, library: &Library) {
        let Some(rx) = self.results.take() else {
            return;
        };
        match rx.try_recv() {
            Ok((id, result)) => {
                if Some(id) != self.track {
                    return;
                }
                match result {
                    Ok(Some(remote)) => {
                        let text = remote.synced.or(remote.plain);
                        store(library, id, text.as_deref());
                        self.cached = Some(text.as_deref().and_then(lyrics_from_text));
                        self.lookup = Lookup::Done;
                    }
                    Ok(None) => {
                        store(library, id, None);
                        self.cached = Some(None);
                        self.lookup = Lookup::Done;
                    }
                    // A failed lookup is not cached, so it can be tried again.
                    Err(e) => self.lookup = Lookup::Failed(Text::from(e)),
                }
            }
            Err(TryRecvError::Empty) => self.results = Some(rx),
            Err(TryRecvError::Disconnected) => {
                self.lookup = Lookup::Failed(Text::of(i18n::the_lookup_stopped_unexpectedly));
            }
        }
    }

    /// Start a lookup for the track on a background thread, unless one is needed and not running.
    fn look_up(&mut self, item: &MediaItem, library: &Library, ctx: &egui::Context) {
        self.collect(library);
        // A `.lrc` file or synced lyrics already in hand need no lookup.
        let needed = self.sidecar.is_none() && !self.current().is_some_and(|l| l.is_synced());
        if !needed || self.lookup != Lookup::Idle {
            return;
        }
        let artist = item
            .artist
            .clone()
            .or_else(|| item.album_artist.clone())
            .unwrap_or_default();
        if artist.is_empty() {
            // Lookups match on the artist, so a track without one can't be looked up.
            self.lookup = Lookup::Done;
            return;
        }
        let query = LyricsQuery {
            title: format::title(item),
            artist,
            album: item.album.clone(),
            duration: item
                .duration_ms
                .and_then(|ms| u64::try_from(ms).ok())
                .map(Duration::from_millis),
        };
        let (tx, rx) = mpsc::channel();
        let id = item.id;
        let ctx = ctx.clone();
        thread::spawn(move || {
            let result = fetch_lyrics(&query);
            let _ = tx.send((id, result));
            ctx.request_repaint();
        });
        self.lookup = Lookup::Pending;
        self.results = Some(rx);
    }

    /// The lyrics panel. Synced lines follow the playback position and can be clicked to seek.
    /// When there are no lyrics and lookups are off, it offers to turn them on.
    pub fn lyrics(
        &mut self,
        ui: &mut egui::Ui,
        library: &Library,
        item: Option<&MediaItem>,
        position: Duration,
        online: bool,
    ) -> LyricsOutcome {
        let mut outcome = LyricsOutcome::default();
        heading(ui, i18n::lyrics());
        let Some(item) = item else {
            indented(ui, |ui| {
                ui.weak(i18n::nothing_playing_2());
            });
            return outcome;
        };
        self.load(item, library);
        if online {
            self.look_up(item, library, ui.ctx());
        }
        let current = self.current();
        let lookup = self.lookup.clone();
        let has_sidecar = self.sidecar.is_some();
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(0, 10))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match &current {
                        Some(Lyrics::Synced(lines)) => {
                            outcome.seek = self.synced(ui, lines, position);
                        }
                        Some(Lyrics::Plain(text)) => indented(ui, |ui| {
                            ui.label(text);
                            if !online && !has_sidecar {
                                ui.add_space(12.0);
                                offer(ui, &mut outcome);
                            }
                        }),
                        None => indented(ui, |ui| {
                            if !online {
                                ui.label(i18n::this_track_has_no_lyrics_in());
                                ui.add_space(4.0);
                                offer(ui, &mut outcome);
                            } else {
                                match &lookup {
                                    Lookup::Idle | Lookup::Pending => {
                                        ui.horizontal(|ui| {
                                            ui.spinner();
                                            ui.weak(i18n::looking_up_lyrics_on_lrclib_net());
                                        });
                                    }
                                    Lookup::Done => {
                                        ui.weak(i18n::no_lyrics_were_found_on_lrclib());
                                    }
                                    Lookup::Failed(e) => {
                                        ui.colored_label(
                                            ui.visuals().error_fg_color,
                                            i18n::fill(
                                                i18n::couldnt_reach_lrclib(),
                                                &[&e.to_string()],
                                            ),
                                        );
                                        if squircle::text_button(ui, i18n::try_again()).clicked() {
                                            self.lookup = Lookup::Idle;
                                        }
                                    }
                                }
                            }
                        }),
                    });
            });
        outcome
    }

    /// Synced lines, the current one highlighted and kept in view. Returns the time of a line
    /// the user clicked.
    fn synced(
        &mut self,
        ui: &mut egui::Ui,
        lines: &[LyricLine],
        position: Duration,
    ) -> Option<Duration> {
        let current = lines.iter().rposition(|line| line.at <= position);
        let mut seek = None;
        ui.add_space(8.0);
        for (i, line) in lines.iter().enumerate() {
            let is_current = Some(i) == current;
            let text = if line.text.is_empty() {
                " "
            } else {
                line.text.as_str()
            };
            // Same size for every line. Only the line being sung is at full opacity.
            let full = ui.visuals().text_color();
            let color = if is_current {
                full
            } else {
                full.gamma_multiply(0.4)
            };
            let rich = egui::RichText::new(text).size(16.0).color(color);
            let response = egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(14, 0))
                .show(ui, |ui| {
                    ui.add(egui::Label::new(rich).wrap().sense(egui::Sense::click()))
                })
                .inner;
            if is_current && self.scrolled != Some(i) {
                response.scroll_to_me(Some(egui::Align::Center));
                self.scrolled = Some(i);
            }
            if response.clicked() {
                seek = Some(line.at);
            }
            ui.add_space(6.0);
        }
        seek
    }

    /// Everything the file says about itself, and a button to show it in the file manager.
    /// Whether tags were written since this was last asked, so the library views should reload.
    pub fn take_library_changed(&mut self) -> bool {
        std::mem::take(&mut self.library_changed)
    }

    pub fn info(&mut self, ui: &mut egui::Ui, library: &Library, item: Option<&MediaItem>) {
        heading(ui, i18n::track_info());
        let Some(item) = item else {
            indented(ui, |ui| {
                ui.weak(i18n::nothing_playing_2());
            });
            return;
        };
        self.load(item, library);
        let item = self
            .refreshed
            .as_ref()
            .filter(|row| row.id == item.id)
            .unwrap_or(item);
        let lines = self
            .details
            .as_ref()
            .map(|d| d.tags.clone())
            .unwrap_or_default();
        let can_revert = self.can_revert;
        let error = self.tag_error.clone();
        let details = self.details.as_ref();
        let drafts = &mut self.drafts;
        let mut action: Option<TagAction> = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                indented(ui, |ui| {
                    egui::Grid::new("track_info")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            row(ui, i18n::title(), &format::title(item));
                            row(
                                ui,
                                i18n::artist(),
                                item.artist
                                    .as_deref()
                                    .or(item.album_artist.as_deref())
                                    .unwrap_or("—"),
                            );
                            row(ui, i18n::album_2(), item.album.as_deref().unwrap_or("—"));
                            row(ui, i18n::codec(), item.codec.as_deref().unwrap_or("—"));
                            row(ui, i18n::duration(), &format::duration(item.duration_ms));
                            row(ui, i18n::size(), &format::size(item.size));
                            if let Some(kbps) = item.bitrate_kbps {
                                row(ui, i18n::bitrate(), &format::kbps(kbps));
                            }
                            if let Some(sample) =
                                format::sample_format(item.bit_depth, item.sample_rate)
                            {
                                row(ui, i18n::format(), &sample);
                            }
                            if let Some(channels) = item.channels {
                                row(ui, i18n::channels(), &format::channels(channels));
                            }
                            if let Some(details) = details {
                                row(ui, i18n::pictures(), &details.pictures.to_string());
                            }
                            row(ui, i18n::file(), &item.path.display().to_string());
                        });
                    ui.add_space(10.0);
                    if squircle::text_button(ui, i18n::reveal_in_finder()).clicked() {
                        reveal(&item.path);
                    }
                });
                if let Some(details) = details {
                    ui.add_space(16.0);
                    indented(ui, |ui| {
                        ui.label(crate::fonts::bold(i18n::tags()));
                        ui.add_space(4.0);
                        egui::Grid::new("track_tags")
                            .num_columns(2)
                            .spacing([12.0, 4.0])
                            .show(ui, |ui| {
                                for (line, draft) in details.tags.iter().zip(drafts.iter_mut()) {
                                    ui.weak(&line.name);
                                    if line.editable {
                                        crate::squircle::SquircleTextField::singleline(draft)
                                            .desired_width(f32::INFINITY)
                                            .show(ui);
                                    } else {
                                        ui.add(egui::Label::new(line.value.as_str()).wrap());
                                    }
                                    ui.end_row();
                                }
                            });
                        ui.add_space(8.0);
                        let dirty = details
                            .tags
                            .iter()
                            .zip(drafts.iter())
                            .any(|(line, draft)| line.editable && &line.value != draft);
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled_ui(dirty, |ui| squircle::text_button(ui, i18n::save()))
                                .inner
                                .clicked()
                            {
                                action = Some(TagAction::Save(drafts.clone()));
                            }
                            if ui
                                .add_enabled_ui(can_revert, |ui| {
                                    squircle::text_button(ui, i18n::revert())
                                })
                                .inner
                                .on_hover_text(i18n::put_back_the_tags_the_file())
                                .clicked()
                            {
                                action = Some(TagAction::Revert);
                            }
                        });
                        if let Some(error) = &error {
                            ui.colored_label(ui.visuals().error_fg_color, error.as_str());
                        }
                    });
                }
            });

        let path = item.path.clone();
        let id = item.id;
        // A save leaves a snapshot to revert to, and a revert takes it away.
        let outcome = match action {
            Some(TagAction::Save(values)) => {
                Some((library.save_tags(&path, &lines, &values), true))
            }
            Some(TagAction::Revert) => Some((library.revert_tags(&path).map(|_| ()), false)),
            None => None,
        };
        if let Some((result, has_snapshot)) = outcome {
            match result {
                Ok(()) => {
                    self.details = track_details(&path);
                    self.drafts = draft_values(self.details.as_ref());
                    self.refreshed = library.get(id).ok().flatten();
                    self.tag_error = None;
                    self.library_changed = true;
                    self.can_revert = has_snapshot;
                }
                Err(e) => self.tag_error = Some(e.to_string()),
            }
        }
    }
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.heading(text);
    });
    ui.add_space(8.0);
}

/// Content with the panel's side margin.
fn indented(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(14, 0))
        .show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            add(ui);
        });
}

/// One label and its value, as a grid row. Long values wrap.
fn row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.weak(label);
    ui.add(egui::Label::new(value).wrap());
    ui.end_row();
}

/// Select `path` in the file manager. Falls back to opening its folder where the file manager
/// can't select a file.
fn reveal(path: &Path) {
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg("-R").arg(path).spawn();
    #[cfg(target_os = "windows")]
    let result = {
        use std::os::windows::process::CommandExt;
        // Explorer reads `/select,"path"` as typed. A plain argument would have the quotes added
        // around the whole thing, which Explorer does not read when the path has spaces.
        Command::new("explorer")
            .raw_arg(format!("/select,\"{}\"", path.display()))
            .spawn()
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let result = Command::new("xdg-open")
        .arg(path.parent().unwrap_or(path))
        .spawn();
    if let Err(e) = result {
        eprintln!(
            "player: could not show {} in the file manager: {e}",
            path.display()
        );
    }
}

/// Save a lookup result, or the fact that nothing was found. Failure is logged, not shown.
fn store(library: &Library, id: i64, text: Option<&str>) {
    if let Err(e) = library.store_lyrics(id, text) {
        eprintln!("player: could not cache lyrics: {e}");
    }
}

/// The prompt that asks for permission before lyrics are looked up online.
fn offer(ui: &mut egui::Ui, outcome: &mut LyricsOutcome) {
    ui.label(i18n::look_up_synced_lyrics_on_lrclib_2());
    if squircle::text_button(ui, i18n::turn_on_lyrics_lookup()).clicked() {
        outcome.enable = true;
    }
}
