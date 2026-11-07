//! A button shaped as a squircle: the superellipse |x/a|⁴ + |y/b|⁴ = 1. The button's major axis is
//! its width and its minor axis is its height, so `a` and `b` are half of each.

use eframe::egui;

/// The exponent of the superellipse. 4 gives the classic squircle.
const EXPONENT: f32 = 4.0;
/// Points on each quarter of the outline. The outline is drawn as a polygon with `4 × this` sides.
const POINTS_PER_QUARTER: usize = 48;
/// How far the fill moves towards the text colour while the pointer is over the button.
const HOVER_TINT: f32 = 0.1;
/// Text size, in points.
const TEXT_SIZE: f32 = 14.0;

/// The outline of a squircle with half-axes `a` and `b`, centred on the origin. The points go round
/// once, starting on the right-hand end of the major axis.
pub fn outline(a: f32, b: f32) -> Vec<egui::Vec2> {
    let count = 4 * POINTS_PER_QUARTER;
    let power = 2.0 / EXPONENT;
    (0..count)
        .map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / count as f32;
            let (sin, cos) = angle.sin_cos();
            egui::vec2(a * signed_power(cos, power), b * signed_power(sin, power))
        })
        .collect()
}

/// Whether `offset` from the centre lies inside the squircle with half-axes `a` and `b`.
pub fn contains(a: f32, b: f32, offset: egui::Vec2) -> bool {
    (offset.x / a).abs().powf(EXPONENT) + (offset.y / b).abs().powf(EXPONENT) <= 1.0
}

/// `value` raised to `power`, keeping its sign, so the curve is symmetric about both axes.
fn signed_power(value: f32, power: f32) -> f32 {
    value.signum() * value.abs().powf(power)
}

/// A squircle-shaped button, `major` points wide and `minor` points high.
pub struct SquircleButton<'a> {
    text: &'a str,
    major: f32,
    minor: f32,
    fill: Option<egui::Color32>,
    text_color: Option<egui::Color32>,
}

impl<'a> SquircleButton<'a> {
    /// A button with `text` inside, of the given major and minor axes, in points.
    pub fn new(text: &'a str, major: f32, minor: f32) -> Self {
        Self {
            text,
            major,
            minor,
            fill: None,
            text_color: None,
        }
    }

    /// The fill colour. Without one, the button uses the theme's inactive widget fill.
    #[must_use]
    pub fn fill(mut self, colour: egui::Color32) -> Self {
        self.fill = Some(colour);
        self
    }

    /// The text colour. Without one, the button uses the theme's text colour.
    #[must_use]
    pub fn text_color(mut self, colour: egui::Color32) -> Self {
        self.text_color = Some(colour);
        self
    }

    /// Lay the button out and paint it. Clicks count anywhere in its bounding box; the painted
    /// outline is the squircle.
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let size = egui::vec2(self.major, self.minor);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, self.text));

        let visuals = ui.visuals();
        let text_color = self.text_color.unwrap_or(visuals.text_color());
        let base = self.fill.unwrap_or(visuals.widgets.inactive.weak_bg_fill);
        let hover = ui.ctx().animate_bool_with_time(
            response.id.with("squircle_hover"),
            response.hovered(),
            0.15,
        );
        let fill = base.lerp_to_gamma(base.lerp_to_gamma(text_color, HOVER_TINT), hover);

        let centre = rect.center();
        let points = outline(self.major / 2.0, self.minor / 2.0)
            .into_iter()
            .map(|offset| centre + offset)
            .collect();
        let painter = ui.painter();
        painter.add(egui::Shape::convex_polygon(
            points,
            fill,
            egui::Stroke::NONE,
        ));
        painter.text(
            centre,
            egui::Align2::CENTER_CENTER,
            self.text,
            egui::FontId::proportional(TEXT_SIZE),
            text_color,
        );
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_outline_lies_on_the_superellipse() {
        let (a, b) = (80.0, 30.0);
        for point in outline(a, b) {
            let on_curve = (point.x / a).abs().powf(EXPONENT) + (point.y / b).abs().powf(EXPONENT);
            assert!((on_curve - 1.0).abs() < 1e-3, "{point:?} gives {on_curve}");
        }
    }

    #[test]
    fn the_outline_reaches_both_half_axes() {
        let (a, b) = (80.0, 30.0);
        let points = outline(a, b);
        let max_x = points.iter().map(|p| p.x).fold(f32::MIN, f32::max);
        let max_y = points.iter().map(|p| p.y).fold(f32::MIN, f32::max);
        assert!((max_x - a).abs() < 1e-3);
        assert!((max_y - b).abs() < 1e-3);
    }

    #[test]
    fn the_corners_of_the_bounding_box_are_outside() {
        let (a, b) = (80.0, 30.0);
        assert!(contains(a, b, egui::vec2(0.0, 0.0)));
        assert!(contains(a, b, egui::vec2(0.7 * a, 0.7 * b)));
        assert!(!contains(a, b, egui::vec2(0.9 * a, 0.9 * b)));
        assert!(!contains(a, b, egui::vec2(a, b)));
    }
}
