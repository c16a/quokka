//! Desktop UI (egui via eframe): Albums, Movies, Settings views, and the playback bar.

mod albums;
mod artists;
mod covers;
mod fonts;
mod format;
mod icons;
mod media;
mod movies;
mod playback;
mod playlists;
mod queue;
mod radio;
mod recent;
mod search;
mod settings_view;
mod side;
mod slider;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;
use notify::{RecursiveMode, Watcher};
use player_core::{Library, MediaItem, ScanStats, Session, Settings, Theme};

use playback::{MenuRequest, Panel, Playback};

/// Where the app keeps its library database and settings file.
pub struct Paths {
    pub db: PathBuf,
    pub settings: PathBuf,
    /// Where the playback state is saved on exit, so it can resume.
    pub session: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Radio,
    Search,
    Albums,
    Recent,
    Artists,
    Playlists,
    Movies,
    Settings,
}

/// The outcome of scanning one folder.
type ScanResult = Result<ScanStats, String>;

/// How long the library must stay quiet after a file change before rescanning.
const RESCAN_DELAY: Duration = Duration::from_secs(2);

struct App {
    paths: Paths,
    settings: Settings,
    library: Library,
    tab: Tab,
    albums: albums::AlbumsView,
    search: search::SearchView,
    movies: movies::MoviesView,
    playback: Playback,
    queue_view: queue::QueueView,
    artists: artists::ArtistsView,
    recent: recent::RecentView,
    radio: radio::RadioView,
    /// A station index sync running on a background thread, and its result.
    station_sync: Option<std::sync::mpsc::Receiver<Result<usize, String>>>,
    side: side::SidePanels,
    /// The track the info panel shows, when one was chosen from a menu. Otherwise the playing track.
    info_track: Option<MediaItem>,
    /// The track last logged as played, so one track is logged once per start.
    last_played: Option<i64>,
    playlists: playlists::PlaylistsView,
    /// System media keys and now-playing display. `None` when the system has none.
    media: Option<media::Media>,
    /// The folder being scanned now, and the channel its result arrives on.
    scan: Option<(PathBuf, Receiver<ScanResult>)>,
    /// Folders waiting to be scanned. Each folder is scanned on its own, one after another.
    queue: VecDeque<PathBuf>,
    /// One line per folder scanned in the current run.
    report: Vec<String>,
    /// Watches the library folder. Dropping it stops the watch.
    watcher: Option<notify::RecommendedWatcher>,
    change_tx: Sender<()>,
    changes: Receiver<()>,
    /// When to rescan after changes, once the library has been quiet for a while.
    rescan_at: Option<Instant>,
    /// The playback state last written to disk, and when the state was last checked.
    session_saved: Option<Session>,
    last_save: Instant,
    status: String,
    /// Whether the sidebar is open. It collapses to a narrow rail when the window is narrow.
    sidebar_open: bool,
    /// Whether the window was narrow last frame, so the sidebar only changes when the width crosses the line.
    narrow: bool,
    /// Indexed files under each library folder, in the order of `settings.library_roots`.
    folder_files: Vec<i64>,
}

/// Open the library and settings, then run the window until it is closed.
pub fn run(paths: Paths) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(dir) = paths.db.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let settings = Settings::load(&paths.settings)?;
    let library = Library::open(&paths.db)?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Player")
            .with_inner_size([1100.0, 720.0])
            // The narrowest the window can be dragged. Narrow layouts still need room for the page padding and the rail.
            .with_min_inner_size([MIN_WINDOW_WIDTH, 0.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Player",
        options,
        Box::new(move |cc| {
            let app = App::new(cc.egui_ctx.clone(), paths, settings, library);
            let app: Box<dyn eframe::App> = Box::new(app);
            Ok(app)
        }),
    )?;
    Ok(())
}

/// The window's surfaces in each theme: the main pane, the sidebar and the playback bar.
const LIGHT_MAIN: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xEE, 0xEE);
const LIGHT_SIDEBAR: egui::Color32 = egui::Color32::from_rgb(0xDD, 0xDD, 0xDD);
const LIGHT_BAR: egui::Color32 = egui::Color32::WHITE;
const DARK_MAIN: egui::Color32 = egui::Color32::from_rgb(0x11, 0x11, 0x11);
const DARK_SIDEBAR: egui::Color32 = egui::Color32::from_rgb(0x22, 0x22, 0x22);
const DARK_BAR: egui::Color32 = egui::Color32::BLACK;

/// Map the saved theme onto egui's preference.
fn preference(theme: Theme) -> egui::ThemePreference {
    match theme {
        Theme::System => egui::ThemePreference::System,
        Theme::Light => egui::ThemePreference::Light,
        Theme::Dark => egui::ThemePreference::Dark,
    }
}

impl App {
    fn new(ctx: egui::Context, paths: Paths, settings: Settings, library: Library) -> Self {
        let mut playback = Playback::new();
        playback.set_library(paths.db.clone());
        // Artist photos are cached beside the library database.
        let photo_dir = paths
            .db
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("artist-photos");
        fonts::install(&ctx);
        // Buttons grow a little when hovered, which made them flicker under the pointer.
        ctx.all_styles_mut(|style| {
            // Hovering only changes colour: size, expansion and stroke width stay the same in every
            // state, so nothing under the pointer moves or flickers.
            let widgets = &mut style.visuals.widgets;
            let fg = widgets.inactive.fg_stroke.width;
            let bg = widgets.inactive.bg_stroke.width;
            for state in [
                &mut widgets.inactive,
                &mut widgets.hovered,
                &mut widgets.active,
            ] {
                state.expansion = 0.0;
                state.fg_stroke.width = fg;
                state.bg_stroke.width = bg;
            }
        });
        // The main pane's colour in each theme. The sidebar and the bar are set where they are drawn.
        ctx.style_mut_of(egui::Theme::Light, |style| {
            style.visuals.panel_fill = LIGHT_MAIN
        });
        ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.visuals.panel_fill = DARK_MAIN
        });
        ctx.set_theme(preference(settings.theme));
        let tab = if !settings.library_roots.is_empty() {
            Tab::Albums
        } else {
            Tab::Settings
        };
        let media = media::Media::new(&ctx);
        let (change_tx, changes) = mpsc::channel();
        let mut app = Self {
            paths,
            settings,
            library,
            tab,
            albums: Default::default(),
            search: Default::default(),
            movies: Default::default(),
            playback,
            queue_view: Default::default(),
            artists: artists::ArtistsView::new(photo_dir),
            recent: Default::default(),
            radio: Default::default(),
            station_sync: None,
            side: Default::default(),
            info_track: None,
            last_played: None,
            playlists: Default::default(),
            media,
            scan: None,
            queue: VecDeque::new(),
            report: Vec::new(),
            watcher: None,
            change_tx,
            changes,
            rescan_at: None,
            session_saved: None,
            last_save: Instant::now(),
            status: String::new(),
            sidebar_open: true,
            narrow: false,
            folder_files: Vec::new(),
        };
        app.playback.set_replay_gain(app.settings.replay_gain);
        app.reload_views();
        app.restore_session();
        if station_index_stale(&app.library) {
            app.start_station_sync(&ctx);
        }
        if app.settings.library_roots.is_empty() {
            app.status = "Add a folder to get started.".into();
        } else {
            app.watch(&ctx);
            let roots = app.settings.library_roots.clone();
            app.enqueue(&ctx, roots);
        }
        app
    }

    /// Put back the queue and position saved when the app last closed. Playback stays paused.
    /// Files that are no longer in the library are skipped.
    fn restore_session(&mut self) {
        let Some(session) = Session::load(&self.paths.session) else {
            return;
        };
        let mut items = Vec::new();
        let mut index = None;
        for (i, path) in session.queue.iter().enumerate() {
            let Ok(Some(item)) = self.library.find_path(path) else {
                continue;
            };
            if i == session.index {
                index = Some(items.len());
            }
            items.push(item);
        }
        if items.is_empty() {
            return;
        }
        // If the saved current track is gone, start from the first one at the beginning.
        let position = if index.is_some() {
            session.position_secs
        } else {
            0.0
        };
        self.playback.restore(
            items,
            index.unwrap_or(0),
            position,
            session.shuffle,
            session.repeat,
        );
        self.session_saved = Some(self.playback.session());
    }

    /// Refresh the radio station index on a background thread. Does nothing while a sync runs.
    fn start_station_sync(&mut self, ctx: &egui::Context) {
        if self.station_sync.is_some() {
            return;
        }
        let db = self.paths.db.clone();
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        thread::spawn(move || {
            let _ = tx.send(sync_stations(&db));
            ctx.request_repaint();
        });
        self.station_sync = Some(rx);
        self.status = "Syncing the radio station list…".into();
    }

    /// Collect a finished station sync, if there is one.
    fn poll_station_sync(&mut self) {
        let Some(rx) = self.station_sync.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(count)) => {
                self.status = format!("Radio: {count} stations.");
                self.radio.reload(&self.library);
            }
            Ok(Err(e)) => self.status = format!("Radio station sync failed: {e}"),
            Err(TryRecvError::Empty) => self.station_sync = Some(rx),
            Err(TryRecvError::Disconnected) => {
                self.status = "Radio station sync stopped unexpectedly.".into();
            }
        }
    }

    /// Turn lookups on or off for lyrics, and save the choice.
    fn set_lyrics_online(&mut self, on: bool) {
        self.settings.lyrics_online = on;
        if let Err(e) = self.settings.save(&self.paths.settings) {
            self.status = format!("Could not save settings: {e}");
        }
    }

    /// Log each track as it starts playing, and refresh the recent lists when one does.
    fn record_plays(&mut self) {
        let current = self.playback.current_id();
        if current.is_none() || current == self.last_played || !self.playback.is_playing() {
            return;
        }
        if let Some(id) = current
            && let Err(e) = self.library.record_play(id)
        {
            eprintln!("player: could not record play: {e}");
        }
        self.last_played = current;
        self.recent.reload(&self.library, self.albums.albums());
    }

    /// Save the playback state if it has changed since the last save.
    fn save_session(&mut self) {
        let session = self.playback.session();
        if self.session_saved.as_ref() == Some(&session) {
            return;
        }
        if let Err(e) = session.save(&self.paths.session) {
            eprintln!("player: could not save playback state: {e}");
        }
        self.session_saved = Some(session);
    }

    fn reload_views(&mut self) {
        self.folder_files = self
            .settings
            .library_roots
            .iter()
            .map(|root| self.library.files_under(root).unwrap_or(0))
            .collect();
        self.albums.reload(&self.library);
        self.movies.reload(&self.library);
        self.playlists.reload(&self.library);
        self.artists.reload(self.albums.albums());
        self.recent.reload(&self.library, self.albums.albums());
        self.radio.reload(&self.library);
    }

    /// Add folders to the scan queue. Each folder is scanned on its own. A folder that is
    /// already queued or being scanned is skipped.
    fn enqueue(&mut self, ctx: &egui::Context, roots: Vec<PathBuf>) {
        let idle = self.scan.is_none() && self.queue.is_empty();
        for root in roots {
            let running = self.scan.as_ref().is_some_and(|(r, _)| *r == root);
            if !running && !self.queue.contains(&root) {
                self.queue.push_back(root);
            }
        }
        if idle {
            self.report.clear();
        }
        self.start_next(ctx);
    }

    /// Start scanning the next queued folder on a worker thread. The worker opens its own
    /// connection, so the UI stays responsive while files are read.
    fn start_next(&mut self, ctx: &egui::Context) {
        if self.scan.is_some() {
            return;
        }
        let Some(root) = self.queue.pop_front() else {
            return;
        };
        let (tx, rx) = mpsc::channel();
        let db = self.paths.db.clone();
        let ctx = ctx.clone();
        let job = root.clone();
        thread::spawn(move || {
            let result = Library::open(&db)
                .and_then(|mut lib| lib.scan(&job))
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        let more = match self.queue.len() {
            0 => String::new(),
            n => format!(" ({n} more queued)"),
        };
        self.status = format!("Scanning {}{more}…", root.display());
        self.scan = Some((root, rx));
    }

    /// Save the folder list and watch all of its folders again.
    fn save_and_watch(&mut self, ctx: &egui::Context) {
        if let Err(e) = self.settings.save(&self.paths.settings) {
            self.status = format!("Could not save settings: {e}");
        }
        self.watch(ctx);
    }

    /// Watch every folder for changes. A change schedules a rescan after a short quiet period.
    fn watch(&mut self, ctx: &egui::Context) {
        let roots = self.settings.library_roots.clone();
        let tx = self.change_tx.clone();
        let ctx = ctx.clone();
        let watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            // Reads don't change the library, so they don't need a rescan.
            if let Ok(event) = event
                && !event.kind.is_access()
            {
                let _ = tx.send(());
                ctx.request_repaint();
            }
        });
        self.watcher = match watcher {
            Ok(mut watcher) => {
                // A folder that can't be watched is still scanned at startup and on Rescan.
                for root in &roots {
                    let _ = watcher.watch(root, RecursiveMode::Recursive);
                }
                Some(watcher)
            }
            Err(_) => None,
        };
    }

    /// Start a rescan once the library has been quiet for `RESCAN_DELAY`.
    fn handle_changes(&mut self, ctx: &egui::Context) {
        while self.changes.try_recv().is_ok() {
            self.rescan_at = Some(Instant::now() + RESCAN_DELAY);
        }
        let Some(at) = self.rescan_at else {
            return;
        };
        let now = Instant::now();
        if now < at {
            ctx.request_repaint_after(at - now);
        } else if self.scan.is_none() && self.queue.is_empty() {
            self.rescan_at = None;
            let roots = self.settings.library_roots.clone();
            self.enqueue(ctx, roots);
        }
    }

    fn poll_scan(&mut self, ctx: &egui::Context) {
        let Some((root, rx)) = self.scan.as_ref() else {
            return;
        };
        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("the scan stopped unexpectedly".to_string()),
        };
        let name = root.display().to_string();
        self.scan = None;
        let line = match result {
            Ok(stats) => format!(
                "{name}: {} added or changed, {} unchanged, {} removed.",
                stats.indexed, stats.unchanged, stats.removed
            ),
            // One unreadable folder (for example an unplugged drive) doesn't stop the others.
            Err(e) => format!("{name}: could not be scanned ({e})."),
        };
        self.report.push(line);
        self.reload_views();
        if self.queue.is_empty() {
            self.status = self.report.join("\n");
        }
        self.start_next(ctx);
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_scan(ui.ctx());
        self.poll_station_sync();
        self.handle_changes(ui.ctx());
        self.playback.advance_if_finished();
        self.playback
            .set_tempo(self.settings.playback_speed, self.settings.crossfade_secs);
        let ctx = ui.ctx().clone();
        playback::handle_keys(&ctx, &mut self.playback);
        self.record_plays();
        if self.last_save.elapsed() >= Duration::from_secs(2) {
            self.last_save = Instant::now();
            self.save_session();
        }
        if let Some(media) = self.media.as_mut() {
            media.handle_events(&mut self.playback);
            media.publish(&self.playback);
        }
        if self.playback.is_playing() {
            // Keep the slider moving while audio plays.
            ctx.request_repaint_after(Duration::from_millis(200));
        }

        let mut next_tab = self.tab;
        // Open the sidebar when the window is wide and collapse it when it is narrow. Changing the width
        // across the line resets the sidebar; the toggle can change it by hand after that.
        let narrow = ui.ctx().content_rect().width() < NARROW_WINDOW;
        if narrow != self.narrow {
            self.narrow = narrow;
            self.sidebar_open = !narrow;
            // A side panel beside the main view leaves no room in a narrow window, so it closes too.
            if narrow {
                self.playback.panel = Panel::Closed;
            }
        }
        let mut toggle_sidebar = false;
        // The sidebar and the playback bar colours for the current theme. The sidebar runs the full
        // window height.
        let dark = ui.visuals().dark_mode;
        let (sidebar_fill, bar_fill) = if dark {
            (DARK_SIDEBAR, DARK_BAR)
        } else {
            (LIGHT_SIDEBAR, LIGHT_BAR)
        };
        let highlight = if dark {
            sidebar_fill.gamma_multiply(1.5)
        } else {
            sidebar_fill.lerp_to_gamma(egui::Color32::BLACK, 0.06)
        };
        // The bar is declared first so it spans the whole window, with the sidebar and the main pane
        // above it. It has no top border.
        // The top edge of the playback bar, so the panels beside the main view can stop there.
        let mut bar_top = f32::INFINITY;
        if self.playback.has_bar() {
            let bar = egui::Panel::bottom("now_playing")
                .show_separator_line(false)
                .frame(egui::Frame::side_top_panel(ui.style()).fill(bar_fill))
                .show(ui, |ui| {
                    self.playback.show(ui, narrow);
                });
            bar_top = bar.response.rect.top();
        }
        egui::Panel::left("sidebar")
            .exact_size(if self.sidebar_open {
                SIDEBAR_WIDTH
            } else {
                SIDEBAR_RAIL
            })
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(sidebar_fill)
                    .inner_margin(egui::Margin::symmetric(10, 12)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);
                if sidebar_toggle(ui, self.sidebar_open).clicked() {
                    toggle_sidebar = true;
                }
                // Buttons grow when hovered by default, which made the items jump under the pointer.
                let widgets = &mut ui.style_mut().visuals.widgets;
                widgets.inactive.expansion = 0.0;
                widgets.hovered.expansion = 0.0;
                widgets.active.expansion = 0.0;
                for (tab, icon, label) in [
                    (Tab::Search, icons::MAGNIFYING_GLASS, "Search"),
                    (Tab::Albums, icons::VINYL_RECORD, "Albums"),
                    (Tab::Recent, icons::CLOCK_COUNTER_CLOCKWISE, "Recent"),
                    (Tab::Artists, icons::MICROPHONE_STAGE, "Artists"),
                    (Tab::Playlists, icons::PLAYLIST, "Playlists"),
                    (Tab::Radio, icons::RADIO, "Radio"),
                    (Tab::Movies, icons::FILM_STRIP, "Movies"),
                    (Tab::Settings, icons::GEAR, "Settings"),
                ] {
                    let selected = self.tab == tab;
                    let width = ui.available_width();
                    let entry = sidebar_entry(
                        ui,
                        width,
                        icon,
                        label,
                        selected,
                        highlight,
                        self.sidebar_open,
                    );
                    if entry.clicked() {
                        next_tab = tab;
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    if self.scan.is_some() {
                        if self.sidebar_open {
                            ui.weak("Scanning…");
                        }
                        ui.spinner();
                    }
                });
            });
        if toggle_sidebar {
            self.sidebar_open = !self.sidebar_open;
        }
        if next_tab == Tab::Search && self.tab != Tab::Search {
            self.search.focus();
        }
        self.tab = next_tab;

        // A speed chosen in the bar is kept in the settings, like the others.
        if let Some(speed) = self.playback.take_speed_change() {
            self.settings.playback_speed = speed;
            if let Err(e) = self.settings.save(&self.paths.settings) {
                self.status = format!("Could not save settings: {e}");
            }
        }
        // An album clicked in the bar opens on the album page.
        if let Some(name) = self.playback.take_album_request() {
            if self.albums.open_by_name(&self.library, &name) {
                self.tab = Tab::Albums;
            } else {
                self.status = format!("Could not find the album {name}");
            }
        }
        let panel = self.playback.panel;
        let mut lyrics_outcome = side::LyricsOutcome::default();
        if self.playback.has_current() && panel != Panel::Closed {
            egui::Panel::right("side_panel")
                .default_size(320.0)
                .resizable(false)
                .show_separator_line(false)
                .frame(egui::Frame::new().fill(sidebar_fill))
                .show(ui, |ui| {
                    clip_above(ui, bar_top);
                    match panel {
                        Panel::Queue => queue::show(ui, &mut self.queue_view, &mut self.playback),
                        Panel::Lyrics => {
                            lyrics_outcome = self.side.lyrics(
                                ui,
                                &self.library,
                                self.playback.current_item().as_deref(),
                                self.playback.position().unwrap_or_default(),
                                self.settings.lyrics_online,
                            );
                        }
                        Panel::Info => {
                            let playing = self.playback.current_item();
                            let shown = self.info_track.as_ref().or(playing.as_deref());
                            self.side.info(ui, &self.library, shown)
                        }
                        Panel::Closed => {}
                    }
                });
        }
        // Tags written in the info panel change the library, so its views reload.
        if self.side.take_library_changed() {
            self.reload_views();
        }
        if let Some(at) = lyrics_outcome.seek {
            self.playback.seek(at.as_secs_f64());
        }
        if lyrics_outcome.enable {
            self.set_lyrics_online(true);
        }
        let mut queue_requests: Vec<MenuRequest> = Vec::new();
        let playlist_summaries = self.playlists.summaries().to_vec();
        let mut action = None;
        let mut search_outcome = None;
        let mut opened_album = None;
        let mut radio_action = None;
        let mut play = None;
        // The Radio page pads its own content, so its scrollbar can reach the window edge.
        // The album page scrolls with the window's right edge, so it keeps its own padding.
        let full_width = self.tab == Tab::Radio
            || self.tab == Tab::Settings
            || (self.tab == Tab::Artists && !self.artists.showing_detail())
            || (self.tab == Tab::Playlists && !self.playlists.showing_detail())
            || (self.tab == Tab::Albums && self.albums.showing_detail());
        let main_frame = if full_width {
            egui::Frame::central_panel(ui.style()).inner_margin(egui::Margin::same(0))
        } else {
            egui::Frame::central_panel(ui.style()).inner_margin(egui::Margin::symmetric(32, 32))
        };
        egui::CentralPanel::default()
            .frame(main_frame)
            .show(ui, |ui| match self.tab {
                Tab::Albums => {
                    play = self.albums.show(
                        ui,
                        &self.library,
                        self.playback.current_id(),
                        &mut queue_requests,
                        self.playlists.summaries(),
                    );
                }
                Tab::Search => {
                    search_outcome = Some(self.search.show(
                        ui,
                        &mut self.albums,
                        &self.library,
                        self.playback.current_id(),
                        &mut queue_requests,
                        self.playlists.summaries(),
                    ));
                }
                Tab::Recent => {
                    opened_album = self.recent.show(ui, &mut self.albums, &self.library);
                }
                Tab::Artists => {
                    opened_album = self.artists.show(ui, &mut self.albums, &self.library);
                }
                Tab::Radio => {
                    let on_air = self.playback.radio_station().cloned();
                    radio_action = self.radio.show(
                        ui,
                        &self.library,
                        on_air.as_ref(),
                        !self.playback.is_playing(),
                        self.station_sync.is_some(),
                    );
                }
                Tab::Playlists => {
                    if let Some(request) = self.playlists.show(
                        ui,
                        &self.library,
                        self.playback.current_id(),
                        &mut queue_requests,
                        &playlist_summaries,
                    ) {
                        play = Some(request);
                    }
                }
                Tab::Movies => self.movies.show(ui),
                Tab::Settings => {
                    action = settings_view::show(
                        ui,
                        &self.settings,
                        self.scan.is_some(),
                        &self.status,
                        &self.folder_files,
                    );
                }
            });

        if let Some(outcome) = search_outcome {
            if outcome.play.is_some() {
                play = outcome.play;
            }
            if outcome.radio.is_some() {
                radio_action = outcome.radio;
            }
            if let Some(i) = outcome.open_album {
                self.albums.open(&self.library, i);
                self.tab = Tab::Albums;
            }
            if let Some(id) = outcome.open_playlist {
                self.playlists.open(id, &self.library);
                self.tab = Tab::Playlists;
            }
        }
        if let Some(i) = opened_album {
            self.albums.open(&self.library, i);
            self.tab = Tab::Albums;
        }
        match radio_action {
            Some(radio::RadioAction::Play(station)) => self.playback.tune(station),
            Some(radio::RadioAction::TogglePlayback) => self.playback.toggle(),
            Some(radio::RadioAction::TuneInAndPlay(station)) => {
                if let Err(e) = self.library.tune_in(&station) {
                    self.status = format!("Could not tune in: {e}");
                }
                self.radio.reload(&self.library);
                self.playback.tune(station);
            }
            Some(radio::RadioAction::TuneOut(uuid)) => {
                if let Err(e) = self.library.tune_out(&uuid) {
                    self.status = format!("Could not tune out: {e}");
                }
                self.radio.reload(&self.library);
            }
            Some(radio::RadioAction::Sync) => self.start_station_sync(&ctx),
            None => {}
        }
        if let Some(request) = play {
            self.playback.play_queue(request.tracks, request.index);
        }
        for request in queue_requests {
            match request {
                MenuRequest::PlayNext(item) => self.playback.play_next(item),
                MenuRequest::AddToQueue(item) => self.playback.add_to_queue(item),
                MenuRequest::ShowInfo(item) => {
                    self.info_track = Some(item);
                    self.playback.panel = Panel::Info;
                }
                MenuRequest::AddToPlaylist { playlist, media_id } => {
                    match self.library.add_to_playlist(playlist, &[media_id]) {
                        Ok(()) => self.playlists.reload(&self.library),
                        Err(e) => self.status = format!("Could not add to playlist: {e}"),
                    }
                }
            }
        }

        match action {
            Some(settings_view::Action::AddFolder { path, kind }) => {
                if self.settings.library_roots.contains(&path) {
                    self.status = format!("{} is already in the library.", path.display());
                } else {
                    self.settings.library_roots.push(path.clone());
                    self.settings.set_library_kind(&path, kind);
                    self.save_and_watch(&ctx);
                    self.enqueue(&ctx, vec![path]);
                }
            }
            Some(settings_view::Action::SetFolderKind { index, kind }) => {
                if let Some(root) = self.settings.library_roots.get(index).cloned() {
                    self.settings.set_library_kind(&root, kind);
                    if let Err(e) = self.settings.save(&self.paths.settings) {
                        self.status = format!("Could not save settings: {e}");
                    }
                }
            }
            Some(settings_view::Action::RemoveFolder(i)) => {
                if i < self.settings.library_roots.len() && self.scan.is_none() {
                    let root = self.settings.library_roots.remove(i);
                    self.settings.forget_library_kind(&root);
                    self.queue.retain(|queued| *queued != root);
                    self.status = match self.library.forget_root(&root) {
                        Ok(removed) => format!("Removed {} ({removed} files).", root.display()),
                        Err(e) => format!("Could not remove {}: {e}", root.display()),
                    };
                    self.reload_views();
                    self.save_and_watch(&ctx);
                }
            }
            Some(settings_view::Action::RescanFolder(i)) => {
                if let Some(root) = self.settings.library_roots.get(i).cloned() {
                    self.enqueue(&ctx, vec![root]);
                }
            }
            Some(settings_view::Action::RescanAll) => {
                let roots = self.settings.library_roots.clone();
                self.enqueue(&ctx, roots);
            }
            Some(settings_view::Action::SetLyricsOnline(on)) => self.set_lyrics_online(on),
            Some(settings_view::Action::SetCrossfade(secs)) => {
                self.settings.crossfade_secs = secs;
                if let Err(e) = self.settings.save(&self.paths.settings) {
                    self.status = format!("Could not save settings: {e}");
                }
            }
            Some(settings_view::Action::SetReplayGain(mode)) => {
                self.settings.replay_gain = mode;
                self.playback.set_replay_gain(mode);
                if let Err(e) = self.settings.save(&self.paths.settings) {
                    self.status = format!("Could not save settings: {e}");
                }
            }
            Some(settings_view::Action::SetTheme(theme)) => {
                self.settings.theme = theme;
                ctx.set_theme(preference(theme));
                if let Err(e) = self.settings.save(&self.paths.settings) {
                    self.status = format!("Could not save settings: {e}");
                }
            }
            None => {}
        }
    }
}

/// A page that scrolls as a whole: its title and content sit in one scroll area, padded inside it by `margin`,
/// so the scroll bar is at the window's right edge.
pub(crate) fn page<R>(
    ui: &mut egui::Ui,
    margin: egui::Margin,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(margin)
                .show(ui, add_contents)
                .inner
        })
        .inner
}

/// Clip what is drawn in `ui` to above `bottom`, so a panel cannot draw over the playback bar.
fn clip_above(ui: &mut egui::Ui, bottom: f32) {
    let clip = ui.clip_rect();
    ui.set_clip_rect(egui::Rect::from_min_max(
        clip.min,
        egui::pos2(clip.max.x, clip.max.y.min(bottom)),
    ));
}

/// Window width below which the sidebar starts collapsed.
const NARROW_WINDOW: f32 = 720.0;
/// The narrowest width the window can be resized to, in points.
const MIN_WINDOW_WIDTH: f32 = 400.0;
/// Width of the sidebar when it is open, and of the rail it collapses to.
const SIDEBAR_WIDTH: f32 = 200.0;
const SIDEBAR_RAIL: f32 = 52.0;

/// The button that opens and closes the sidebar: three lines, drawn so it needs no font glyph.
fn sidebar_toggle(ui: &mut egui::Ui, open: bool) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), egui::Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let response = response.on_hover_text(if open { "Hide sidebar" } else { "Show sidebar" });
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, 6.0, ui.visuals().widgets.hovered.weak_bg_fill);
    }
    let centre = rect.center();
    let colour = ui.visuals().text_color();
    for dy in [-5.0, 0.0, 5.0] {
        painter.line_segment(
            [
                egui::pos2(centre.x - 8.0, centre.y + dy),
                egui::pos2(centre.x + 8.0, centre.y + dy),
            ],
            egui::Stroke::new(1.6, colour),
        );
    }
    response
}

/// A sidebar entry: its icon, with its label beside it when the sidebar is open. The selected entry has
/// a background slightly different from the sidebar.
fn sidebar_entry(
    ui: &mut egui::Ui,
    width: f32,
    icon: &str,
    label: &str,
    selected: bool,
    highlight: egui::Color32,
    open: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 34.0), egui::Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let colour = ui.visuals().text_color();
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, 6.0, highlight);
    }
    if open {
        let centre = egui::pos2(rect.left() + 18.0, rect.center().y);
        icons::paint(painter, centre, icon, 18.0, colour);
        painter.text(
            egui::pos2(rect.left() + 38.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(14.0),
            colour,
        );
    } else {
        icons::paint(painter, rect.center(), icon, 20.0, colour);
    }
    response
}

/// Whether the station index is missing, or more than a day old.
fn station_index_stale(library: &Library) -> bool {
    let synced: Option<i64> = library
        .meta("stations_synced")
        .ok()
        .flatten()
        .and_then(|value| value.parse().ok());
    match synced {
        None => true,
        Some(at) => now_secs() - at >= 24 * 60 * 60,
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Fetch the station list and store it, on the calling thread. Returns how many stations were
/// stored.
fn sync_stations(db: &Path) -> Result<usize, String> {
    let stations = player_core::fetch_all_stations()?;
    let mut library = Library::open(db).map_err(|e| e.to_string())?;
    let count = library
        .replace_stations(&stations)
        .map_err(|e| e.to_string())?;
    library
        .set_meta("stations_synced", &now_secs().to_string())
        .map_err(|e| e.to_string())?;
    Ok(count)
}
