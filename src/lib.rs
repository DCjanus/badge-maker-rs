//! Rust badge renderer targeting Shields-compatible badge output.
//!
//! Start with [`BadgeOptions::builder`] and render with [`make_badge`].
//!
//! ```
//! use badge_maker_rs::{BadgeOptions, Color, Style, make_badge};
//!
//! let svg = make_badge(
//!     &BadgeOptions::builder()
//!         .message("passing")
//!         .label("build")
//!         .color("brightgreen".parse::<Color>().unwrap())
//!         .style(Style::Flat)
//!         .build(),
//! )
//! .unwrap();
//!
//! assert!(svg.starts_with("<svg "));
//! assert!(svg.contains("aria-label=\"build: passing\""));
//! ```
//!
//! The public API is intentionally Rust-first. The compatibility target is the
//! final rendered badge image: pixel-equivalent output matters more than SVG
//! source parity, and JavaScript-specific API shape is out of scope.

use bon::Builder;
mod anafanafo;
mod badge_color;
mod render;

pub use badge_color::{Color, NamedColor, ParseColorError};

use thiserror::Error;

/// Badge visual style.
#[doc = include_str!("../docs/style-previews.md")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Style {
    Plastic,
    Flat,
    FlatSquare,
    ForTheBadge,
    Social,
}

/// Badge rendering options.
///
/// Construct values with [`BadgeOptions::builder`]. `message` is required; the
/// remaining fields are optional overrides.
///
/// Link configuration is modeled as two explicit slots:
///
/// - `left_link` only: wraps the full badge body
/// - `right_link` only: links only the right half
/// - both present: links left and right halves independently
#[derive(Clone, Debug, Eq, PartialEq, Builder)]
pub struct BadgeOptions {
    /// Left-hand label text. Empty by default.
    #[builder(default, into)]
    pub label: String,
    /// Right-hand message text.
    #[builder(into)]
    pub message: String,
    /// Message-side color override.
    #[builder(into)]
    pub color: Option<Color>,
    /// Label-side color override.
    #[builder(into)]
    pub label_color: Option<Color>,
    /// Badge visual style.
    #[builder(default = Style::Flat)]
    pub style: Style,
    /// Optional logo data URL embedded as an SVG `<image>`.
    #[builder(into)]
    pub logo_base64: Option<String>,
    /// Explicit logo width in pixels.
    ///
    /// This is a Rust-side convenience override, not an upstream JavaScript API.
    pub logo_width: Option<u32>,
    /// Optional link for the left slot.
    ///
    /// When this is the only link provided, the full badge body is wrapped by
    /// one anchor to match upstream behavior.
    #[builder(into)]
    pub left_link: Option<String>,
    /// Optional link for the right slot.
    ///
    /// When only `right_link` is set, only the right half is linked.
    #[builder(into)]
    pub right_link: Option<String>,
    /// Optional suffix appended to generated SVG IDs to avoid collisions on the same page.
    #[builder(into)]
    pub id_suffix: Option<String>,
}

/// Rendering errors returned by [`make_badge`].
#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum Error {
    /// `id_suffix` contained characters outside `[A-Za-z0-9_-]`.
    #[error("Field `idSuffix` must contain only numbers, letters, -, and _")]
    InvalidIdSuffix,
    /// Width measurement failed in the internal `anafanafo` compatibility layer.
    #[error("anafanafo measurement failed: {0}")]
    WidthComputation(String),
}

/// Renders a complete SVG badge document from [`BadgeOptions`].
///
/// Invalid `id_suffix` returns [`Error::InvalidIdSuffix`]. Invalid
/// [`Color::Literal`] values fall back to defaults instead of erroring.
///
/// The returned SVG is an intermediate representation. The primary
/// compatibility goal of this crate is the final rendered image, with
/// pixel-level parity against upstream used as the main target.
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
