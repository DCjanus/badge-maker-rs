use crate::{Color, css_color::parse_css_color};

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
    let Some(parsed) = parse_css_color(color) else {
        return 0.0;
    };
    let [r, g, b, _] = parsed.rgba;
    let brightness =
        (f64::from(r) * 299.0 + f64::from(g) * 587.0 + f64::from(b) * 114.0) / 255000.0;
    (brightness * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::colors_for_background;

    #[test]
    fn invalid_background_color_uses_dark_fallback_contrast() {
        let colors = colors_for_background("definitely-not-a-color");

        assert_eq!(colors.text_color, "#fff");
        assert_eq!(colors.shadow_color, "#010101");
    }
}
