//! A button shaped as a squircle: the superellipse |x/a|⁴ + |y/b|⁴ = 1. The button wraps a child widget
//! and is sized to it plus its padding, so its major axis (width) and minor axis (height) come from the
//! child rather than being set by hand.

use eframe::egui;

/// The exponent of the superellipse. 4 gives the classic squircle.
const EXPONENT: f32 = 4.0;
/// Points on each quarter of the outline. The outline is drawn as a polygon with `4 × this` sides.
const POINTS_PER_QUARTER: usize = 48;
/// How far the fill moves towards the text colour while the pointer is over the button.
const HOVER_TINT: f32 = 0.1;

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

/// Space between the child and the outline, on each side, in points.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Padding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Padding {
    /// The same padding on every side.
    pub fn all(amount: f32) -> Self {
        Self::new(amount, amount, amount, amount)
    }

    /// `horizontal` on the left and right, `vertical` on the top and bottom.
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self::new(horizontal, vertical, horizontal, vertical)
    }

    /// Padding for each side, in the order left, top, right, bottom.
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// The space the padding takes up along the x axis.
    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    /// The space the padding takes up along the y axis.
    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

/// A squircle-shaped button around a child widget. Its size is the child's size plus the padding,
/// and its outline is the squircle inscribed in that box. The child is laid out as it would be
/// anywhere else, so a label, an icon or a layout of several widgets all work.
pub struct SquircleButton<W> {
    child: W,
    padding: Padding,
    fill: Option<egui::Color32>,
}

impl<W: egui::Widget> SquircleButton<W> {
    /// A button around `child`, with no padding and the theme's fill.
    pub fn new(child: W) -> Self {
        Self {
            child,
            padding: Padding::default(),
            fill: None,
        }
    }

    /// The space between the child and the outline.
    #[must_use]
    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// The fill colour. Without one, the button uses the theme's inactive widget fill.
    #[must_use]
    pub fn fill(mut self, colour: egui::Color32) -> Self {
        self.fill = Some(colour);
        self
    }

    /// Lay the child out with the padding around it, then paint the squircle behind it. Clicks count
    /// anywhere in the bounding box, and the painted outline is the squircle.
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let padding = self.padding;
        // The fill is painted after the child has set the size, so it reserves a slot now, below the
        // child's shapes. egui's Frame works the same way.
        let background = ui.painter().add(egui::Shape::Noop);

        let available = ui.available_rect_before_wrap();
        let mut bounds = egui::Rect::from_min_max(
            available.min + egui::vec2(padding.left, padding.top),
            available.max - egui::vec2(padding.right, padding.bottom),
        );
        bounds.max.x = bounds.max.x.max(bounds.min.x);
        bounds.max.y = bounds.max.y.max(bounds.min.y);

        let mut content = ui.new_child(egui::UiBuilder::new().max_rect(bounds));
        self.child.ui(&mut content);
        let used = content.min_rect();

        let outer = egui::Rect::from_min_max(
            used.min - egui::vec2(padding.left, padding.top),
            used.max + egui::vec2(padding.right, padding.bottom),
        );
        let response = ui.allocate_rect(outer, egui::Sense::click());

        let hover = ui.ctx().animate_bool_with_time(
            response.id.with("squircle_hover"),
            response.hovered(),
            0.15,
        );
        let text = ui.visuals().text_color();
        let base = self
            .fill
            .unwrap_or(ui.visuals().widgets.inactive.weak_bg_fill);
        let fill = base.lerp_to_gamma(base.lerp_to_gamma(text, HOVER_TINT), hover);

        if ui.is_rect_visible(outer) {
            let centre = outer.center();
            let points = outline(outer.width() / 2.0, outer.height() / 2.0)
                .into_iter()
                .map(|offset| centre + offset)
                .collect();
            ui.painter().set(
                background,
                egui::Shape::convex_polygon(points, fill, egui::Stroke::NONE),
            );
        }
        response
    }
}

impl<W: egui::Widget> egui::Widget for SquircleButton<W> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.show(ui)
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

    #[test]
    fn the_button_is_its_child_plus_the_padding() {
        let ctx = egui::Context::default();
        let padding = Padding::new(6.0, 4.0, 10.0, 8.0);
        let mut child_size = egui::Vec2::ZERO;
        let mut button_size = egui::Vec2::ZERO;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            child_size = ui.add(egui::Label::new("Hello")).rect.size();
            button_size = ui
                .add(SquircleButton::new(egui::Label::new("Hello")).padding(padding))
                .rect
                .size();
        });
        assert!((button_size.x - (child_size.x + padding.horizontal())).abs() < 1e-3);
        assert!((button_size.y - (child_size.y + padding.vertical())).abs() < 1e-3);
    }
}
