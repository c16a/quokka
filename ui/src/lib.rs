//! Desktop UI (egui via eframe): Albums, Movies, Settings views, and the playback bar.

mod albums;
mod artists;
mod colour;
mod covers;
mod fonts;
mod format;
mod i18n;
mod icons;
mod media;
mod mesh_background;
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
mod metal_video;
mod movies;
mod playback;
mod playlists;
mod progress_writer;
mod queue;
mod radio;
mod recent;
mod search;
mod settings_view;
mod side;
mod slider;
pub mod squircle;
pub mod video_gpu;

use i18n::Text;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;
use notify::{RecursiveMode, Watcher};
use quokka_core::tmdb::Tmdb;
use quokka_core::{Library, MediaItem, Progress, ScanStats, Session, Settings, Theme};

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

/// What scanning one folder found, and what the TMDb lookups after it did.
struct Scanned {
    stats: ScanStats,
    /// Movies matched on TMDb by the scan.
    matched: usize,
    /// Why the TMDb lookups stopped early, if they did. They are tried again on the next scan.
    lookup_error: Option<String>,
}

/// The outcome of scanning one folder.
type ScanResult = Result<Scanned, String>;

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
    report: Vec<Text>,
    /// Set while the scan worker runs the processing steps, after the files are read.
    processing: Arc<AtomicBool>,
    /// Watches the library folder. Dropping it stops the watch.
    watcher: Option<notify::RecommendedWatcher>,
    change_tx: Sender<()>,
    changes: Receiver<()>,
    /// When to rescan after changes, once the library has been quiet for a while.
    rescan_at: Option<Instant>,
    /// The playback state last written to disk, and when the state was last checked.
    session_saved: Option<Session>,
    last_save: Instant,
    /// Saves how far the playing file has been played, once a second.
    progress: Option<progress_writer::ProgressWriter>,
    /// The last position saved, so an unchanged position is not written again.
    progress_sent: Option<(PathBuf, f64)>,
    last_progress: Instant,
    /// The movie whose saved progress is cached in `movie_progress`. The progress is read again
    /// when the open movie changes or once a second.
    movie_progress_for: Option<PathBuf>,
    movie_progress: Option<Progress>,
    /// Whether the video window is open. A movie that starts opens it again.
    #[cfg(target_os = "macos")]
    video_window_open: bool,
    /// Whether a video was showing last frame, to see when a new one starts.
    #[cfg(target_os = "macos")]
    video_was_showing: bool,
    /// Whether the movie was playing last frame, to see when playback starts again.
    #[cfg(target_os = "macos")]
    video_was_playing: bool,
    status: Text,
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
            .with_title("Quokka")
            .with_inner_size([1100.0, 720.0])
            // The narrowest the window can be dragged. Narrow layouts still need room for the page padding and the rail.
            .with_min_inner_size([MIN_WINDOW_WIDTH, 0.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Quokka",
        options,
        Box::new(move |cc| {
            if let Some(render_state) = cc.wgpu_render_state.as_ref() {
                mesh_background::install(render_state);
            }
            #[cfg(target_os = "macos")]
            if let Some(render_state) = cc.wgpu_render_state.as_ref() {
                video_gpu::install(render_state);
            }
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
        let progress = progress_writer::ProgressWriter::start(&paths.db);
        let movies = movies::MoviesView::new(paths.db.clone());
        let mut albums = albums::AlbumsView::default();
        albums.set_db(paths.db.clone());
        // Artist photos are cached beside the library database.
        let photo_dir = paths
            .db
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("artist-photos");
        i18n::set_language(settings.language.as_deref());
        fonts::install(&ctx);
        fonts::use_language(&ctx, i18n::current_tag());
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
            albums,
            search: Default::default(),
            movies,
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
            processing: Arc::new(AtomicBool::new(false)),
            watcher: None,
            change_tx,
            changes,
            rescan_at: None,
            session_saved: None,
            last_save: Instant::now(),
            progress,
            progress_sent: None,
            last_progress: Instant::now(),
            movie_progress_for: None,
            movie_progress: None,
            #[cfg(target_os = "macos")]
            video_window_open: false,
            #[cfg(target_os = "macos")]
            video_was_showing: false,
            #[cfg(target_os = "macos")]
            video_was_playing: false,
            status: Text::empty(),
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
            app.status = Text::of(i18n::add_a_folder_to_get_started);
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
        // If the saved current track is gone, start from the first one at the beginning. The saved
        // progress is newer than the session's position, so it wins when there is one.
        let position = match index.and_then(|i| items.get(i)) {
            Some(current) => match self.library.progress(&current.path) {
                Ok(Some(saved)) => saved.position_secs,
                _ => session.position_secs,
            },
            None => 0.0,
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
        self.status = Text::of(i18n::syncing_the_radio_station_list);
    }

    /// Collect a finished station sync, if there is one.
    fn poll_station_sync(&mut self) {
        let Some(rx) = self.station_sync.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(count)) => {
                self.status =
                    Text::filled(i18n::radio_station_count, vec![count.to_string().into()]);
                self.radio.reload(&self.library);
            }
            Ok(Err(e)) => {
                self.status = Text::filled(i18n::radio_sync_failed, vec![e.to_string().into()])
            }
            Err(TryRecvError::Empty) => self.station_sync = Some(rx),
            Err(TryRecvError::Disconnected) => {
                self.status = Text::of(i18n::radio_station_sync_stopped_unexpectedly);
            }
        }
    }

    /// Turn lookups on or off for lyrics, and save the choice.
    fn set_lyrics_online(&mut self, on: bool) {
        self.settings.lyrics_online = on;
        if let Err(e) = self.settings.save(&self.paths.settings) {
            self.status = Text::filled(i18n::could_not_save_settings, vec![e.to_string().into()]);
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
    /// Save how far the current file has been played, if that has moved since the last save.
    fn save_progress(&mut self) {
        let Some(writer) = self.progress.as_ref() else {
            return;
        };
        let Some((path, position, duration)) = self.playback.progress_snapshot() else {
            return;
        };
        if let Some((sent_path, sent_position)) = &self.progress_sent
            && *sent_path == path
            && (sent_position - position).abs() < 0.05
        {
            return;
        }
        writer.save(progress_writer::Save {
            path: path.clone(),
            position_secs: position,
            duration_secs: duration,
        });
        self.progress_sent = Some((path, position));
    }

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
        let processing = Arc::clone(&self.processing);
        let tmdb_key = self.settings.tmdb_api_key.clone();
        thread::spawn(move || {
            let result = Library::open(&db)
                .and_then(|mut lib| {
                    let stats = lib.scan_files(&job)?;
                    // The flag shows while the processing steps run, after the files are read.
                    processing.store(true, Ordering::Relaxed);
                    ctx.request_repaint();
                    let done = lib.process_pending();
                    processing.store(false, Ordering::Relaxed);
                    done?;
                    // With a TMDb key, movies that are new or changed are matched now. A failed
                    // lookup doesn't fail the scan: it is reported, and tried again next time.
                    let (matched, lookup_error) = match tmdb_key {
                        Some(key) => match lib.match_pending_movies(&Tmdb::new(key)) {
                            Ok(saved) => (saved, None),
                            Err(e) => (0, Some(e.to_string())),
                        },
                        None => (0, None),
                    };
                    Ok(Scanned {
                        stats,
                        matched,
                        lookup_error,
                    })
                })
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        let folder = root.display().to_string();
        self.status = match self.queue.len() {
            0 => Text::filled(i18n::scanning_folder, vec![folder.to_string().into()]),
            n => Text::filled(
                i18n::scanning_folder_queued,
                vec![folder.to_string().into(), n.to_string().into()],
            ),
        };
        self.scan = Some((root, rx));
    }

    /// Save the folder list and watch all of its folders again.
    fn save_and_watch(&mut self, ctx: &egui::Context) {
        if let Err(e) = self.settings.save(&self.paths.settings) {
            self.status = Text::filled(i18n::could_not_save_settings, vec![e.to_string().into()]);
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
            Ok(result) => result.map_err(Text::from),
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err(Text::of(i18n::the_scan_stopped_unexpectedly)),
        };
        let name = root.display().to_string();
        self.scan = None;
        let line = match result {
            Ok(scanned) => {
                let stats = scanned.stats;
                let mut sentences = vec![Text::filled(
                    i18n::scan_summary,
                    vec![
                        name.to_string().into(),
                        stats.indexed.to_string().into(),
                        stats.unchanged.to_string().into(),
                        stats.removed.to_string().into(),
                    ],
                )];
                if scanned.matched > 0 {
                    sentences.push(Text::filled(
                        i18n::matched_on_tmdb,
                        vec![scanned.matched.to_string().into()],
                    ));
                }
                if let Some(e) = scanned.lookup_error {
                    sentences.push(Text::filled(
                        i18n::tmdb_lookup_stopped,
                        vec![e.to_string().into()],
                    ));
                }
                Text::joined(sentences, " ")
            }
            // One unreadable folder (for example an unplugged drive) doesn't stop the others.
            Err(e) => Text::filled(
                i18n::could_not_be_scanned,
                vec![name.to_string().into(), e.to_string().into()],
            ),
        };
        self.report.push(line);
        self.reload_views();
        if self.queue.is_empty() {
            self.status = Text::joined(self.report.clone(), "\n");
        }
        self.start_next(ctx);
    }
}

impl App {
    /// Show a movie's picture in its own window. The window opens when a movie starts or playback
    /// starts again. Closing it pauses the movie and saves where it is, so playing again resumes there.
    #[cfg(target_os = "macos")]
    fn show_video_window(&mut self, ctx: &egui::Context) {
        let showing = self.playback.video_showing();
        let playing = self.playback.is_playing();
        if (showing && !self.video_was_showing) || (playing && !self.video_was_playing) {
            self.video_window_open = true;
        }
        self.video_was_showing = showing;
        self.video_was_playing = playing;
        if !showing || !self.video_window_open {
            return;
        }
        let mut closed = false;
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("quokka-video"),
            egui::ViewportBuilder::default()
                .with_title(i18n::video())
                .with_inner_size([960.0, 540.0]),
            |ui, _class| {
                if ui.ctx().input(|input| input.viewport().close_requested()) {
                    self.video_window_open = false;
                    closed = true;
                }
                // The video window is always dark, as the picture is, so its controls are too.
                ui.style_mut().visuals = egui::Visuals::dark();
                let fullscreen = ui
                    .ctx()
                    .input(|input| input.viewport().fullscreen)
                    .unwrap_or(false);
                let bar = || {
                    egui::Frame::new()
                        .fill(egui::Color32::from_black_alpha(184))
                        .inner_margin(egui::Margin::symmetric(20, 12))
                };
                if let Some(title) = self.playback.video_title() {
                    egui::Panel::top("video_title")
                        .show_separator_line(false)
                        .frame(bar())
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(title).size(16.0).strong());
                        });
                }
                // The keys (F, Escape) and the full-screen button both ask for a change.
                let mut toggle_fullscreen =
                    playback::handle_video_keys(ui.ctx(), &mut self.playback, fullscreen);
                egui::Panel::bottom("video_controls")
                    .show_separator_line(false)
                    .frame(bar())
                    .show(ui, |ui| {
                        toggle_fullscreen |= self.playback.show_video_controls(ui, fullscreen);
                    });
                if toggle_fullscreen {
                    ui.ctx().send_viewport_cmd_to(
                        egui::ViewportId::from_hash_of("quokka-video"),
                        egui::ViewportCommand::Fullscreen(!fullscreen),
                    );
                }
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::BLACK))
                    .show(ui, |ui| self.playback.show_video(ui));
            },
        );
        if closed {
            // Closing the window stops the movie and frees its decoders and sound output. Where it
            // was is kept, so playing it again resumes from there.
            self.playback.close_video();
            self.save_session();
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_scan(ui.ctx());
        self.poll_station_sync();
        if self.playback.video_active() {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
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
        let progress_due = self.last_progress.elapsed() >= Duration::from_secs(1);
        if progress_due {
            self.last_progress = Instant::now();
            self.save_progress();
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
                    (Tab::Search, icons::MAGNIFYING_GLASS, i18n::search()),
                    (Tab::Albums, icons::VINYL_RECORD, i18n::albums()),
                    (Tab::Recent, icons::CLOCK_COUNTER_CLOCKWISE, i18n::recent()),
                    (Tab::Artists, icons::MICROPHONE_STAGE, i18n::artists()),
                    (Tab::Playlists, icons::PLAYLIST, i18n::playlists()),
                    (Tab::Radio, icons::RADIO, i18n::radio()),
                    (Tab::Movies, icons::FILM_STRIP, i18n::movies()),
                    (Tab::Settings, icons::GEAR, i18n::settings()),
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
                            ui.weak(i18n::scanning());
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
                self.status =
                    Text::filled(i18n::could_not_save_settings, vec![e.to_string().into()]);
            }
        }
        // An album clicked in the bar opens on the album page.
        if let Some(name) = self.playback.take_album_request() {
            if self.albums.open_by_name(&self.library, &name) {
                self.tab = Tab::Albums;
            } else {
                self.status =
                    Text::filled(i18n::could_not_find_album, vec![name.to_string().into()]);
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
        let mut movie_play = None;
        let movie_path = if self.tab == Tab::Movies {
            self.movies.open_movie_path()
        } else {
            None
        };
        if movie_path != self.movie_progress_for || (progress_due && movie_path.is_some()) {
            self.movie_progress = movie_path
                .as_deref()
                .and_then(|path| self.library.progress(path).ok().flatten());
            self.movie_progress_for = movie_path;
        }
        let movie_progress = self.movie_progress.clone();
        // The Radio page pads its own content, so its scrollbar can reach the window edge.
        // The album page scrolls with the window's right edge, so it keeps its own padding.
        let full_width = self.tab == Tab::Radio
            || self.tab == Tab::Settings
            || (self.tab == Tab::Artists && !self.artists.showing_detail())
            || (self.tab == Tab::Playlists && !self.playlists.showing_detail())
            || (self.tab == Tab::Albums && self.albums.showing_detail())
            || (self.tab == Tab::Movies && self.movies.showing_page());
        #[cfg(target_os = "macos")]
        self.show_video_window(&ui.ctx().clone());
        let main_frame = if full_width {
            egui::Frame::central_panel(ui.style()).inner_margin(egui::Margin::same(0))
        } else {
            egui::Frame::central_panel(ui.style()).inner_margin(egui::Margin::symmetric(32, 32))
        };
        // A movie or album with cover art is drawn on the cover's dominant colour.
        let page = match self.tab {
            Tab::Movies => self.movies.page_background(),
            Tab::Albums => self.albums.page_background(),
            _ => None,
        };
        let main_frame = match page {
            Some(colour) => main_frame.fill(colour),
            None => main_frame,
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
                Tab::Movies => {
                    if let Some(request) = self.movies.show(
                        ui,
                        self.settings.tmdb_api_key.as_deref(),
                        movie_progress.clone(),
                    ) {
                        movie_play = Some(request);
                    }
                }
                Tab::Settings => {
                    action = settings_view::show(
                        ui,
                        &self.settings,
                        self.scan.is_some(),
                        &self.status.to_string(),
                        &self.folder_files,
                        self.processing.load(Ordering::Relaxed),
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
                    self.status = Text::filled(i18n::could_not_tune_in, vec![e.to_string().into()]);
                }
                self.radio.reload(&self.library);
                self.playback.tune(station);
            }
            Some(radio::RadioAction::TuneOut(uuid)) => {
                if let Err(e) = self.library.tune_out(&uuid) {
                    self.status =
                        Text::filled(i18n::could_not_tune_out, vec![e.to_string().into()]);
                }
                self.radio.reload(&self.library);
            }
            Some(radio::RadioAction::Sync) => self.start_station_sync(&ctx),
            None => {}
        }
        if let Some(request) = play {
            self.playback.play_queue(request.tracks, request.index);
        }
        if let Some((request, start_secs)) = movie_play {
            self.playback
                .play_from(request.tracks, request.index, start_secs);
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
                        Err(e) => {
                            self.status = Text::filled(
                                i18n::could_not_add_to_playlist,
                                vec![e.to_string().into()],
                            )
                        }
                    }
                }
            }
        }

        match action {
            Some(settings_view::Action::AddFolder { path, kind }) => {
                if self.settings.library_roots.contains(&path) {
                    self.status = Text::filled(
                        i18n::already_in_library,
                        vec![path.display().to_string().into()],
                    );
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
                        self.status =
                            Text::filled(i18n::could_not_save_settings, vec![e.to_string().into()]);
                    }
                }
            }
            Some(settings_view::Action::RemoveFolder(i)) => {
                if i < self.settings.library_roots.len() && self.scan.is_none() {
                    let root = self.settings.library_roots.remove(i);
                    self.settings.forget_library_kind(&root);
                    self.queue.retain(|queued| *queued != root);
                    self.status = match self.library.forget_root(&root) {
                        Ok(removed) => Text::filled(
                            i18n::removed_folder,
                            vec![
                                root.display().to_string().into(),
                                Text::new(move || i18n::file_count(removed)),
                            ],
                        ),
                        Err(e) => Text::filled(
                            i18n::could_not_remove_folder,
                            vec![root.display().to_string().into(), e.to_string().into()],
                        ),
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
                    self.status =
                        Text::filled(i18n::could_not_save_settings, vec![e.to_string().into()]);
                }
            }
            Some(settings_view::Action::SetTmdbKey(key)) => {
                self.settings.tmdb_api_key = (!key.is_empty()).then_some(key);
                match self.settings.save(&self.paths.settings) {
                    Ok(()) => self.status = Text::of(i18n::saved_the_tmdb_key),
                    Err(e) => {
                        self.status =
                            Text::filled(i18n::could_not_save_settings, vec![e.to_string().into()])
                    }
                }
            }
            Some(settings_view::Action::SetReplayGain(mode)) => {
                self.settings.replay_gain = mode;
                self.playback.set_replay_gain(mode);
                if let Err(e) = self.settings.save(&self.paths.settings) {
                    self.status =
                        Text::filled(i18n::could_not_save_settings, vec![e.to_string().into()]);
                }
            }
            Some(settings_view::Action::SetLanguage(language)) => {
                i18n::set_language(language.as_deref());
                fonts::use_language(&ctx, i18n::current_tag());
                self.settings.language = language;
                if let Err(e) = self.settings.save(&self.paths.settings) {
                    self.status =
                        Text::filled(i18n::could_not_save_settings, vec![e.to_string().into()]);
                }
            }
            Some(settings_view::Action::SetTheme(theme)) => {
                self.settings.theme = theme;
                ctx.set_theme(preference(theme));
                if let Err(e) = self.settings.save(&self.paths.settings) {
                    self.status =
                        Text::filled(i18n::could_not_save_settings, vec![e.to_string().into()]);
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
    let response = response.on_hover_text(if open {
        i18n::hide_sidebar()
    } else {
        i18n::show_sidebar()
    });
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
    let stations = quokka_core::fetch_all_stations()?;
    let mut library = Library::open(db).map_err(|e| e.to_string())?;
    let count = library
        .replace_stations(&stations)
        .map_err(|e| e.to_string())?;
    library
        .set_meta("stations_synced", &now_secs().to_string())
        .map_err(|e| e.to_string())?;
    Ok(count)
}
