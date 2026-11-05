//! Desktop UI (egui via eframe): Albums, Movies, Settings views, and the playback bar.

mod albums;
mod format;
mod movies;
mod playback;
mod settings_view;

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use eframe::egui;
use player_core::{Library, ScanStats, Settings, Theme};

use playback::Playback;

/// Where the app keeps its library database and settings file.
pub struct Paths {
    pub db: PathBuf,
    pub settings: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Albums,
    Movies,
    Settings,
}

type ScanResult = Result<ScanStats, String>;

struct App {
    paths: Paths,
    settings: Settings,
    library: Library,
    tab: Tab,
    albums: albums::AlbumsView,
    movies: movies::MoviesView,
    playback: Playback,
    /// Set while a background scan is running.
    scan: Option<Receiver<ScanResult>>,
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
        let tab = if settings.library_root.is_some() {
            Tab::Albums
        } else {
            Tab::Settings
        };
        let mut app = Self {
            paths,
            settings,
            library,
            tab,
            albums: Default::default(),
            movies: Default::default(),
            playback: Playback::new(),
            scan: None,
            status: String::new(),
        };
        app.reload_views();
        match app.settings.library_root.clone() {
            Some(root) => app.start_scan(&ctx, root),
            None => app.status = "Choose your library folder to get started.".into(),
        }
        app
    }

    fn reload_views(&mut self) {
        self.albums.reload(&self.library);
        self.movies.reload(&self.library);
    }

    /// Scan `root` on a worker thread. The worker opens its own connection,
    /// so the UI stays responsive while files are read.
    fn start_scan(&mut self, ctx: &egui::Context, root: PathBuf) {
        if self.scan.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let db = self.paths.db.clone();
        let ctx = ctx.clone();
        let status_root = root.clone();
        thread::spawn(move || {
            let result = Library::open(&db)
                .and_then(|mut lib| lib.scan(&root))
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        self.scan = Some(rx);
        self.status = format!("Scanning {}…", status_root.display());
    }

    fn poll_scan(&mut self) {
        let Some(rx) = &self.scan else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(stats)) => {
                self.status = format!(
                    "Scan complete: {} added or changed, {} unchanged, {} removed.",
                    stats.indexed, stats.unchanged, stats.removed
                );
                self.scan = None;
                self.reload_views();
            }
            Ok(Err(message)) => {
                self.status = format!("Scan failed: {message}");
                self.scan = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.status = "Scan stopped unexpectedly.".into();
                self.scan = None;
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_scan();
        self.playback.advance_if_finished();
        let ctx = ui.ctx().clone();
        if self.playback.is_playing() {
            // Keep the slider moving while audio plays.
            ctx.request_repaint_after(Duration::from_millis(200));
        }

        let mut next_tab = self.tab;
        egui::Panel::top("tabs").show(ui, |ui| {
            ui.horizontal(|ui| {
                for (tab, label) in [
                    (Tab::Albums, "Albums"),
                    (Tab::Movies, "Movies"),
                    (Tab::Settings, "Settings"),
                ] {
                    if ui.selectable_label(self.tab == tab, label).clicked() {
                        next_tab = tab;
                    }
                }
                if self.scan.is_some() {
                    ui.spinner();
                    ui.weak("Scanning…");
                }
            });
        });
        self.tab = next_tab;

        if self.playback.has_current() {
            egui::Panel::bottom("now_playing").show(ui, |ui| {
                self.playback.show(ui);
            });
        }
        let mut action = None;
        let mut play = None;
        egui::CentralPanel::default().show(ui, |ui| match self.tab {
            Tab::Albums => {
                play = self
                    .albums
                    .show(ui, &self.library, self.playback.current_id());
            }
            Tab::Movies => self.movies.show(ui),
            Tab::Settings => {
                action = settings_view::show(ui, &self.settings, self.scan.is_some(), &self.status);
            }
        });

        if let Some(request) = play {
            self.playback.play_queue(request.tracks, request.index);
        }

        match action {
            Some(settings_view::Action::ChooseFolder(root)) => {
                self.settings.library_root = Some(root.clone());
                match self.settings.save(&self.paths.settings) {
                    Ok(()) => self.start_scan(&ctx, root),
                    Err(e) => self.status = format!("Could not save settings: {e}"),
                }
            }
            Some(settings_view::Action::Rescan) => {
                if let Some(root) = self.settings.library_root.clone() {
                    self.start_scan(&ctx, root);
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
