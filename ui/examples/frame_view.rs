//! A window that plays one video file, with its soundtrack, from the decoders, to check the playback
//! path in a real window. Run it with `cargo run -p quokka-ui --example frame_view -- <file>`. Space
//! pauses. The sound sets the clock. When the sound ends, the video carries on from the same point on
//! the wall clock. A file without sound runs on the wall clock throughout.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui;
use quokka_playback::decode::frames::Rgba;
use quokka_playback::decode::player::{FramePlayer, VideoFrame};
use quokka_playback::movie_audio::MovieAudio;

struct FrameView {
    player: Option<FramePlayer>,
    audio: Option<MovieAudio>,
    /// Whether the sound has ended, and the wall clock has taken over.
    audio_done: bool,
    error: Option<String>,
    /// The frame whose time has not come up yet.
    next: Option<VideoFrame>,
    texture: Option<egui::TextureHandle>,
    /// Playback position in seconds, when the wall clock is in charge.
    wall_clock: f64,
    last_tick: Instant,
    paused: bool,
    shown: usize,
    size: (u32, u32),
}

impl FrameView {
    fn new(path: Option<PathBuf>) -> Self {
        let mut error = None;
        let (player, audio) = match path {
            None => {
                error = Some("usage: frame_view <video file>".into());
                (None, None)
            }
            Some(path) => {
                let player = match FramePlayer::open(&path) {
                    Ok(player) => Some(player),
                    Err(e) => {
                        error = Some(e.to_string());
                        None
                    }
                };
                let audio = match MovieAudio::open(&path) {
                    Ok(audio) => audio,
                    Err(e) => {
                        error = Some(format!("audio: {e}"));
                        None
                    }
                };
                (player, audio)
            }
        };
        if let Some(audio) = &audio {
            audio.play();
        }
        FrameView {
            player,
            audio,
            audio_done: false,
            error,
            next: None,
            texture: None,
            wall_clock: 0.0,
            last_tick: Instant::now(),
            paused: false,
            shown: 0,
            size: (0, 0),
        }
    }

    /// Where playback is: the sound's clock while the sound plays, else the wall clock.
    fn clock(&self) -> f64 {
        match &self.audio {
            Some(audio) if !self.audio_done => audio.clock(),
            _ => self.wall_clock,
        }
    }

    /// Show the frames whose times have come, keeping the next one for later.
    fn advance(&mut self, ctx: &egui::Context, clock: f64) {
        loop {
            if self.next.is_none() {
                let Some(player) = self.player.as_mut() else {
                    return;
                };
                match player.next_frame() {
                    Ok(Some(frame)) => self.next = Some(frame),
                    Ok(None) => return,
                    Err(error) => {
                        self.error = Some(error.to_string());
                        return;
                    }
                }
            }
            match self.next.take() {
                Some(frame) if frame.time <= clock => {
                    self.show(ctx, &frame.image.to_rgba());
                    self.shown += 1;
                }
                other => {
                    self.next = other;
                    return;
                }
            }
        }
    }

    fn show(&mut self, ctx: &egui::Context, image: &Rgba) {
        self.size = (image.width, image.height);
        let color = egui::ColorImage::from_rgba_unmultiplied(
            [image.width as usize, image.height as usize],
            &image.pixels,
        );
        match &mut self.texture {
            Some(texture) => texture.set(color, egui::TextureOptions::LINEAR),
            None => {
                self.texture = Some(ctx.load_texture("frame", color, egui::TextureOptions::LINEAR));
            }
        }
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if let Some(audio) = &self.audio {
            if self.paused {
                audio.pause();
            } else {
                audio.play();
            }
        }
    }
}

impl eframe::App for FrameView {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|input| input.key_pressed(egui::Key::Space)) {
            self.toggle_pause();
        }
        // When the sound runs out, the wall clock picks up from where the sound stopped.
        if let Some(audio) = &self.audio
            && !self.audio_done
            && audio.finished()
        {
            self.audio_done = true;
            self.wall_clock = audio.clock();
        }
        let now = Instant::now();
        if !self.paused && (self.audio.is_none() || self.audio_done) {
            self.wall_clock += now.duration_since(self.last_tick).as_secs_f64();
        }
        self.last_tick = now;
        let clock = self.clock();
        self.advance(&ctx, clock);

        let finished =
            self.player.as_ref().is_some_and(|player| player.finished()) && self.next.is_none();
        egui::Panel::bottom("status").show(ui, |ui| {
            let state = if finished {
                "end of file"
            } else if self.paused {
                "paused"
            } else {
                "playing"
            };
            let source = if self.audio.is_some() && !self.audio_done {
                "sound clock"
            } else {
                "wall clock"
            };
            let detail = self.error.clone().unwrap_or_else(|| {
                format!(
                    "{state} at {clock:.2} s ({source}), {} frames shown, {}x{}",
                    self.shown, self.size.0, self.size.1
                )
            });
            ui.label(detail);
        });

        egui::CentralPanel::default().show(ui, |ui| match &self.texture {
            Some(texture) => {
                ui.centered_and_justified(|ui| {
                    ui.add(egui::Image::new(texture).shrink_to_fit());
                });
            }
            None => {
                ui.label("waiting for the first frame");
            }
        });

        // The clock only moves on a repaint, so keep repainting until the sound has ended too.
        let sound_playing = self.audio.is_some() && !self.audio_done;
        if !finished || sound_playing {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}

fn main() -> eframe::Result {
    let path = std::env::args().nth(1).map(PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Frame view")
            .with_inner_size([960.0, 600.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Frame view",
        options,
        Box::new(move |_cc| Ok(Box::new(FrameView::new(path)) as Box<dyn eframe::App>)),
    )
}
