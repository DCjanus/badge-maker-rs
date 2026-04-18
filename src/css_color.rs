use std::borrow::Cow;

use crate::css_named_color::parse_named_color;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParsedCssColor<'a> {
    pub(crate) normalized: Cow<'a, str>,
    pub(crate) rgba: [u8; 4],
}

pub(crate) fn parse_css_color(value: &str) -> Option<ParsedCssColor<'_>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = if trimmed.bytes().any(|byte| byte.is_ascii_uppercase()) {
        Cow::Owned(trimmed.to_ascii_lowercase())
    } else {
        Cow::Borrowed(trimmed)
    };
    let rgba = parse_named_color(normalized.as_ref())
        .or_else(|| parse_css_hex_color(normalized.as_ref()))
        .or_else(|| parse_rgb_like_function(normalized.as_ref()))
        .or_else(|| parse_hsl_like_function(normalized.as_ref()))?;

    Some(ParsedCssColor { normalized, rgba })
}

fn parse_css_hex_color(value: &str) -> Option<[u8; 4]> {
    let raw = value.strip_prefix('#')?;
    match raw.len() {
        3 => {
            let [r, g, b] = parse_hex_nibbles::<3>(raw)?;
            Some([expand_nibble(r), expand_nibble(g), expand_nibble(b), 255])
        }
        4 => {
            let [r, g, b, a] = parse_hex_nibbles::<4>(raw)?;
            Some([
                expand_nibble(r),
                expand_nibble(g),
                expand_nibble(b),
                expand_nibble(a),
            ])
        }
        6 => {
            let [r, g, b] = parse_hex_pairs::<3>(raw)?;
            Some([r, g, b, 255])
        }
        8 => {
            let [r, g, b, a] = parse_hex_pairs::<4>(raw)?;
            Some([r, g, b, a])
        }
        _ => None,
    }
}

fn parse_rgb_like_function(value: &str) -> Option<[u8; 4]> {
    let args = function_arguments(value, &["rgb", "rgba"])?;
    if !(args.len() == 3 || args.len() == 4) {
        return None;
    }

    let alpha = match args.get(3) {
        Some(alpha) => parse_alpha(alpha)?,
        None => 1.0,
    };

    let uses_percentages = args[..3].iter().all(|arg| arg.trim_end().ends_with('%'));
    let uses_numbers = args[..3].iter().all(|arg| !arg.trim_end().ends_with('%'));
    if !(uses_percentages || uses_numbers) {
        return None;
    }

    let mut rgba = [0_u8; 4];
    for (index, component) in args[..3].iter().enumerate() {
        rgba[index] = if uses_percentages {
            let percent = parse_non_negative_number(component.trim_end_matches('%'))?;
            scale_unit_interval(percent / 100.0)
        } else {
            let number = parse_non_negative_number(component)?;
            scale_byte_range(number)
        };
    }
    rgba[3] = scale_unit_interval(alpha);
    Some(rgba)
}

fn parse_hsl_like_function(value: &str) -> Option<[u8; 4]> {
    let args = function_arguments(value, &["hsl", "hsla"])?;
    if !(args.len() == 3 || args.len() == 4) {
        return None;
    }

    let hue = parse_finite_number(args[0])?;
    let saturation = parse_percentage(args[1])?;
    let lightness = parse_percentage(args[2])?;
    let alpha = match args.get(3) {
        Some(alpha) => parse_alpha(alpha)?,
        None => 1.0,
    };

    let [r, g, b] = hsl_to_rgb(hue, saturation, lightness);
    Some([r, g, b, scale_unit_interval(alpha)])
}

fn function_arguments<'a>(value: &'a str, names: &[&str]) -> Option<Vec<&'a str>> {
    let open = value.find('(')?;
    let close = value.strip_suffix(')')?;
    let name = value[..open].trim_end();
    if !names.contains(&name) {
        return None;
    }

    let args = close[open + 1..]
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>();
    (!args.iter().any(|arg| arg.is_empty())).then_some(args)
}

fn parse_hex_nibbles<const N: usize>(value: &str) -> Option<[u8; N]> {
    let mut out = [0_u8; N];
    for (index, byte) in value.bytes().enumerate() {
        out[index] = hex_value(byte)?;
    }
    Some(out)
}

fn parse_hex_pairs<const N: usize>(value: &str) -> Option<[u8; N]> {
    let mut out = [0_u8; N];
    let bytes = value.as_bytes();
    for (index, component) in out.iter_mut().enumerate() {
        let hi = hex_value(*bytes.get(index * 2)?)?;
        let lo = hex_value(*bytes.get(index * 2 + 1)?)?;
        *component = (hi << 4) | lo;
    }
    Some(out)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn expand_nibble(value: u8) -> u8 {
    (value << 4) | value
}

fn parse_percentage(value: &str) -> Option<f64> {
    let number = parse_finite_number(value.trim_end_matches('%'))?;
    if value.trim_end().ends_with('%') {
        Some((number / 100.0).clamp(0.0, 1.0))
    } else {
        None
    }
}

fn parse_alpha(value: &str) -> Option<f64> {
    let number = parse_finite_number(value)?;
    (0.0..=1.0).contains(&number).then_some(number)
}

fn parse_non_negative_number(value: &str) -> Option<f64> {
    let number = parse_finite_number(value)?;
    (number >= 0.0).then_some(number)
}

fn parse_finite_number(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    let number = trimmed.parse::<f64>().ok()?;
    number.is_finite().then_some(number)
}

fn scale_unit_interval(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn scale_byte_range(value: f64) -> u8 {
    value.clamp(0.0, 255.0).round() as u8
}

fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> [u8; 3] {
    let h = hue.rem_euclid(360.0) / 360.0;
    let s = saturation.clamp(0.0, 1.0);
    let l = lightness.clamp(0.0, 1.0);

    if s == 0.0 {
        let gray = scale_unit_interval(l);
        return [gray, gray, gray];
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    [
        scale_unit_interval(hue_to_rgb(p, q, h + 1.0 / 3.0)),
        scale_unit_interval(hue_to_rgb(p, q, h)),
        scale_unit_interval(hue_to_rgb(p, q, h - 1.0 / 3.0)),
    ]
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::parse_css_color;

    #[test]
    fn parse_css_color_keeps_normalized_and_rgba_in_sync() {
        let parsed = parse_css_color("  PaPaYaWhIp  ").expect("css color should parse");

        assert_eq!(parsed.normalized.as_ref(), "papayawhip");
        assert_eq!(parsed.rgba, [255, 239, 213, 255]);
    }

    #[test]
    fn parse_css_color_preserves_already_normalized_literals() {
        let parsed = parse_css_color(" #4c1 ").expect("css hex should parse");

        assert_eq!(parsed.normalized.as_ref(), "#4c1");
        assert_eq!(parsed.rgba, [68, 204, 17, 255]);
    }
}
