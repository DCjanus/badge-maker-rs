use csscolorparser::Color as CssColor;

use crate::Color;

#[derive(Clone, Debug)]
pub(super) struct ColorsForBackground {
    pub(super) text_color: &'static str,
    pub(super) shadow_color: &'static str,
}

pub(super) fn colors_for_background(color: &str) -> ColorsForBackground {
    if brightness(color) <= 0.69 {
        ColorsForBackground {
            text_color: "#fff",
            shadow_color: "#010101",
        }
    } else {
        ColorsForBackground {
            text_color: "#333",
            shadow_color: "#ccc",
        }
    }
}

pub(super) fn to_svg_color(color: Option<&Color>) -> Option<String> {
    color.and_then(Color::to_svg_color)
}

fn brightness(color: &str) -> f64 {
    let Ok(parsed) = color.parse::<CssColor>() else {
        return 0.0;
    };
    let [r, g, b, _] = parsed.to_rgba8();
    let brightness =
        (f64::from(r) * 299.0 + f64::from(g) * 587.0 + f64::from(b) * 114.0) / 255000.0;
    (brightness * 100.0).round() / 100.0
}
