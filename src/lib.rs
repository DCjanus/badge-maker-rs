//! Rust badge renderer targeting Shields-like SVG output.
//!
//! The crate currently focuses on SVG badge generation rather than full
//! JavaScript API compatibility. Start with [`BadgeOptions`] and render with
//! [`make_badge`].
//!
//! # Example
//!
//! ```
//! use badge_maker_rs::{BadgeOptions, Style, make_badge};
//!
//! let mut options = BadgeOptions::new("passing");
//! options.label = "build".to_owned();
//! options.color = Some("brightgreen".to_owned());
//! options.style = Style::Flat;
//!
//! let svg = make_badge(&options).unwrap();
//! assert!(svg.starts_with("<svg "));
//! assert!(svg.contains("aria-label=\"build: passing\""));
//! ```
//!
//! # Compatibility Boundary
//!
//! This crate intentionally exposes a Rust-first API instead of mirroring the
//! upstream JavaScript package surface.
//!
//! - Supported: SVG rendering for the five official badge styles.
//! - Supported: upstream-aligned layout, escaping, trimming, colors, links,
//!   logos, and `id_suffix` behavior.
//! - Supported: ergonomic two-link setters and upstream semantic color aliases
//!   such as `success` and `critical`.
//! - Not exposed: JavaScript object validation, `ValidationError`, JSON output,
//!   raster output, or other Node-specific entry points.

mod anafanafo;
mod render;

use thiserror::Error;

#[doc = include_str!("../docs/style-previews.md")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Style {
    Plastic,
    Flat,
    FlatSquare,
    ForTheBadge,
    Social,
}

/// User-facing badge rendering options.
///
/// Construct with [`BadgeOptions::new`] and then override the optional fields
/// you need.
///
/// The public API intentionally models the stable rendering inputs directly
/// instead of reproducing the upstream JavaScript validation wrapper.
///
/// Public input semantics:
///
/// - `label` and `message` are trimmed before layout.
/// - Text and attribute content are XML-escaped during SVG generation.
/// - `color` and `label_color` silently fall back to style defaults when the
///   provided value is not a recognized Shields color, hex color, or CSS color.
/// - `links` uses at most the first two elements; extra entries are ignored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BadgeOptions {
    /// Left-hand label text. Empty by default.
    pub label: String,
    /// Right-hand message text.
    pub message: String,
    /// Message-side color. Named Shields colors, CSS colors, and hex colors are supported.
    pub color: Option<String>,
    /// Label-side color. Defaults to the standard Shields label color when unset.
    pub label_color: Option<String>,
    /// Badge visual style.
    pub style: Style,
    /// Optional logo data URL embedded as an SVG `<image>`.
    pub logo_base64: Option<String>,
    /// Explicit logo width in pixels.
    ///
    /// This is a Rust-side convenience override for callers that already know
    /// the desired rendered logo width. It is not part of the upstream
    /// `badge-maker` public JavaScript API.
    pub logo_width: Option<u32>,
    /// Optional badge links.
    ///
    /// The first link targets the left/body region, and the second targets the
    /// message region. Supplying only the first link wraps the whole badge body,
    /// matching the upstream SVG structure. If more than two links are
    /// supplied, only the first two are used.
    pub links: Vec<String>,
    /// Optional suffix appended to generated SVG IDs to avoid collisions on the same page.
    pub id_suffix: Option<String>,
}

impl BadgeOptions {
    /// Creates options with the given message and sensible Shields-like defaults.
    ///
    /// This is the smallest useful entry point for callers:
    ///
    /// ```
    /// use badge_maker_rs::{BadgeOptions, make_badge};
    ///
    /// let svg = make_badge(&BadgeOptions::new("passing")).unwrap();
    /// assert!(svg.contains("passing"));
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            label: String::new(),
            message: message.into(),
            color: None,
            label_color: None,
            style: Style::Flat,
            logo_base64: None,
            logo_width: None,
            links: Vec::new(),
            id_suffix: None,
        }
    }

    /// Replaces the raw `links` vector using explicit left/right slots.
    ///
    /// `left` targets the left or full badge region, while `right` targets the
    /// message region. Passing `None` for `left` and `Some(...)` for `right`
    /// preserves the upstream "right-only link" structure.
    pub fn set_links(&mut self, left: Option<impl Into<String>>, right: Option<impl Into<String>>) {
        self.links = match (left, right) {
            (None, None) => Vec::new(),
            (Some(left), None) => vec![left.into()],
            (None, Some(right)) => vec![String::new(), right.into()],
            (Some(left), Some(right)) => vec![left.into(), right.into()],
        };
    }

    /// Builder-style variant of [`BadgeOptions::set_links`].
    pub fn with_links(
        mut self,
        left: Option<impl Into<String>>,
        right: Option<impl Into<String>>,
    ) -> Self {
        self.set_links(left, right);
        self
    }
}

/// Rendering errors returned by [`make_badge`].
#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum Error {
    /// `id_suffix` contained characters outside `[A-Za-z0-9_-]`.
    #[error("Field `idSuffix` must contain only numbers, letters, -, and _")]
    InvalidIdSuffix,
    /// Reserved for styles that may be declared before their renderer exists.
    #[error("style `{0}` is not implemented yet")]
    UnimplementedStyle(&'static str),
    /// Width measurement failed in the internal `anafanafo` compatibility layer.
    #[error("anafanafo measurement failed: {0}")]
    WidthComputation(String),
}

/// Renders an SVG badge from [`BadgeOptions`].
///
/// The returned string is a complete SVG document.
///
/// Input text is trimmed before layout, then XML-escaped during SVG emission.
/// This matches the behavior we care about from the upstream `badge-maker`
/// renderer without exposing its JavaScript-only validation layer.
///
/// Error behavior is intentionally narrow:
///
/// - Invalid `id_suffix` returns [`Error::InvalidIdSuffix`].
/// - Unknown colors do not error; they fall back to style defaults.
/// - Extra `links` entries are ignored rather than rejected.
pub fn make_badge(options: &BadgeOptions) -> Result<String, Error> {
    let id_suffix = options.id_suffix.clone().unwrap_or_default();
    if !id_suffix
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(Error::InvalidIdSuffix);
    }

    let label = options.label.trim().to_owned();
    let message = options.message.trim().to_owned();
    render::render_svg(options, label, message, id_suffix)
}
