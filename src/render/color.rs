use csscolorparser::Color as CssColor;

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

pub(super) fn to_svg_color(color: Option<&str>) -> Option<String> {
    let normalized = normalize_color(color)?;
    match normalized.as_str() {
        "brightgreen" => Some("#4c1".to_owned()),
        "green" => Some("#97ca00".to_owned()),
        "yellow" => Some("#dfb317".to_owned()),
        "yellowgreen" => Some("#a4a61d".to_owned()),
        "orange" => Some("#fe7d37".to_owned()),
        "red" => Some("#e05d44".to_owned()),
        "blue" => Some("#007ec6".to_owned()),
        "grey" => Some("#555".to_owned()),
        "lightgrey" => Some("#9f9f9f".to_owned()),
        other => Some(other.to_owned()),
    }
}

fn normalize_color(color: Option<&str>) -> Option<String> {
    let color = color?;
    if let Some(named) = normalize_named_color(color) {
        return Some(named.to_owned());
    }
    if is_hex_color(color) {
        return Some(format!(
            "#{}",
            color.trim_start_matches('#').to_ascii_lowercase()
        ));
    }
    if color.trim().parse::<CssColor>().is_ok() {
        return Some(color.trim().to_ascii_lowercase());
    }
    None
}

fn normalize_named_color(color: &str) -> Option<&'static str> {
    match color.trim().to_ascii_lowercase().as_str() {
        "brightgreen" => Some("brightgreen"),
        "green" => Some("green"),
        "yellow" => Some("yellow"),
        "yellowgreen" => Some("yellowgreen"),
        "orange" => Some("orange"),
        "red" => Some("red"),
        "blue" => Some("blue"),
        "grey" | "gray" => Some("grey"),
        "lightgrey" | "lightgray" | "inactive" => Some("lightgrey"),
        "critical" => Some("red"),
        "important" => Some("orange"),
        "success" => Some("brightgreen"),
        "informational" => Some("blue"),
        _ => None,
    }
}

fn is_hex_color(input: &str) -> bool {
    let raw = input.trim().trim_start_matches('#');
    matches!(raw.len(), 3 | 6) && raw.bytes().all(|byte| byte.is_ascii_hexdigit())
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
