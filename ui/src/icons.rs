//! Icons from Phosphor (Regular weight, MIT licence in `assets/fonts/Phosphor-LICENSE.txt`).
//! Each icon is one glyph of the Phosphor font, which `fonts::install` registers as its own
//! family. The codepoints are those of egui-phosphor 0.14.0, which bundles the same font.

use eframe::egui::{Align2, Color32, FontId, Painter, Pos2};

use crate::fonts;

pub const AIRPLAY: &str = "\u{E004}";
pub const CLOCK_COUNTER_CLOCKWISE: &str = "\u{E1A0}";
pub const FILM_STRIP: &str = "\u{E792}";
pub const GEAR: &str = "\u{E270}";
pub const MICROPHONE_STAGE: &str = "\u{E75C}";
pub const PLAYLIST: &str = "\u{E6AA}";
pub const RADIO: &str = "\u{E77E}";
pub const VINYL_RECORD: &str = "\u{ECAC}";
pub const CARET_LEFT: &str = "\u{E138}";
pub const DOTS_SIX_VERTICAL: &str = "\u{EAE2}";
pub const EQUALIZER: &str = "\u{EBBC}";
pub const INFO: &str = "\u{E2CE}";
pub const MINUS: &str = "\u{E32A}";
pub const MAGNIFYING_GLASS: &str = "\u{E30C}";
pub const PAUSE: &str = "\u{E39E}";
pub const PLUS: &str = "\u{E3D4}";
pub const PLAY: &str = "\u{E3D0}";
pub const QUEUE: &str = "\u{E6AC}";
pub const QUOTES: &str = "\u{E660}";
pub const REPEAT: &str = "\u{E3F6}";
pub const REPEAT_ONCE: &str = "\u{E3F8}";
pub const SHUFFLE: &str = "\u{E422}";
pub const SKIP_BACK: &str = "\u{E5A4}";
pub const SKIP_FORWARD: &str = "\u{E5A6}";
pub const SPEAKER_HIGH: &str = "\u{E44A}";
pub const SPEAKER_X: &str = "\u{E45C}";
pub const STAR: &str = "\u{E46A}";
pub const X: &str = "\u{E4F6}";

/// Draw `glyph` centred on `center`, `size` points tall, in `color`.
pub fn paint(painter: &Painter, center: Pos2, glyph: &str, size: f32, color: Color32) {
    painter.text(
        center,
        Align2::CENTER_CENTER,
        glyph,
        FontId::new(size, fonts::icon_family()),
        color,
    );
}

/// Like [`paint`], with the filled weight of the same glyph (for example a starred favourite).
pub fn paint_filled(painter: &Painter, center: Pos2, glyph: &str, size: f32, color: Color32) {
    painter.text(
        center,
        Align2::CENTER_CENTER,
        glyph,
        FontId::new(size, fonts::icon_family_filled()),
        color,
    );
}
