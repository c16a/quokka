//! Desktop UI (egui via eframe): Albums, Movies, Settings views, and the playback bar.

mod albums;
mod artists;
mod format;
mod media;
mod movies;
mod playback;
mod playlists;
mod queue;
mod search;
mod settings_view;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;
use notify::{RecursiveMode, Watcher};
use player_core::{Library, ScanStats, Session, Settings, Theme};

use playback::{MenuRequest, Playback};

/// Where the app keeps its library database and settings file.
pub struct Paths {
    pub db: PathBuf,
    pub settings: PathBuf,
    /// Where the playback state is saved on exit, so it can resume.
    pub session: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Search,
    Albums,
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
            .with_inner_size([1100.0, 720.0]),
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
            playback: Playback::new(),
            queue_view: Default::default(),
            artists: Default::default(),
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
        };
        app.playback.set_replay_gain(app.settings.replay_gain);
        app.reload_views();
        app.restore_session();
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
        self.albums.reload(&self.library);
        self.movies.reload(&self.library);
        self.playlists.reload(&self.library);
        self.artists.reload(self.albums.albums());
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
        self.handle_changes(ui.ctx());
        self.playback.advance_if_finished();
        let ctx = ui.ctx().clone();
        playback::handle_keys(&ctx, &mut self.playback);
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
        // The sidebar fill is a shade apart from the main pane, and runs the full window height.
        let panel = ui.visuals().panel_fill;
        let sidebar_fill = if ui.visuals().dark_mode {
            panel.gamma_multiply(1.25)
        } else {
            panel.lerp_to_gamma(egui::Color32::WHITE, 0.6)
        };
        let highlight = if ui.visuals().dark_mode {
            sidebar_fill.gamma_multiply(1.5)
        } else {
            sidebar_fill.lerp_to_gamma(egui::Color32::BLACK, 0.06)
        };
        egui::Panel::left("sidebar")
            .default_size(180.0)
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
                // Buttons grow when hovered by default, which made the items jump under the pointer.
                let widgets = &mut ui.style_mut().visuals.widgets;
                widgets.inactive.expansion = 0.0;
                widgets.hovered.expansion = 0.0;
                widgets.active.expansion = 0.0;
                for (tab, label) in [
                    (Tab::Search, "Search"),
                    (Tab::Albums, "Albums"),
                    (Tab::Artists, "Artists"),
                    (Tab::Playlists, "Playlists"),
                    (Tab::Movies, "Movies"),
                    (Tab::Settings, "Settings"),
                ] {
                    let selected = self.tab == tab;
                    let width = ui.available_width();
                    if ui
                        .add_sized([width, 34.0], sidebar_item(selected, label, highlight))
                        .clicked()
                    {
                        next_tab = tab;
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    if self.scan.is_some() {
                        ui.weak("Scanning…");
                        ui.spinner();
                    }
                });
            });
        if next_tab == Tab::Search && self.tab != Tab::Search {
            self.search.focus();
        }
        self.tab = next_tab;

        if self.playback.has_current() {
            egui::Panel::bottom("now_playing").show(ui, |ui| {
                self.playback.show(ui);
            });
        }
        if self.playback.has_current() && self.playback.queue_open {
            egui::Panel::right("queue")
                .default_size(300.0)
                .resizable(false)
                .show_separator_line(false)
                .frame(egui::Frame::new().fill(sidebar_fill))
                .show(ui, |ui| {
                    queue::show(ui, &mut self.queue_view, &mut self.playback);
                });
        }
        let mut queue_requests: Vec<MenuRequest> = Vec::new();
        let mut action = None;
        let mut search_outcome = None;
        let mut artist_album = None;
        let mut play = None;
        egui::CentralPanel::default().show(ui, |ui| match self.tab {
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
            Tab::Artists => {
                artist_album = self.artists.show(ui, &mut self.albums, &self.library);
            }
            Tab::Playlists => {
                if let Some(request) =
                    self.playlists
                        .show(ui, &self.library, self.playback.current_id())
                {
                    play = Some(request);
                }
            }
            Tab::Movies => self.movies.show(ui),
            Tab::Settings => {
                action = settings_view::show(ui, &self.settings, self.scan.is_some(), &self.status);
            }
        });

        if let Some(outcome) = search_outcome {
            if outcome.play.is_some() {
                play = outcome.play;
            }
            if let Some(i) = outcome.open_album {
                self.albums.open(&self.library, i);
                self.tab = Tab::Albums;
            }
        }
        if let Some(i) = artist_album {
            self.albums.open(&self.library, i);
            self.tab = Tab::Albums;
        }
        if let Some(request) = play {
            self.playback.play_queue(request.tracks, request.index);
        }
        for request in queue_requests {
            match request {
                MenuRequest::PlayNext(item) => self.playback.play_next(item),
                MenuRequest::AddToQueue(item) => self.playback.add_to_queue(item),
                MenuRequest::AddToPlaylist { playlist, media_id } => {
                    match self.library.add_to_playlist(playlist, &[media_id]) {
                        Ok(()) => self.playlists.reload(&self.library),
                        Err(e) => self.status = format!("Could not add to playlist: {e}"),
                    }
                }
            }
        }

        match action {
            Some(settings_view::Action::AddFolder(folder)) => {
                if self.settings.library_roots.contains(&folder) {
                    self.status = format!("{} is already in the library.", folder.display());
                } else {
                    self.settings.library_roots.push(folder.clone());
                    self.save_and_watch(&ctx);
                    self.enqueue(&ctx, vec![folder]);
                }
            }
            Some(settings_view::Action::RemoveFolder(i)) => {
                if i < self.settings.library_roots.len() && self.scan.is_none() {
                    let root = self.settings.library_roots.remove(i);
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

/// A sidebar entry: label on the left, with a background only slightly different from the
/// sidebar when it is the selected one.
fn sidebar_item(selected: bool, label: &str, highlight: egui::Color32) -> egui::Button<'static> {
    let fill = if selected {
        highlight
    } else {
        egui::Color32::TRANSPARENT
    };
    egui::Button::new((label.to_owned(), egui::Atom::grow()))
        .fill(fill)
        .stroke(egui::Stroke::NONE)
        .corner_radius(6.0)
}
