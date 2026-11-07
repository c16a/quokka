//! A window that plays one video file's frames from the decoders, to check the frame path in a
//! real window. Run it with `cargo run -p player-ui --example frame_view -- <file>`. Space pauses.

use std::path::PathBuf;
use std::time::Instant;

use eframe::egui;
use player_playback::decode::frames::Rgba;
use player_playback::decode::player::{FramePlayer, VideoFrame};

struct FrameView {
    player: Option<FramePlayer>,
    error: Option<String>,
    /// The frame whose time has not come up yet.
    next: Option<VideoFrame>,
    texture: Option<egui::TextureHandle>,
    /// Where playback is, in seconds, and when the clock last moved.
    clock: f64,
    last_tick: Instant,
    paused: bool,
    shown: usize,
    size: (u32, u32),
}

impl FrameView {
    fn new(path: Option<PathBuf>) -> Self {
        let (player, error) = match path {
            None => (None, Some("usage: frame_view <video file>".into())),
            Some(path) => match FramePlayer::open(&path) {
                Ok(player) => (Some(player), None),
                Err(error) => (None, Some(error.to_string())),
            },
        };
        FrameView {
            player,
            error,
            next: None,
            texture: None,
            clock: 0.0,
            last_tick: Instant::now(),
            paused: false,
            shown: 0,
            size: (0, 0),
        }
    }

    /// Show the frames whose times have come, keeping the next one for later.
    fn advance(&mut self, ctx: &egui::Context) {
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
                Some(frame) if frame.time <= self.clock => {
                    self.show(ctx, &frame.image);
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
}

impl eframe::App for FrameView {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|input| input.key_pressed(egui::Key::Space)) {
            self.paused = !self.paused;
        }
        let now = Instant::now();
        if !self.paused {
            self.clock += now.duration_since(self.last_tick).as_secs_f64();
        }
        self.last_tick = now;
        self.advance(&ctx);

        let finished =
            self.player.as_ref().is_some_and(|player| player.finished()) && self.next.is_none();
        egui::Panel::bottom("status").show(ui, |ui| {
            let state = if self.error.is_some() {
                "error".to_string()
            } else if finished {
                "end of file".to_string()
            } else if self.paused {
                "paused".to_string()
            } else {
                "playing".to_string()
            };
            let detail = self.error.clone().unwrap_or_else(|| {
                format!(
                    "{state} at {:.2} s, {} frames shown, {}x{}",
                    self.clock, self.shown, self.size.0, self.size.1
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

        if !self.paused && !finished {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
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
