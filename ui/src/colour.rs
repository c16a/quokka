//! Colours in OKLab, the perceptual space: equal steps in lightness look equal to the eye, and a
//! hue angle means the same hue at every lightness. Colours are converted back to sRGB to draw.

use eframe::egui::Color32;

/// A colour in OKLab: lightness `l`, from 0 (black) to 1 (white), and the two opposing axes `a`
/// (green to red) and `b` (blue to yellow).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl Lab {
    pub fn from_srgb(colour: Color32) -> Self {
        let r = srgb_to_linear(colour.r());
        let g = srgb_to_linear(colour.g());
        let b = srgb_to_linear(colour.b());
        let l = (0.412_221_47 * r + 0.536_332_54 * g + 0.051_445_99 * b).cbrt();
        let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
        let s = (0.088_302_46 * r + 0.281_718_84 * g + 0.629_978_7 * b).cbrt();
        Lab {
            l: 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
            a: 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
            b: 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
        }
    }

    /// The linear sRGB channels of this colour. They can fall outside 0 to 1 when the colour is
    /// more vivid than sRGB can show.
    fn linear_rgb(self) -> [f32; 3] {
        let l_ = self.l + 0.396_337_78 * self.a + 0.215_803_76 * self.b;
        let m_ = self.l - 0.105_561_35 * self.a - 0.063_854_17 * self.b;
        let s_ = self.l - 0.089_484_18 * self.a - 1.291_485_5 * self.b;
        let (l, m, s) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
        [
            4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_93 * s,
            -1.268_438 * l + 2.609_757_4 * m - 0.341_319_4 * s,
            -0.004_196_086 * l - 0.703_418_6 * m + 1.707_614_7 * s,
        ]
    }

    /// The colour in sRGB. A colour sRGB cannot show keeps its lightness and hue, and loses chroma
    /// until it fits.
    pub fn to_srgb(self) -> Color32 {
        let fits = |lab: Lab| {
            lab.linear_rgb()
                .iter()
                .all(|v| (-0.000_1..=1.000_1).contains(v))
        };
        let fitted = if fits(self) {
            self
        } else {
            let (mut low, mut high) = (0.0_f32, 1.0_f32);
            for _ in 0..24 {
                let mid = (low + high) / 2.0;
                let candidate = Lab {
                    a: self.a * mid,
                    b: self.b * mid,
                    ..self
                };
                if fits(candidate) {
                    low = mid;
                } else {
                    high = mid;
                }
            }
            Lab {
                a: self.a * low,
                b: self.b * low,
                ..self
            }
        };
        let [r, g, b] = fitted.linear_rgb();
        Color32::from_rgb(linear_to_srgb(r), linear_to_srgb(g), linear_to_srgb(b))
    }

    /// The same hue and chroma, at lightness `l`.
    pub fn with_lightness(self, l: f32) -> Self {
        Lab { l, ..self }
    }

    /// The opposite hue, at the same lightness and chroma.
    pub fn complement(self) -> Self {
        Lab {
            a: -self.a,
            b: -self.b,
            ..self
        }
    }
}

/// The WCAG contrast ratio between two colours, from 1 (no contrast) to 21 (black on white).
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    let (x, y) = (relative_luminance(a), relative_luminance(b));
    let (light, dark) = if x > y { (x, y) } else { (y, x) };
    (light + 0.05) / (dark + 0.05)
}

/// The colour with the hue of `hue`, closest in lightness to `background`, that still contrasts with
/// `background` by at least `min_ratio`. When no tinted colour reaches that, black or white does
/// instead, whichever contrasts more.
pub fn readable_on(background: Color32, hue: Lab, min_ratio: f32) -> Color32 {
    let background_l = Lab::from_srgb(background).l;
    let mut best: Option<(f32, Color32)> = None;
    for step in 0..=98 {
        let l = 0.02 + step as f32 * 0.01;
        let candidate = hue.with_lightness(l).to_srgb();
        if contrast_ratio(candidate, background) < min_ratio {
            continue;
        }
        let distance = (l - background_l).abs();
        if best.is_none_or(|(closest, _)| distance < closest) {
            best = Some((distance, candidate));
        }
    }
    match best {
        Some((_, colour)) => colour,
        None if contrast_ratio(Color32::WHITE, background)
            >= contrast_ratio(Color32::BLACK, background) =>
        {
            Color32::WHITE
        }
        None => Color32::BLACK,
    }
}

fn relative_luminance(colour: Color32) -> f32 {
    0.2126 * srgb_to_linear(colour.r())
        + 0.7152 * srgb_to_linear(colour.g())
        + 0.0722 * srgb_to_linear(colour.b())
}

fn srgb_to_linear(value: u8) -> f32 {
    let c = f32::from(value) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> u8 {
    let c = value.clamp(0.0, 1.0);
    let encoded = if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
}

/// The colour that covers most of a poster, found in OKLab. Near-black, near-white and grey pixels
/// are left out, so the art's colour shows rather than its background. `None` when no colour stands
/// out.
pub(crate) fn dominant_colour(image: &eframe::egui::ColorImage) -> Option<eframe::egui::Color32> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_survive_a_round_trip() {
        for colour in [
            Color32::from_rgb(255, 0, 0),
            Color32::from_rgb(20, 140, 200),
            Color32::from_rgb(240, 240, 240),
            Color32::from_rgb(0, 0, 0),
        ] {
            let back = Lab::from_srgb(colour).to_srgb();
            for (x, y) in [
                (colour.r(), back.r()),
                (colour.g(), back.g()),
                (colour.b(), back.b()),
            ] {
                assert!(
                    (i16::from(x) - i16::from(y)).abs() <= 1,
                    "{colour:?} came back as {back:?}"
                );
            }
        }
    }

    #[test]
    fn the_complement_keeps_lightness_and_chroma() {
        let lab = Lab::from_srgb(Color32::from_rgb(40, 110, 126));
        let opposite = lab.complement();
        assert_eq!(opposite.l, lab.l);
        assert!((opposite.a.hypot(opposite.b) - lab.a.hypot(lab.b)).abs() < 1e-6);
        assert_eq!(opposite.a, -lab.a);
    }

    #[test]
    fn contrast_runs_from_one_to_twenty_one() {
        assert!((contrast_ratio(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
        assert!((contrast_ratio(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn readable_colours_meet_the_ratio_on_dark_and_light_backgrounds() {
        let hue = Lab::from_srgb(Color32::from_rgb(40, 110, 126)).complement();
        for background in [
            Color32::from_rgb(29, 42, 51),
            Color32::from_rgb(230, 238, 241),
            Color32::from_rgb(120, 120, 120),
        ] {
            let text = readable_on(background, hue, 4.5);
            assert!(
                contrast_ratio(text, background) >= 4.5,
                "{background:?} gave {text:?}"
            );
        }
    }

    #[test]
    fn out_of_gamut_colours_still_draw() {
        let vivid = Lab {
            l: 0.5,
            a: 0.6,
            b: 0.6,
        };
        let _ = vivid.to_srgb();
    }
}
