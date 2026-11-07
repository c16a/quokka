//! A button with Apple-style continuous corners: straight sides, and each corner is a superellipse
//! |x/r|⁴ + |y/r|⁴ = 1 of radius `r`. A wide button is a pill with flat top and bottom edges, and a
//! square one is a full squircle. The button wraps a child widget and is sized to it plus its padding.

use eframe::egui;

/// The exponent of the superellipse that each corner follows. 4 gives the classic squircle.
const EXPONENT: f32 = 4.0;
/// Points on each corner of the outline. The outline is drawn as a polygon with `4 × (this + 1)` points.
const POINTS_PER_CORNER: usize = 48;
/// How far the fill moves towards the text colour while the pointer is over the button.
const HOVER_TINT: f32 = 0.1;

/// The outline of a button with half-axes `a` and `b` and corners of radius `radius`, centred on the
/// origin. The radius is limited to what the box allows. The points go round once, starting on the
/// right-hand side.
pub fn outline(a: f32, b: f32, radius: f32) -> Vec<egui::Vec2> {
    let r = radius.clamp(0.0, a.min(b));
    let power = 2.0 / EXPONENT;
    // Each corner is a quarter turn of a superellipse, centred where its straight sides meet it.
    let corners = [
        (a - r, b - r, 0.0),
        (-(a - r), b - r, std::f32::consts::FRAC_PI_2),
        (-(a - r), -(b - r), std::f32::consts::PI),
        (a - r, -(b - r), 3.0 * std::f32::consts::FRAC_PI_2),
    ];
    let mut points = Vec::with_capacity(4 * (POINTS_PER_CORNER + 1));
    for (cx, cy, start) in corners {
        for i in 0..=POINTS_PER_CORNER {
            let angle = start + std::f32::consts::FRAC_PI_2 * i as f32 / POINTS_PER_CORNER as f32;
            let (sin, cos) = angle.sin_cos();
            points.push(egui::vec2(
                cx + r * signed_power(cos, power),
                cy + r * signed_power(sin, power),
            ));
        }
    }
    points
}

/// Whether `offset` from the centre lies inside a button with half-axes `a` and `b` and corners of
/// radius `radius`.
pub fn contains(a: f32, b: f32, radius: f32, offset: egui::Vec2) -> bool {
    let r = radius.clamp(0.0, a.min(b));
    let (x, y) = (offset.x.abs(), offset.y.abs());
    if x > a || y > b {
        return false;
    }
    // Only the corners curve in: beyond the straight sides' inset, the superellipse decides.
    let dx = x - (a - r);
    let dy = y - (b - r);
    if dx <= 0.0 || dy <= 0.0 {
        return true;
    }
    (dx / r).powf(EXPONENT) + (dy / r).powf(EXPONENT) <= 1.0
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

/// A button with continuous corners around a child widget. Its size is the child's size plus the
/// padding, and its outline fills that box. The child is laid out as it would be anywhere else, so a
/// label, an icon or a layout of several widgets all work.
pub struct SquircleButton<W> {
    child: W,
    padding: Padding,
    fill: Option<egui::Color32>,
    stroke: Option<egui::Stroke>,
    radius: Option<f32>,
}

impl<W: egui::Widget> SquircleButton<W> {
    /// A button around `child`, with no padding, the theme's fill, no outline and the default corners.
    pub fn new(child: W) -> Self {
        Self {
            child,
            padding: Padding::default(),
            fill: None,
            stroke: None,
            radius: None,
        }
    }

    /// The radius of the corners, in points. Without one, the corners are as round as the box allows:
    /// a pill for a wide button, and a full squircle for a square one.
    #[must_use]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    /// An outline drawn round the button. Without one, there is none.
    #[must_use]
    pub fn stroke(mut self, stroke: egui::Stroke) -> Self {
        self.stroke = Some(stroke);
        self
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

    /// Lay the child out with the padding around it, then paint the button behind it. Clicks count
    /// anywhere in the bounding box, and the painted outline has the continuous corners.
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
            // Without a radius, the corners are as round as the box allows.
            let radius = self.radius.unwrap_or(f32::MAX);
            let points = outline(outer.width() / 2.0, outer.height() / 2.0, radius)
                .into_iter()
                .map(|offset| centre + offset)
                .collect();
            let stroke = self.stroke.unwrap_or(egui::Stroke::NONE);
            ui.painter().set(
                background,
                egui::Shape::convex_polygon(points, fill, stroke),
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

/// The padding round a button's text.
pub const TEXT_PADDING: Padding = Padding {
    left: 16.0,
    top: 8.0,
    right: 16.0,
    bottom: 8.0,
};

/// A button with one line of text, in the theme's text colour and fill. The text is never wrapped,
/// so the button grows with it.
pub fn text_button(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> egui::Response {
    SquircleButton::new(egui::Label::new(text).extend())
        .padding(TEXT_PADDING)
        .show(ui)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The value of the corner equation for a point: 1 on the outline, less inside.
    fn boundary_value(a: f32, b: f32, r: f32, point: egui::Vec2) -> f32 {
        let dx = (point.x.abs() - (a - r)).max(0.0) / r;
        let dy = (point.y.abs() - (b - r)).max(0.0) / r;
        dx.powf(EXPONENT) + dy.powf(EXPONENT)
    }

    #[test]
    fn every_outline_point_is_on_the_boundary() {
        for (a, b, r) in [(80.0, 30.0, 30.0), (30.0, 30.0, 30.0), (60.0, 40.0, 12.0)] {
            for point in outline(a, b, r) {
                let value = boundary_value(a, b, r, point);
                assert!((value - 1.0).abs() < 1e-3, "{point:?} gives {value}");
            }
        }
    }

    #[test]
    fn the_straight_sides_are_flat() {
        let (a, b, r) = (80.0, 30.0, 12.0);
        for point in outline(a, b, r) {
            if point.x.abs() <= a - r {
                assert!((point.y.abs() - b).abs() < 1e-3, "{point:?} bows");
            }
        }
    }

    #[test]
    fn the_outline_reaches_both_half_axes() {
        let (a, b, r) = (80.0, 30.0, 12.0);
        let points = outline(a, b, r);
        let max_x = points.iter().map(|p| p.x).fold(f32::MIN, f32::max);
        let max_y = points.iter().map(|p| p.y).fold(f32::MIN, f32::max);
        assert!((max_x - a).abs() < 1e-3);
        assert!((max_y - b).abs() < 1e-3);
    }

    #[test]
    fn a_square_with_full_radius_is_a_squircle() {
        let (a, b) = (30.0, 30.0);
        for point in outline(a, b, f32::MAX) {
            let on_curve = (point.x / a).abs().powf(EXPONENT) + (point.y / b).abs().powf(EXPONENT);
            assert!((on_curve - 1.0).abs() < 1e-3, "{point:?} gives {on_curve}");
        }
    }

    #[test]
    fn containment_follows_the_outline() {
        // A pill: the flat sides run between the rounded ends.
        let (a, b, r) = (80.0, 30.0, 30.0);
        assert!(contains(a, b, r, egui::vec2(0.0, 0.0)));
        assert!(contains(a, b, r, egui::vec2(a - 0.5, 0.0)));
        assert!(contains(a, b, r, egui::vec2(0.0, b - 0.5)));
        assert!(!contains(a, b, r, egui::vec2(a - 1.0, b - 1.0)));
        assert!(!contains(a, b, r, egui::vec2(a + 1.0, 0.0)));
        // A square with full radius: the squircle.
        let (a, b) = (30.0, 30.0);
        assert!(contains(a, b, f32::MAX, egui::vec2(0.7 * a, 0.7 * b)));
        assert!(!contains(a, b, f32::MAX, egui::vec2(0.9 * a, 0.9 * b)));
    }

    #[test]
    fn a_button_inside_a_row_is_only_as_tall_as_its_child() {
        let ctx = egui::Context::default();
        let padding = Padding::new(6.0, 4.0, 10.0, 8.0);
        let mut child_height = 0.0;
        let mut button_rect = egui::Rect::NOTHING;
        let mut row_top = 0.0;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.add_space(40.0);
            ui.horizontal(|ui| {
                row_top = ui.cursor().top();
                ui.label("before");
                let inner = ui.add(egui::Label::new("Hello").extend());
                child_height = inner.rect.height();
                button_rect = ui
                    .add(
                        SquircleButton::new(egui::Label::new("Resume at 21:08").extend())
                            .padding(padding),
                    )
                    .rect;
            });
        });
        assert!(
            button_rect.top() >= row_top - 1e-3,
            "button starts at {} above its row at {}",
            button_rect.top(),
            row_top
        );
        assert!(
            button_rect.height() < child_height + padding.vertical() + 40.0,
            "button is {} tall",
            button_rect.height()
        );
    }

    #[test]
    fn a_button_in_a_column_beside_a_poster_is_only_as_tall_as_its_child() {
        let ctx = egui::Context::default();
        let padding = Padding::new(6.0, 4.0, 10.0, 8.0);
        let mut button_rect = egui::Rect::NOTHING;
        let mut poster_rect = egui::Rect::NOTHING;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.horizontal(|ui| {
                poster_rect = ui
                    .allocate_exact_size(egui::vec2(160.0, 240.0), egui::Sense::hover())
                    .0;
                ui.vertical(|ui| {
                    ui.label("MOVIE");
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        button_rect = ui
                            .add(
                                SquircleButton::new(egui::Label::new("Resume at 21:08").extend())
                                    .padding(padding),
                            )
                            .rect;
                        ui.add(SquircleButton::new(egui::Label::new("Start over").extend()));
                    });
                });
            });
        });
        assert!(
            button_rect.top() > poster_rect.top() + 20.0,
            "button rect {button_rect:?} starts above the column"
        );
        assert!(
            button_rect.height() < 60.0,
            "button rect {button_rect:?} is too tall"
        );
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
