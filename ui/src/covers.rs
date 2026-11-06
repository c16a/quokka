//! Cover images, decoded off the UI thread. A job reads and decodes an image on a worker thread,
//! and the finished pixels come back to the UI thread, which uploads them as textures. Uploading is
//! quick, so scrolling and opening a playlist don't wait on image decoding.

use std::hash::Hash;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use eframe::egui;

/// Largest side of a decoded album or artist cover. Bigger pictures are scaled down to save memory.
pub(crate) const COVER_MAX_SIDE: u32 = 256;
/// Largest side of a track's thumbnail in a list.
pub(crate) const THUMB_MAX_SIDE: u32 = 72;
/// Share of grey in a cover that is not hovered: CSS `filter: grayscale(50%)`.
pub(crate) const COVER_GREY: f32 = 0.5;

/// A cover as textures: the colour one, and the partly grey one shown while not hovered.
#[derive(Clone)]
pub(crate) struct Cover {
    pub(crate) colour: egui::TextureHandle,
    pub(crate) grayscaled: egui::TextureHandle,
}

/// A cover's pixels, decoded and ready to upload.
pub(crate) struct Decoded {
    colour: egui::ColorImage,
    grayscaled: egui::ColorImage,
}

/// Decode a picture to a square cover no larger than `max_side`: the central square is kept.
pub(crate) fn decode(bytes: &[u8], max_side: u32) -> Option<Decoded> {
    let picture = image::load_from_memory(bytes).ok()?;
    let side = picture.width().min(picture.height());
    let (x, y) = ((picture.width() - side) / 2, (picture.height() - side) / 2);
    let image = picture
        .crop_imm(x, y, side, side)
        .thumbnail(max_side, max_side)
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let colour = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    let grayscaled = egui::ColorImage::from_rgba_unmultiplied(size, &grayscale(&image, COVER_GREY));
    Some(Decoded { colour, grayscaled })
}

/// Upload decoded pixels as a cover. `key` names the textures.
pub(crate) fn upload(ctx: &egui::Context, key: &str, decoded: Decoded) -> Cover {
    Cover {
        colour: ctx.load_texture(key, decoded.colour, egui::TextureOptions::LINEAR),
        grayscaled: ctx.load_texture(
            format!("{key}-grayscaled"),
            decoded.grayscaled,
            egui::TextureOptions::LINEAR,
        ),
    }
}

/// The pixels of `image` with `amount` of grey, as CSS `grayscale(amount)` mixes each pixel with
/// its luma. Alpha is kept.
fn grayscale(image: &image::RgbaImage, amount: f32) -> Vec<u8> {
    image
        .pixels()
        .flat_map(|p| {
            let [r, g, b, a] = p.0;
            let (r, g, b) = (r as f32, g as f32, b as f32);
            // Luma weights from the CSS filter specification.
            let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let mix = |c: f32| (c * (1.0 - amount) + luma * amount).round() as u8;
            [mix(r), mix(g), mix(b), a]
        })
        .collect()
}

/// A job for the worker: runs off the UI thread and returns the decoded pixels, if any.
pub(crate) type Job = Box<dyn FnOnce() -> Option<Decoded> + Send>;

/// A background thread that runs decoding jobs, one at a time, for keys of type `K`.
pub(crate) struct Worker<K> {
    jobs: Sender<(K, Job)>,
    results: Receiver<(K, Option<Decoded>)>,
}

impl<K: Send + 'static> Worker<K> {
    /// Start the thread. It wakes the UI when a job finishes.
    pub(crate) fn new(ctx: &egui::Context) -> Self {
        let (job_tx, job_rx) = mpsc::channel::<(K, Job)>();
        let (result_tx, result_rx) = mpsc::channel();
        let ctx = ctx.clone();
        thread::spawn(move || {
            // Ends when the owner is dropped, which closes the job channel.
            for (key, job) in job_rx {
                let decoded = job();
                if result_tx.send((key, decoded)).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
        });
        Self {
            jobs: job_tx,
            results: result_rx,
        }
    }

    /// Queue a job.
    pub(crate) fn submit(&self, key: K, job: Job) {
        let _ = self.jobs.send((key, job));
    }

    /// The jobs that have finished since the last call.
    pub(crate) fn finished(&self) -> Vec<(K, Option<Decoded>)> {
        self.results.try_iter().collect()
    }
}

/// Keep at most `cap` entries in a cache of textures: when it is full, the entry used longest ago
/// is dropped. Entries still loading are kept.
pub(crate) fn evict<K: Eq + Hash + Clone, V>(
    entries: &mut std::collections::HashMap<K, (u64, V)>,
    cap: usize,
    is_loaded: impl Fn(&V) -> bool,
) {
    while entries.len() > cap {
        let oldest = entries
            .iter()
            .filter(|(_, (_, value))| is_loaded(value))
            .min_by_key(|(_, (used, _))| *used)
            .map(|(key, _)| key.clone());
        match oldest {
            Some(key) => {
                entries.remove(&key);
            }
            None => break,
        }
    }
}
