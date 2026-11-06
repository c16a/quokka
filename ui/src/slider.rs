//! Sliders in the app's own style: the part up to the value is filled, the rest is paler, and a round
//! knob takes the filled colour. The seek bar, the volume and the crossfade setting all use these.

use std::ops::RangeInclusive;

use eframe::egui;

/// Height of the rail, in points.
const RAIL_HEIGHT: f32 = 4.0;
/// Radius of the knob: an 8pt knob.
const KNOB_RADIUS: f32 = 4.0;
/// Height of a slider's row, in points.
pub const HEIGHT: f32 = 16.0;

/// The colours for a theme: the filled part and the knob at 64%, and the rest of the rail at 40%.
/// Black in light mode and white in dark mode.
pub fn colours(dark: bool) -> (egui::Color32, egui::Color32) {
    if dark {
        (
            egui::Color32::from_white_alpha(163),
            egui::Color32::from_white_alpha(102),
        )
    } else {
        (
            egui::Color32::from_black_alpha(163),
            egui::Color32::from_black_alpha(102),
        )
    }
}

/// A slider `width` points wide, placed by the layout. `value` changes while the slider is dragged or
/// clicked, and snaps to `step` when that is not zero. The response reports the change.
pub fn show(
    ui: &mut egui::Ui,
    value: &mut f64,
    range: RangeInclusive<f64>,
    step: f64,
    width: f32,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, HEIGHT), egui::Sense::click_and_drag());
    update(ui, rect, response, value, range, step)
}

/// A slider in `rect`, which the caller places. `salt` tells it apart from the other widgets of the ui.
pub fn show_in(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    salt: &str,
    value: &mut f64,
    range: RangeInclusive<f64>,
    step: f64,
) -> egui::Response {
    let response = ui.interact(
        rect,
        ui.scope_id().with(salt),
        egui::Sense::click_and_drag(),
    );
    update(ui, rect, response, value, range, step)
}

/// Apply a drag or click to `value`, then paint the slider in `rect`.
fn update(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    mut response: egui::Response,
    value: &mut f64,
    range: RangeInclusive<f64>,
    step: f64,
) -> egui::Response {
    let (min, max) = (*range.start(), *range.end());
    let span = (max - min).max(f64::EPSILON);
    // The knob stays inside the rect, so its centre runs between two insets.
    let left = rect.left() + KNOB_RADIUS;
    let right = rect.right() - KNOB_RADIUS;

    let before = *value;
    if (response.dragged() || response.clicked())
        && let Some(pointer) = response.interact_pointer_pos()
    {
        let fraction = f64::from(((pointer.x - left) / (right - left)).clamp(0.0, 1.0));
        let mut next = min + fraction * span;
        if step > 0.0 {
            next = (min + ((next - min) / step).round() * step).clamp(min, max);
        }
        *value = next;
    }
    if *value != before {
        response.mark_changed();
    }

    let fraction = ((*value - min) / span).clamp(0.0, 1.0) as f32;
    let knob_x = left + fraction * (right - left);
    let centre_y = rect.center().y;
    let (filled, rest) = colours(ui.visuals().dark_mode);
    let rail = |from: f32, to: f32| {
        egui::Rect::from_min_max(
            egui::pos2(from, centre_y - RAIL_HEIGHT / 2.0),
            egui::pos2(to, centre_y + RAIL_HEIGHT / 2.0),
        )
    };
    let painter = ui.painter();
    painter.rect_filled(rail(left, right), RAIL_HEIGHT / 2.0, rest);
    painter.rect_filled(rail(left, knob_x), RAIL_HEIGHT / 2.0, filled);
    painter.circle_filled(egui::pos2(knob_x, centre_y), KNOB_RADIUS, filled);
    response
}
