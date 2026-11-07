//! Movies: a grid of portrait posters, and one movie's page with its details and a Play button.
//! The grid posters are empty rectangles until they come from a remote source. A movie's
//! description and cast come from TMDb, using the key the user set in Settings. When a file name
//! matches weakly, the page offers the candidates to pick from, each with its cover.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

use crate::colour::{self, Lab};
use eframe::egui;
use quokka_core::tmdb::{MovieInfo, Tmdb};
use quokka_core::{Library, MatchOutcome, MediaItem, Progress, finished};

use crate::albums::{PlayRequest, TILE_COVER, TILE_GAP, TILE_TEXT, hover_scroll, tile_line};
use crate::format;
use crate::icons;

/// Height of a poster in the grid. Its width is two thirds of that: posters are portrait.
const TILE_POSTER_HEIGHT: f32 = 225.0;
const TILE_WIDTH: f32 = TILE_POSTER_HEIGHT * 2.0 / 3.0;
/// Height of the poster on a movie's page.
const PAGE_POSTER_HEIGHT: f32 = 360.0;
const POSTER_RADIUS: f32 = 8.0;
const GRID_GAP: f32 = 18.0;
/// The size of a cover on a choice card.
const CHOICE_WIDTH: f32 = 92.0;
const CHOICE_HEIGHT: f32 = 138.0;
/// How much of a synopsis a choice card shows, in characters.
const SYNOPSIS_CHARS: usize = 280;
/// The width of the file information panel on a movie's page.
const INFO_PANEL_WIDTH: f32 = 330.0;

/// A background job: runs with the library database open, and says what it found.
type Job = Box<dyn FnOnce(&Library) -> quokka_core::Result<MatchOutcome> + Send>;

/// The result of a job: what it found, or an error message.
type Lookup = Result<MatchOutcome, String>;

/// A cover for a choice card, as the download leaves it.
enum Thumb {
    Loading,
    Ready(egui::TextureHandle),
    Missing,
}

/// The movie grid, or one movie's page when one is selected.
pub struct MoviesView {
    /// The library database, which each background job opens for its own connection.
    db: PathBuf,
    movies: Vec<MediaItem>,
    /// Index into `movies` of the movie whose page is open.
    selected: Option<usize>,
    /// Whether the file information panel is open on the right of the page.
    info_open: bool,
    /// Saved TMDb details by media id.
    details: HashMap<i64, MovieInfo>,
    /// The job in progress: the movie it is for, and where its result comes back.
    fetching: Option<(i64, Receiver<Lookup>)>,
    /// A background refetch of the details that lack photos, when one is running.
    refreshing: Option<Receiver<Vec<(i64, MovieInfo)>>>,
    /// Movies whose details have been refetched this session, so each is tried once.
    refresh_tried: HashSet<i64>,
    /// Candidates to pick from, for one movie. Nothing is saved until one is picked.
    choices: Option<(i64, Vec<MovieInfo>)>,
    /// What happened on the last job, when it needs saying.
    notice: Option<String>,
    /// Covers for choice cards, by TMDb poster path.
    thumbs: HashMap<String, Thumb>,
    /// The colour that covers most of each poster, by TMDb poster path.
    accents: HashMap<String, egui::Color32>,
    thumb_tx: Sender<(String, Option<egui::ColorImage>)>,
    thumb_rx: Receiver<(String, Option<egui::ColorImage>)>,
}

impl MoviesView {
    pub fn new(db: PathBuf) -> Self {
        let (thumb_tx, thumb_rx) = mpsc::channel();
        Self {
            db,
            movies: Vec::new(),
            selected: None,
            info_open: false,
            details: HashMap::new(),
            fetching: None,
            refreshing: None,
            refresh_tried: HashSet::new(),
            choices: None,
            notice: None,
            thumbs: HashMap::new(),
            accents: HashMap::new(),
            thumb_tx,
            thumb_rx,
        }
    }

    /// Whether a movie's page is open, rather than the grid.
    pub fn showing_page(&self) -> bool {
        self.selected.is_some()
    }

    /// The colour the open movie's page is drawn on: the dominant colour of its cover, once known.
    pub fn page_background(&self) -> Option<egui::Color32> {
        let index = self.selected?;
        let movie = self.movies.get(index)?;
        let path = self.poster_path(movie.id)?;
        self.accents.get(&path).copied()
    }

    /// The file of the movie whose page is open, if one is open.
    pub fn open_movie_path(&self) -> Option<PathBuf> {
        self.selected
            .and_then(|index| self.movies.get(index))
            .map(|movie| movie.path.clone())
    }

    pub fn reload(&mut self, library: &Library) {
        self.movies = library.movies().unwrap_or_else(|e| {
            eprintln!("player: could not load movies: {e}");
            Vec::new()
        });
        self.details = library.all_movie_details().unwrap_or_else(|e| {
            eprintln!("player: could not load movie details: {e}");
            HashMap::new()
        });
        if self.selected.is_some_and(|i| i >= self.movies.len()) {
            self.selected = None;
        }
    }

    /// Draw the grid, or the open movie's page. Returns a request when a movie is asked to play.
    /// `tmdb_key` is the user's TMDb key, if they have set one.
    /// `progress` is the open movie's saved position, if the page is open.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        tmdb_key: Option<&str>,
        progress: Option<Progress>,
    ) -> Option<(PlayRequest, f64)> {
        self.poll_fetch();
        self.poll_refresh();
        self.start_refresh(ui.ctx(), tmdb_key);
        self.poll_thumbs(ui.ctx());
        // While a lookup or a poster download is pending, redraw on a timer, so its result shows
        // as soon as it arrives rather than on the next input.
        let pending =
            self.fetching.is_some() || self.thumbs.values().any(|t| matches!(t, Thumb::Loading));
        if pending {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(100));
        }
        match self.selected {
            Some(index) => {
                let page_area = ui.max_rect();
                // The page scrolls as a whole, from the window's right edge, with the page padded
                // inside the scroll area, as the album page does.
                let mut play = None;
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Frame::new()
                            .inner_margin(egui::Margin::symmetric(32, 32))
                            .show(ui, |ui| {
                                play = self.page(ui, index, tmdb_key, progress.clone());
                            });
                    });
                // The file information panel overlays the right of the page, above its content.
                if self.info_open {
                    let p = MoviePalette::of(ui);
                    egui::Area::new(ui.scope_id().with("movie_file_info"))
                        .order(egui::Order::Foreground)
                        .fixed_pos(egui::pos2(
                            page_area.right() - INFO_PANEL_WIDTH,
                            page_area.top(),
                        ))
                        .show(ui.ctx(), |ui| {
                            egui::Frame::new()
                                .fill(p.panel)
                                .inner_margin(egui::Margin::symmetric(24, 26))
                                .show(ui, |ui| {
                                    ui.set_width(INFO_PANEL_WIDTH - 48.0);
                                    ui.set_height(page_area.height() - 52.0);
                                    self.info_panel(ui, index);
                                });
                        });
                }
                play
            }
            None => {
                self.grid(ui);
                None
            }
        }
    }

    fn grid(&mut self, ui: &mut egui::Ui) {
        crate::fonts::page_title(ui, "Movies");
        if self.movies.is_empty() {
            ui.label("No movies yet. Choose your library folder in Settings.");
            return;
        }
        ui.label(format!("{} movies", self.movies.len()));
        ui.add_space(8.0);
        let mut opened = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let columns = ((ui.available_width() + GRID_GAP) / (TILE_WIDTH + GRID_GAP))
                    .floor()
                    .max(1.0) as usize;
                egui::Grid::new("movie_grid")
                    .spacing([GRID_GAP, 24.0])
                    .show(ui, |ui| {
                        for index in 0..self.movies.len() {
                            if self.tile(ui, index).clicked() {
                                opened = Some(index);
                            }
                            if (index + 1) % columns == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
        if let Some(index) = opened {
            self.selected = Some(index);
            self.info_open = false;
            self.notice = None;
            self.choices = None;
        }
    }

    /// One grid tile: the poster, then the title and a line of details. The poster is the click target.
    fn tile(&mut self, ui: &mut egui::Ui, index: usize) -> egui::Response {
        let movie = self.movies[index].clone();
        let title = self.shown_title(&movie);
        let poster_path = self.poster_path(movie.id);
        ui.vertical(|ui| {
            ui.set_width(TILE_WIDTH);
            let response = self.cover(
                ui,
                poster_path.as_deref(),
                TILE_WIDTH,
                TILE_POSTER_HEIGHT,
                egui::Sense::click(),
            );
            ui.add(egui::Label::new(crate::fonts::bold(title)).truncate());
            ui.add(egui::Label::new(egui::RichText::new(tile_details(&movie)).weak()).truncate());
            response
        })
        .inner
    }

    /// One movie's page, laid out as the design: the poster beside the title and actions, then the
    /// synopsis, cast, crew and details. Saved TMDb details fill in the text and photos.
    fn page(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        tmdb_key: Option<&str>,
        progress: Option<Progress>,
    ) -> Option<(PlayRequest, f64)> {
        let movie = self.movies[index].clone();
        let title = self.shown_title(&movie);
        let poster_path = self.poster_path(movie.id);
        let info = self.details.get(&movie.id).cloned();
        let accent = poster_path
            .as_ref()
            .and_then(|path| self.accents.get(path))
            .copied();
        let p = accent.map_or_else(|| MoviePalette::of(ui), MoviePalette::poster);
        if let Some(colour) = accent {
            // The background is drawn on the GPU, and asks for a redraw so the strands keep moving.
            let area = ui.clip_rect();
            let background = crate::flow_background::FlowBackground {
                base: colour,
                seed: (movie.id % 1000) as f32,
                time: ui.input(|input| input.time) as f32,
                aspect: area.width() / area.height().max(1.0),
            };
            ui.painter()
                .add(eframe::egui_wgpu::Callback::new_paint_callback(
                    area, background,
                ));
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(33));
        }
        let mut play = None;

        ui.horizontal(|ui| {
            if back_button(ui, &p).clicked() {
                self.selected = None;
            }
            ui.label(egui::RichText::new("Movies").size(13.0).color(p.weak));
        });
        ui.add_space(28.0);

        ui.horizontal_top(|ui| {
            let _ = self.cover(
                ui,
                poster_path.as_deref(),
                PAGE_POSTER_HEIGHT * 2.0 / 3.0,
                PAGE_POSTER_HEIGHT,
                egui::Sense::hover(),
            );
            ui.add_space(32.0);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("MOVIE")
                        .size(11.0)
                        .strong()
                        .color(p.weak),
                );
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(title.clone())
                            .size(40.0)
                            .family(crate::fonts::bold_family())
                            .color(p.text),
                    )
                    .wrap(),
                );
                if let Some(tagline) = info.as_ref().and_then(|i| i.tagline.clone()) {
                    ui.label(
                        egui::RichText::new(tagline)
                            .size(15.0)
                            .italics()
                            .color(p.body),
                    );
                }
                ui.label(
                    egui::RichText::new(meta_line(&movie, info.as_ref()))
                        .size(13.0)
                        .color(p.weak),
                );
                if movie.duration_ms.is_none() {
                    ui.label(
                        egui::RichText::new("Length unknown")
                            .size(12.0)
                            .color(p.weak),
                    );
                }
                ui.add_space(16.0);
                // A movie with progress offers to resume it, or to start over. A movie that
                // ended, or was only started, plays from the beginning.
                let resume = progress.filter(|saved| {
                    saved.position_secs >= 1.0
                        && !finished(saved.position_secs, saved.duration_secs)
                });
                ui.horizontal(|ui| {
                    let from_start = || PlayRequest {
                        tracks: vec![movie.clone()],
                        index: 0,
                    };
                    match &resume {
                        Some(saved) => {
                            let label =
                                format!("Resume at {}", crate::format::clock(saved.position_secs));
                            if pill_button(ui, &label, true, Some(icons::PLAY), &p).clicked() {
                                play = Some((from_start(), saved.position_secs));
                            }
                            if pill_button(
                                ui,
                                "Start over",
                                false,
                                Some(icons::CLOCK_COUNTER_CLOCKWISE),
                                &p,
                            )
                            .clicked()
                            {
                                play = Some((from_start(), 0.0));
                            }
                        }
                        None => {
                            if pill_button(ui, "Play", true, Some(icons::PLAY), &p).clicked() {
                                play = Some((from_start(), 0.0));
                            }
                        }
                    }
                    if info_button(ui, self.info_open, &p).clicked() {
                        self.info_open = !self.info_open;
                    }
                });
            });
        });
        ui.add_space(28.0);

        self.info_section(ui, &movie, tmdb_key, &p);
        if let Some(info) = &info {
            self.details_sections(ui, info, &p);
        }
        ui.add_space(28.0);
        ui.horizontal(|ui| {
            if info.is_some() {
                if let Some(key) = tmdb_key
                    && ui
                        .add(outline_button("Wrong movie? Choose another", &p))
                        .clicked()
                {
                    let job = find_job(movie.id, movie.path.clone(), key.to_owned(), true);
                    self.start(ui.ctx(), movie.id, job);
                }
                if ui.add(ghost_button("Remove details", &p)).clicked() {
                    self.details.remove(&movie.id);
                    let media_id = movie.id;
                    let job: Job = Box::new(move |library| {
                        library.clear_movie_details(media_id)?;
                        Ok(MatchOutcome::Nothing)
                    });
                    self.start(ui.ctx(), movie.id, job);
                }
            }
        });
        ui.add_space(36.0);
        play
    }

    /// The file information panel: the source file's name, then the facts about the file.
    fn info_panel(&mut self, ui: &mut egui::Ui, index: usize) {
        let movie = self.movies[index].clone();
        let p = MoviePalette::of(ui);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("File information")
                    .size(16.0)
                    .strong()
                    .color(p.text),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("×").size(16.0).color(p.text))
                            .fill(p.chip)
                            .corner_radius(999.0)
                            .min_size(egui::vec2(32.0, 32.0)),
                    )
                    .clicked()
                {
                    self.info_open = false;
                }
            });
        });
        ui.add_space(18.0);
        ui.label(
            egui::RichText::new("SOURCE")
                .size(11.0)
                .strong()
                .color(p.weak),
        );
        ui.add_space(6.0);
        let file_name = movie
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned();
        egui::Frame::new()
            .fill(p.chip)
            .corner_radius(10.0)
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(file_name)
                            .monospace()
                            .size(12.0)
                            .color(p.body),
                    )
                    .wrap(),
                );
            });
        ui.add_space(14.0);
        for (label, value) in file_info_rows(&movie) {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(label).size(14.0).color(p.weak));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(value).size(14.0).color(p.body));
                });
            });
            ui.separator();
        }
    }

    /// The lookup part of a movie's page: the candidates to pick from, a lookup to start, or what the
    /// last lookup said. Saved details are shown by the page itself.
    fn info_section(
        &mut self,
        ui: &mut egui::Ui,
        movie: &MediaItem,
        tmdb_key: Option<&str>,
        p: &MoviePalette,
    ) {
        let choosing = self.choices.as_ref().is_some_and(|(id, _)| *id == movie.id);
        if choosing {
            self.choice_list(ui, movie.id);
        } else if self.details.contains_key(&movie.id) {
            if let Some(notice) = &self.notice {
                ui.label(egui::RichText::new(notice).size(13.0).color(p.weak));
            }
        } else if self.fetching_id() == Some(movie.id) {
            ui.label(
                egui::RichText::new("Finding details…")
                    .size(13.0)
                    .color(p.weak),
            );
        } else if let Some(key) = tmdb_key {
            if ui.add(outline_button("Find details on TMDb", p)).clicked() {
                let job = find_job(movie.id, movie.path.clone(), key.to_owned(), false);
                self.start(ui.ctx(), movie.id, job);
            }
            if let Some(notice) = &self.notice {
                ui.label(egui::RichText::new(notice).size(13.0).color(p.weak));
            }
        } else {
            ui.label(
                egui::RichText::new(
                    "Add your TMDb API key in Settings to see the description and cast.",
                )
                .size(13.0)
                .color(p.weak),
            );
        }
        ui.add_space(16.0);
    }

    /// The synopsis, the cast, and the crew, from the saved TMDb details.
    fn details_sections(&mut self, ui: &mut egui::Ui, info: &MovieInfo, p: &MoviePalette) {
        if let Some(overview) = &info.overview {
            section_title(ui, "Synopsis", p);
            ui.add_space(4.0);
            ui.scope(|ui| {
                ui.set_max_width(760.0);
                ui.add(
                    egui::Label::new(egui::RichText::new(overview).size(15.0).color(p.body)).wrap(),
                );
            });
            ui.add_space(24.0);
        }
        if !info.cast.is_empty() {
            section_title(ui, "Cast", p);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(TILE_GAP, TILE_GAP);
                for member in &info.cast {
                    self.person(
                        ui,
                        member.profile_path.as_deref(),
                        &member.name,
                        &member.character,
                        p,
                    );
                }
            });
            ui.add_space(24.0);
        }
        let mut crew: Vec<(&str, &str, Option<&str>)> = Vec::new();
        for name in &info.directors {
            crew.push((name, "Director", info.photos.get(name).map(String::as_str)));
        }
        for name in &info.writers {
            if !info.directors.contains(name) {
                crew.push((name, "Writer", info.photos.get(name).map(String::as_str)));
            }
        }
        if !crew.is_empty() {
            section_title(ui, "Crew", p);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(TILE_GAP, TILE_GAP);
                for (name, role, photo) in crew {
                    self.person(ui, photo, name, role, p);
                }
            });
            ui.add_space(24.0);
        }
    }

    /// A person, as an artist tile: a circle with their photo, then their name, then their role in
    /// subtle text.
    fn person(
        &mut self,
        ui: &mut egui::Ui,
        photo: Option<&str>,
        name: &str,
        role: &str,
        p: &MoviePalette,
    ) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(TILE_COVER, TILE_COVER + TILE_TEXT),
            egui::Sense::hover(),
        );
        let circle = egui::Rect::from_min_size(rect.min, egui::vec2(TILE_COVER, TILE_COVER));
        let painter = ui.painter_at(rect);
        painter.circle_filled(circle.center(), TILE_COVER / 2.0, p.placeholder);
        if let Some(path) = photo
            && let Some(texture) = self.texture_for(ui.ctx(), path)
        {
            let uv = centre_square(texture.size_vec2());
            egui::Image::new(&texture)
                .uv(uv)
                .corner_radius(TILE_COVER / 2.0)
                .paint_at(ui, circle);
        }
        let text_top = circle.bottom() + 6.0;
        tile_text(
            ui,
            &painter,
            rect,
            text_top,
            name,
            egui::FontId::proportional(14.0),
            p.text,
            response.id.with("name"),
            response.hovered(),
        );
        tile_text(
            ui,
            &painter,
            rect,
            text_top + 20.0,
            role,
            egui::FontId::proportional(12.0),
            p.weak,
            response.id.with("role"),
            response.hovered(),
        );
    }

    /// The candidates for a weak match: a card per candidate, and a way to pick none of them.
    fn choice_list(&mut self, ui: &mut egui::Ui, media_id: i64) {
        ui.strong("Which movie is this?");
        ui.weak("Pick the one your file is. Nothing is saved until you pick.");
        ui.add_space(8.0);
        let candidates = self
            .choices
            .as_ref()
            .map(|(_, list)| list.clone())
            .unwrap_or_default();
        let mut picked = None;
        for info in &candidates {
            if self.choice_card(ui, info) {
                picked = Some(info.clone());
                break;
            }
            ui.add_space(10.0);
        }
        if let Some(info) = picked {
            self.choices = None;
            let job: Job = Box::new(move |library| {
                library.save_movie_details(media_id, &info)?;
                Ok(MatchOutcome::Matched(Box::new(info)))
            });
            self.start(ui.ctx(), media_id, job);
        }
        ui.add_space(6.0);
        if ui.button("None of these").clicked() {
            self.choices = None;
            self.notice = Some("Left unmatched.".to_owned());
        }
    }

    /// One candidate: its cover, title, year, genres and synopsis. Returns true when it is picked.
    fn choice_card(&mut self, ui: &mut egui::Ui, info: &MovieInfo) -> bool {
        let mut picked = false;
        ui.horizontal_top(|ui| {
            let _ = self.cover(
                ui,
                info.poster_path.as_deref(),
                CHOICE_WIDTH,
                CHOICE_HEIGHT,
                egui::Sense::hover(),
            );
            ui.add_space(14.0);
            ui.vertical(|ui| {
                let year = info.year.map(|y| format!(" ({y})")).unwrap_or_default();
                ui.strong(format!("{}{year}", info.title));
                if !info.genres.is_empty() {
                    ui.weak(info.genres.join(", "));
                }
                if let Some(overview) = &info.overview {
                    ui.add(egui::Label::new(shorten(overview, SYNOPSIS_CHARS)).wrap());
                }
                ui.add_space(4.0);
                if ui.button("This one").clicked() {
                    picked = true;
                }
            });
        });
        picked
    }

    /// The TMDb title of a movie once details are saved, otherwise its file name.
    fn shown_title(&self, movie: &MediaItem) -> String {
        self.details
            .get(&movie.id)
            .map(|d| d.title.clone())
            .unwrap_or_else(|| format::title(movie))
    }

    /// The TMDb poster path of a movie, once details are saved.
    fn poster_path(&self, media_id: i64) -> Option<String> {
        self.details
            .get(&media_id)
            .and_then(|d| d.poster_path.clone())
    }

    /// A cover of the given size: the TMDb poster for `path`, or an empty box until it has
    /// downloaded, or when there is no poster. The download starts the first time it is drawn.
    fn cover(
        &mut self,
        ui: &mut egui::Ui,
        path: Option<&str>,
        width: f32,
        height: f32,
        sense: egui::Sense,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), sense);
        ui.painter()
            .rect_filled(rect, POSTER_RADIUS, ui.visuals().faint_bg_color);
        if let Some(path) = path
            && let Some(texture) = self.texture_for(ui.ctx(), path)
        {
            egui::Image::new(&texture)
                .corner_radius(POSTER_RADIUS)
                .paint_at(ui, rect);
        }
        response
    }

    /// The texture of a TMDb image, once it has downloaded. The download starts on the first call.
    fn texture_for(&mut self, ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
        if let Some(Thumb::Ready(texture)) = self.thumbs.get(path) {
            return Some(texture.clone());
        }
        if !self.thumbs.contains_key(path) {
            self.thumbs.insert(path.to_owned(), Thumb::Loading);
            self.fetch_thumb(ctx, path.to_owned());
        }
        None
    }

    /// Download and decode a cover on a background thread.
    fn fetch_thumb(&self, ctx: &egui::Context, path: String) {
        let tx = self.thumb_tx.clone();
        let ctx = ctx.clone();
        thread::spawn(move || {
            let image = Tmdb::new(String::new())
                .poster_bytes(&path)
                .ok()
                .and_then(|bytes| image::load_from_memory(&bytes).ok())
                .map(|picture| {
                    let rgba = picture.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw())
                });
            let _ = tx.send((path, image));
            ctx.request_repaint();
        });
    }

    /// Take in the covers that have finished downloading, and upload them as textures.
    fn poll_thumbs(&mut self, ctx: &egui::Context) {
        let finished: Vec<_> = self.thumb_rx.try_iter().collect();
        for (path, image) in finished {
            let thumb = match image {
                Some(image) => {
                    if let Some(colour) = dominant_colour(&image) {
                        self.accents.insert(path.clone(), colour);
                    }
                    Thumb::Ready(ctx.load_texture(
                        format!("tmdb-poster-{path}"),
                        image,
                        egui::TextureOptions::LINEAR,
                    ))
                }
                None => Thumb::Missing,
            };
            self.thumbs.insert(path, thumb);
        }
    }

    fn fetching_id(&self) -> Option<i64> {
        self.fetching.as_ref().map(|(id, _)| *id)
    }

    /// Run `job` on a background thread, with its own database connection, for movie `media_id`.
    fn start(&mut self, ctx: &egui::Context, media_id: i64, job: Job) {
        let db = self.db.clone();
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        thread::spawn(move || {
            let result: Lookup = Library::open(&db)
                .and_then(|library| job(&library))
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        self.fetching = Some((media_id, rx));
        self.notice = None;
    }

    /// Take in the details that a background refetch has read again.
    fn poll_refresh(&mut self) {
        let Some(rx) = &self.refreshing else {
            return;
        };
        match rx.try_recv() {
            Ok(infos) => {
                for (media_id, info) in infos {
                    self.details.insert(media_id, info);
                }
                self.refreshing = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => self.refreshing = None,
        }
    }

    /// Fetch again, by TMDb id, the saved details that lack photos. Each movie is tried once per
    /// session, so a person TMDb has no photo for is not asked about again and again.
    fn start_refresh(&mut self, ctx: &egui::Context, tmdb_key: Option<&str>) {
        let Some(key) = tmdb_key else {
            return;
        };
        if self.refreshing.is_some() {
            return;
        }
        let stale: Vec<(i64, u64)> = self
            .details
            .iter()
            .filter(|(id, info)| info.is_missing_photos() && !self.refresh_tried.contains(*id))
            .map(|(id, info)| (*id, info.id))
            .collect();
        if stale.is_empty() {
            return;
        }
        self.refresh_tried.extend(stale.iter().map(|(id, _)| *id));
        let db = self.db.clone();
        let key = key.to_owned();
        let (tx, rx) = mpsc::channel();
        let thread_ctx = ctx.clone();
        thread::spawn(move || {
            let client = Tmdb::new(key);
            let found: Vec<(i64, MovieInfo)> = Library::open(&db)
                .map(|library| {
                    stale
                        .into_iter()
                        .filter_map(|(media_id, tmdb_id)| {
                            library
                                .refresh_movie_details(media_id, tmdb_id, &client)
                                .ok()
                                .map(|info| (media_id, info))
                        })
                        .collect()
                })
                .unwrap_or_default();
            let _ = tx.send(found);
            thread_ctx.request_repaint();
        });
        self.refreshing = Some(rx);
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }

    /// Take in the result of a job that has finished.
    fn poll_fetch(&mut self) {
        let Some((media_id, rx)) = &self.fetching else {
            return;
        };
        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("the lookup stopped unexpectedly".to_owned()),
        };
        let media_id = *media_id;
        self.fetching = None;
        match result {
            Ok(MatchOutcome::Matched(info)) => {
                self.details.insert(media_id, *info);
                self.choices = None;
                self.notice = None;
            }
            Ok(MatchOutcome::Choices(list)) => {
                self.choices = Some((media_id, list));
                self.notice = None;
            }
            Ok(MatchOutcome::Nothing) => {
                self.notice = Some("No TMDb details for this movie.".to_owned());
            }
            Err(e) => {
                self.notice = Some(format!("Could not fetch details: {e}"));
            }
        }
    }
}

/// A job that looks the movie up on TMDb. A confident match is saved; `force_choices` always
/// returns the candidates instead.
fn find_job(media_id: i64, path: PathBuf, key: String, force_choices: bool) -> Job {
    Box::new(move |library| library.find_movie(media_id, &path, &Tmdb::new(key), force_choices))
}

/// The first `max` characters of `text`, with an ellipsis when some were cut.
fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let cut: String = text.chars().take(max).collect();
    format!("{}…", cut.trim_end())
}

/// Colours of the movie page, from the design: dark values in dark mode, and light ones otherwise.
struct MoviePalette {
    text: egui::Color32,
    body: egui::Color32,
    weak: egui::Color32,
    fill: egui::Color32,
    on_fill: egui::Color32,
    chip: egui::Color32,
    placeholder: egui::Color32,
    outline: egui::Color32,
    panel: egui::Color32,
}

impl MoviePalette {
    fn of(ui: &egui::Ui) -> Self {
        let rgb = |v: u32| egui::Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8);
        if ui.visuals().dark_mode {
            Self {
                text: rgb(0xf2f2f4),
                body: rgb(0xd4d4d8),
                weak: rgb(0x9a9aa2),
                fill: rgb(0xf2f2f4),
                on_fill: rgb(0x141416),
                chip: rgb(0x2a2a2f),
                placeholder: rgb(0x26262b),
                outline: rgb(0x3a3a41),
                panel: rgb(0x1b1b1e),
            }
        } else {
            Self {
                text: rgb(0x1c1c1e),
                body: rgb(0x3a3a3f),
                weak: rgb(0x6e6e76),
                fill: rgb(0x1c1c1e),
                on_fill: rgb(0xffffff),
                chip: rgb(0xe6e6eb),
                placeholder: rgb(0xe4e4e9),
                outline: rgb(0xc8c8d0),
                panel: rgb(0xececf0),
            }
        }
    }
}

impl MoviePalette {
    /// The palette for a movie with cover art. The page is the cover's dominant colour, and what is
    /// drawn on it is the complement of that colour. Text meets the WCAG contrast ratio on the page,
    /// and the primary buttons use the dominant colour as their text.
    fn poster(dominant: egui::Color32) -> Self {
        let hue = Lab::from_srgb(dominant).complement();
        let text = colour::readable_on(dominant, hue, 4.5);
        let button = colour::readable_on(dominant, hue, 4.5);
        let weak = colour::readable_on(dominant, hue, 3.0);
        Self {
            text,
            body: text,
            weak,
            fill: button,
            on_fill: dominant,
            chip: mix(dominant, text, 0.12),
            placeholder: mix(dominant, text, 0.08),
            outline: text,
            panel: dominant,
        }
    }
}

/// The round back button at the top of the page.
fn back_button(ui: &mut egui::Ui, p: &MoviePalette) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
    let painter = ui.painter();
    painter.circle_filled(rect.center(), 18.0, p.chip);
    icons::paint(painter, rect.center(), icons::CARET_LEFT, 18.0, p.text);
    response
}

/// A rounded button for the page's actions. A filled one is the primary action, and an outlined one
/// is secondary. Its background fades to the hover colour over 200 ms on an ease-in curve.
fn pill_button(
    ui: &mut egui::Ui,
    text: &str,
    filled: bool,
    icon: Option<&str>,
    p: &MoviePalette,
) -> egui::Response {
    const SIDE_PADDING: f32 = 22.0;
    let (font_size, min_height): (f32, f32) = if filled { (14.0, 40.0) } else { (13.0, 36.0) };
    let text_color = if filled { p.on_fill } else { p.body };
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::proportional(font_size),
        text_color,
    );
    // Play keeps the width it had before it became a pill.
    let min_width = if filled { 96.0 } else { 0.0 };
    // The icon, when there is one, sits before the label with a gap between them.
    let icon_size = if filled { 18.0 } else { 17.0 };
    let icon_width = if icon.is_some() { icon_size + 8.0 } else { 0.0 };
    let content_width = icon_width + galley.size().x;
    let size = egui::vec2(
        (content_width + 2.0 * SIDE_PADDING).max(min_width),
        min_height.max(galley.size().y),
    );
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let progress =
        ui.ctx()
            .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.2);
    let eased = progress * progress;
    let fill = if filled {
        mix(p.fill, mix(p.fill, p.on_fill, 0.15), eased)
    } else {
        // The outline's tint keeps its colour and only fades in its alpha. Mixing from transparent
        // would pass through black, because egui stores transparent colours premultiplied.
        let tint = mix(p.panel, p.text, 0.08);
        let alpha = (eased * 255.0).round() as u8;
        egui::Color32::from_rgba_unmultiplied(tint.r(), tint.g(), tint.b(), alpha)
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 999.0, fill);
    if !filled {
        painter.rect_stroke(
            rect,
            999.0,
            egui::Stroke::new(1.0, p.outline),
            egui::StrokeKind::Inside,
        );
    }
    let left = rect.center().x - content_width / 2.0;
    if let Some(glyph) = icon {
        icons::paint_filled(
            painter,
            egui::pos2(left + icon_size / 2.0, rect.center().y),
            glyph,
            icon_size,
            text_color,
        );
    }
    painter.galley(
        egui::pos2(left + icon_width, rect.center().y - galley.size().y / 2.0),
        galley,
        text_color,
    );
    response
}

/// The colour that covers most of a poster, found in OKLab. Near-black, near-white and grey pixels
/// are left out, so the art's colour shows rather than its background. `None` when no colour stands
/// out.
fn dominant_colour(image: &egui::ColorImage) -> Option<egui::Color32> {
    const LIGHTNESS_STEPS: usize = 6;
    const AXIS_STEPS: usize = 8;
    // Per bucket: how many pixels, and the sums of their lightness, a and b.
    let mut buckets = vec![(0u32, 0f32, 0f32, 0f32); LIGHTNESS_STEPS * AXIS_STEPS * AXIS_STEPS];
    let step = (image.pixels.len() / 4000).max(1);
    let step_of = |value: f32, steps: usize, low: f32, high: f32| {
        let position = ((value - low) / (high - low)) * steps as f32;
        position.floor().clamp(0.0, (steps - 1) as f32) as usize
    };
    for pixel in image.pixels.iter().step_by(step) {
        let lab = Lab::from_srgb(*pixel);
        if !(0.15..=0.93).contains(&lab.l) || lab.a.hypot(lab.b) < 0.04 {
            continue;
        }
        let index = (step_of(lab.l, LIGHTNESS_STEPS, 0.15, 0.93) * AXIS_STEPS
            + step_of(lab.a, AXIS_STEPS, -0.3, 0.3))
            * AXIS_STEPS
            + step_of(lab.b, AXIS_STEPS, -0.3, 0.3);
        let bucket = &mut buckets[index];
        bucket.0 += 1;
        bucket.1 += lab.l;
        bucket.2 += lab.a;
        bucket.3 += lab.b;
    }
    let (count, l, a, b) = buckets.into_iter().max_by_key(|bucket| bucket.0)?;
    if count < 20 {
        return None;
    }
    let n = count as f32;
    Some(
        Lab {
            l: l / n,
            a: a / n,
            b: b / n,
        }
        .to_srgb(),
    )
}

/// The colour `t` of the way from `a` to `b`, channel by channel, alpha included.
fn mix(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let channel = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    egui::Color32::from_rgba_unmultiplied(
        channel(a.r(), b.r()),
        channel(a.g(), b.g()),
        channel(a.b(), b.b()),
        channel(a.a(), b.a()),
    )
}

fn outline_button(text: &str, p: &MoviePalette) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text.to_owned())
            .size(13.0)
            .color(p.body),
    )
    .fill(egui::Color32::TRANSPARENT)
    .stroke(egui::Stroke::new(1.0, p.outline))
    .corner_radius(999.0)
    .min_size(egui::vec2(0.0, 36.0))
}

fn ghost_button(text: &str, p: &MoviePalette) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text.to_owned())
            .size(13.0)
            .color(p.weak),
    )
    .fill(egui::Color32::TRANSPARENT)
    .stroke(egui::Stroke::NONE)
    .corner_radius(999.0)
}

fn section_title(ui: &mut egui::Ui, text: &str, p: &MoviePalette) {
    ui.label(
        egui::RichText::new(text.to_owned())
            .size(16.0)
            .family(crate::fonts::bold_family())
            .color(p.text),
    );
    ui.add_space(4.0);
}

/// The year, genres and running time, from the saved details when there are some.
fn meta_line(movie: &MediaItem, info: Option<&MovieInfo>) -> String {
    let year = info.and_then(|i| i.year).or(movie.year);
    let genres = info.map(|i| i.genres.join(", ")).filter(|g| !g.is_empty());
    let minutes = info
        .and_then(|i| i.runtime_minutes)
        .or_else(|| movie.duration_ms.map(|ms| ms / 60_000));
    join(&[
        year.map(|y| y.to_string()),
        genres,
        minutes.map(|m| format!("{m} min")),
    ])
}

/// The largest centred square of a picture, as texture coordinates.
fn centre_square(size: egui::Vec2) -> egui::Rect {
    if size.y > size.x {
        let inset = (1.0 - size.x / size.y) / 2.0;
        egui::Rect::from_min_max(egui::pos2(0.0, inset), egui::pos2(1.0, 1.0 - inset))
    } else {
        let inset = (1.0 - size.y / size.x) / 2.0;
        egui::Rect::from_min_max(egui::pos2(inset, 0.0), egui::pos2(1.0 - inset, 1.0))
    }
}

/// A line under a person's photo, centred. A line wider than the tile scrolls while the tile is
/// hovered, as the albums page does.
#[allow(clippy::too_many_arguments)]
fn tile_text(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: egui::Rect,
    top: f32,
    text: &str,
    font: egui::FontId,
    colour: egui::Color32,
    id: egui::Id,
    hovered: bool,
) {
    let width = painter
        .layout_no_wrap(text.to_owned(), font.clone(), colour)
        .size()
        .x;
    if width <= rect.width() {
        painter.text(
            egui::pos2(rect.center().x, top),
            egui::Align2::CENTER_TOP,
            text,
            font,
            colour,
        );
    } else {
        let offset = hover_scroll(ui, id, hovered);
        tile_line(painter, rect, top, text.to_owned(), (font, colour), offset);
    }
}

/// The facts about a file that the information panel lists. A fact the file does not give reads
/// "Unknown", and the frame rate is not kept in the index at all.
/// A frame rate as people write it: `23.976`, `25`, `29.97`. A variable one says so.
fn frame_rate_label(fps: f64, variable: bool) -> String {
    let text = format!("{fps:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if variable {
        format!("{text} fps average, variable")
    } else {
        format!("{text} fps")
    }
}

fn file_info_rows(movie: &MediaItem) -> Vec<(&'static str, String)> {
    let known = |value: Option<String>| value.unwrap_or_else(|| "Unknown".into());
    // The stored ratio when the file states one, else the picture's own shape.
    let aspect = movie.video_aspect.or(match (movie.width, movie.height) {
        (Some(w), Some(h)) if h > 0 => Some(w as f64 / h as f64),
        _ => None,
    });
    let aspect = aspect.map(|ratio| format!("{ratio:.2}:1"));
    vec![
        ("Container", container_label(movie)),
        ("Codec", known(movie.video_codec.clone())),
        (
            "Colour depth",
            known(movie.video_bit_depth.map(|bits| format!("{bits}-bit"))),
        ),
        ("Chroma subsampling", known(movie.video_chroma.clone())),
        ("Resolution", known(resolution_label(movie))),
        ("Aspect ratio", known(aspect)),
        (
            "Frame rate",
            known(
                movie
                    .video_fps
                    .map(|fps| frame_rate_label(fps, movie.video_fps_variable)),
            ),
        ),
        (
            "Length",
            known(
                movie
                    .duration_ms
                    .map(|ms| format::clock(ms as f64 / 1000.0)),
            ),
        ),
        ("File size", format::size(movie.size)),
    ]
}

/// The round info button beside Play. It is filled while the file information panel is open.
fn info_button(ui: &mut egui::Ui, active: bool, p: &MoviePalette) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::click());
    let painter = ui.painter();
    let centre = rect.center();
    if active {
        painter.circle_filled(centre, 20.0, p.chip);
    }
    painter.circle_stroke(centre, 19.5, egui::Stroke::new(1.0, p.outline));
    let stroke = egui::Stroke::new(1.5, p.text);
    painter.circle_stroke(centre, 6.5, stroke);
    painter.circle_filled(centre + egui::vec2(0.0, -3.2), 0.9, p.text);
    painter.line_segment(
        [
            centre + egui::vec2(0.0, -0.8),
            centre + egui::vec2(0.0, 3.5),
        ],
        stroke,
    );
    response
}

/// The codec, the picture size and the file size, for a grid tile.
fn tile_details(movie: &MediaItem) -> String {
    join(&[
        movie.video_codec.clone(),
        quality_label(movie.height),
        Some(format::size(movie.size)),
    ])
}

/// The file's container, by its extension. A raw OBU stream is named as one, since it holds no
/// container of its own.
fn container_label(movie: &MediaItem) -> String {
    let ext = movie
        .path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "mp4" => "MP4".into(),
        "m4v" => "M4V".into(),
        "mkv" => "Matroska (MKV)".into(),
        "webm" => "WebM".into(),
        "mov" => "QuickTime (MOV)".into(),
        "avi" => "AVI".into(),
        "ts" | "m2ts" | "mts" => "MPEG transport stream".into(),
        "mpg" | "mpeg" => "MPEG program stream".into(),
        "obu" => "Raw AV1 stream (OBU)".into(),
        other => other.to_ascii_uppercase(),
    }
}

/// The picture size as a word: "4K", "1080p" and so on.
fn quality_label(height: Option<i64>) -> Option<String> {
    let height = height?;
    Some(match height {
        h if h >= 2160 => "4K".into(),
        h if h >= 1440 => "1440p".into(),
        h if h >= 1080 => "1080p".into(),
        h if h >= 720 => "720p".into(),
        h => format!("{h}p"),
    })
}

fn resolution_label(movie: &MediaItem) -> Option<String> {
    Some(format!("{} × {}", movie.width?, movie.height?))
}

/// Join the parts that are known with " · ".
fn join(parts: &[Option<String>]) -> String {
    parts
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
        .join(" · ")
}
