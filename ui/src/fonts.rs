//! Fonts. All text is set in Inter, whose licence (SIL Open Font Licence) is in
//! `assets/fonts/Inter-OFL.txt`. Icons are Phosphor glyphs in their own family (licence in
//! `assets/fonts/Phosphor-LICENSE.txt`). egui's bundled fonts stay behind them, for the few
//! symbols neither has.

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily, TextStyle};

/// The bold weight, for emphasis.
const BOLD: &str = "Inter Bold";
/// The semibold weight, for headings.
const SEMIBOLD: &str = "Inter SemiBold";
/// The Phosphor icon font, and its filled weight.
const ICONS: &str = "Phosphor";
const ICONS_FILLED: &str = "Phosphor Fill";

/// Make Inter the font for all text, with headings in semibold.
pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let faces: [(&str, &'static [u8]); 3] = [
        (
            "Inter",
            &include_bytes!("../assets/fonts/Inter-Regular.ttf")[..],
        ),
        (
            SEMIBOLD,
            &include_bytes!("../assets/fonts/Inter-SemiBold.ttf")[..],
        ),
        (BOLD, &include_bytes!("../assets/fonts/Inter-Bold.ttf")[..]),
    ];
    fonts.font_data.insert(
        ICONS.to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Phosphor.ttf"
        ))),
    );
    fonts.font_data.insert(
        ICONS_FILLED.to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Phosphor-Fill.ttf"
        ))),
    );
    for (name, bytes) in faces {
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    }
    // Each family starts with its Inter weight, then falls back to the bundled defaults.
    let defaults = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for (family, first) in [
        (FontFamily::Proportional, "Inter"),
        (FontFamily::Name(SEMIBOLD.into()), SEMIBOLD),
        (FontFamily::Name(BOLD.into()), BOLD),
    ] {
        let mut list = vec![first.to_owned()];
        list.extend(defaults.iter().cloned());
        fonts.families.insert(family, list);
    }
    // Icons use only their own font, so no fallback is needed.
    fonts.families.insert(icon_family(), vec![ICONS.to_owned()]);
    fonts
        .families
        .insert(icon_family_filled(), vec![ICONS_FILLED.to_owned()]);
    ctx.set_fonts(fonts);
    ctx.all_styles_mut(|style| {
        if let Some(heading) = style.text_styles.get_mut(&TextStyle::Heading) {
            heading.family = FontFamily::Name(SEMIBOLD.into());
        }
    });
}

/// The family of the Phosphor icon font.
pub fn icon_family() -> FontFamily {
    FontFamily::Name(ICONS.into())
}

/// The family of the filled Phosphor icons.
pub fn icon_family_filled() -> FontFamily {
    FontFamily::Name(ICONS_FILLED.into())
}

/// The bold family, for text styled by hand.
pub fn bold_family() -> FontFamily {
    FontFamily::Name(BOLD.into())
}

/// Bold text, for emphasis.
pub fn bold(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text).family(bold_family())
}
