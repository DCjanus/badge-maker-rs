use std::{fmt, str::FromStr};

/// Strongly-typed badge color input.
///
/// The common path is `"brightgreen".parse::<Color>()`. [`Color::Literal`]
/// remains available as a low-level compatibility escape hatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Color {
    /// One of the documented Shields named or semantic colors.
    Named(NamedColor),
    /// A hexadecimal color literal such as `#4c1` or `ABC123`.
    Hex(String),
    /// A CSS color literal such as `rgb(...)`, `hsl(...)`, or `papayawhip`.
    Css(String),
    /// Low-level escape hatch for raw color input compatibility.
    Literal(String),
}

impl Color {
    /// Creates a low-level raw color literal.
    pub fn literal(value: impl Into<String>) -> Self {
        Self::Literal(value.into())
    }

    pub(crate) fn to_svg_color(&self) -> Option<String> {
        match self {
            Self::Named(named) => Some(named.to_svg_color().to_owned()),
            Self::Hex(value) => normalize_hex_color(value),
            Self::Css(value) => crate::css_color::normalize_css_color(value),
            Self::Literal(value) => normalize_literal_color(value),
        }
    }
}

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(named) = normalize_named_color(value) {
            return Ok(Self::Named(named));
        }
        if is_hex_color(value) {
            return Ok(Self::Hex(value.trim().to_owned()));
        }
        if crate::css_color::normalize_css_color(value).is_some() {
            return Ok(Self::Css(value.trim().to_owned()));
        }
        Err(ParseColorError)
    }
}

impl From<NamedColor> for Color {
    fn from(value: NamedColor) -> Self {
        Self::Named(value)
    }
}

/// Failed to parse a color string into [`Color`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseColorError;

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid badge color: expected a named color, #rgb/#rrggbb, or a CSS color")
    }
}

impl std::error::Error for ParseColorError {}

/// Documented Shields named colors and semantic aliases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamedColor {
    Brightgreen,
    Green,
    Yellow,
    Yellowgreen,
    Orange,
    Red,
    Blue,
    Grey,
    Gray,
    Lightgrey,
    Lightgray,
    Success,
    Important,
    Critical,
    Informational,
    Inactive,
}

impl NamedColor {
    /// Returns the public-facing Shields color name for this variant.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Brightgreen => "brightgreen",
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Yellowgreen => "yellowgreen",
            Self::Orange => "orange",
            Self::Red => "red",
            Self::Blue => "blue",
            Self::Grey => "grey",
            Self::Gray => "gray",
            Self::Lightgrey => "lightgrey",
            Self::Lightgray => "lightgray",
            Self::Success => "success",
            Self::Important => "important",
            Self::Critical => "critical",
            Self::Informational => "informational",
            Self::Inactive => "inactive",
        }
    }

    pub(crate) const fn to_svg_color(self) -> &'static str {
        match self {
            Self::Brightgreen | Self::Success => "#4c1",
            Self::Green => "#97ca00",
            Self::Yellow => "#dfb317",
            Self::Yellowgreen => "#a4a61d",
            Self::Orange | Self::Important => "#fe7d37",
            Self::Red | Self::Critical => "#e05d44",
            Self::Blue | Self::Informational => "#007ec6",
            Self::Grey | Self::Gray => "#555",
            Self::Lightgrey | Self::Lightgray | Self::Inactive => "#9f9f9f",
        }
    }
}

impl fmt::Display for NamedColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NamedColor {
    type Err = ParseColorError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        normalize_named_color(value).ok_or(ParseColorError)
    }
}

fn normalize_literal_color(value: &str) -> Option<String> {
    normalize_named_color(value)
        .map(|named| named.to_svg_color().to_owned())
        .or_else(|| normalize_literal_hex_color(value))
        .or_else(|| normalize_literal_css_color(value))
}

fn normalize_hex_color(value: &str) -> Option<String> {
    if is_hex_color(value) {
        Some(format!(
            "#{}",
            value.trim().trim_start_matches('#').to_ascii_lowercase()
        ))
    } else {
        None
    }
}

fn normalize_literal_hex_color(value: &str) -> Option<String> {
    if is_literal_hex_color(value) {
        Some(format!(
            "#{}",
            value.trim_start_matches('#').to_ascii_lowercase()
        ))
    } else {
        None
    }
}

fn normalize_literal_css_color(value: &str) -> Option<String> {
    let trimmed = value.trim();

    if trimmed.eq_ignore_ascii_case("transparent") {
        return None;
    }
    if trimmed.starts_with('#') {
        return crate::css_color::normalize_css_color(trimmed);
    }
    if trimmed != trimmed.to_ascii_lowercase() {
        return None;
    }

    if crate::css_color::normalize_css_color(trimmed).is_some() {
        Some(trimmed.to_owned())
    } else {
        None
    }
}

fn normalize_named_color(value: &str) -> Option<NamedColor> {
    match value.trim().to_ascii_lowercase().as_str() {
        "brightgreen" => Some(NamedColor::Brightgreen),
        "green" => Some(NamedColor::Green),
        "yellow" => Some(NamedColor::Yellow),
        "yellowgreen" => Some(NamedColor::Yellowgreen),
        "orange" => Some(NamedColor::Orange),
        "red" => Some(NamedColor::Red),
        "blue" => Some(NamedColor::Blue),
        "grey" => Some(NamedColor::Grey),
        "gray" => Some(NamedColor::Gray),
        "lightgrey" => Some(NamedColor::Lightgrey),
        "lightgray" => Some(NamedColor::Lightgray),
        "success" => Some(NamedColor::Success),
        "important" => Some(NamedColor::Important),
        "critical" => Some(NamedColor::Critical),
        "informational" => Some(NamedColor::Informational),
        "inactive" => Some(NamedColor::Inactive),
        _ => None,
    }
}

fn is_hex_color(input: &str) -> bool {
    let raw = input.trim().trim_start_matches('#');
    matches!(raw.len(), 3 | 6) && raw.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_literal_hex_color(input: &str) -> bool {
    let raw = input.strip_prefix('#').unwrap_or(input);
    matches!(raw.len(), 3 | 6)
        && !raw.starts_with('#')
        && raw.bytes().all(|byte| byte.is_ascii_hexdigit())
}
