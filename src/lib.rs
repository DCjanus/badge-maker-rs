//! Rust badge renderer targeting Shields-compatible badge output.
//!
//! Start with [`BadgeOptions::new`] and render with [`make_badge`].
//!
//! ```
//! use badge_maker_rs::{BadgeOptions, Color, Style, make_badge};
//!
//! let svg = make_badge(
//!     &BadgeOptions::new("passing")
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
mod css_color;
mod render;

pub use badge_color::{Color, NamedColor, ParseColorError};

use thiserror::Error;

/// Badge visual style.
#[doc = include_str!(concat!(env!("OUT_DIR"), "/style-previews.md"))]
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
/// Construct values with [`BadgeOptions::new`]. `message` is required at the
/// start function; the remaining fields are optional overrides.
///
/// `badge-maker` naming notes:
///
/// - `left_link` / `right_link` correspond to the two link slots encoded in the original `badge-maker` `links`
/// - `logo_data_url` corresponds to the original `badge-maker` field name `logoBase64`
/// - `logo_width` corresponds to the original `badge-maker` field name `logoWidth`
/// - `id_suffix` corresponds to the original `badge-maker` field name `idSuffix`
///
/// Link configuration is modeled as two explicit slots:
///
/// - `left_link` only: wraps the full badge body
/// - `right_link` only: links only the right half
/// - both present: links left and right halves independently
#[derive(Clone, Debug, Eq, PartialEq, Builder)]
#[builder(start_fn = new)]
pub struct BadgeOptions {
    /// Right-hand message text.
    #[builder(start_fn, into)]
    pub message: String,
    /// Left-hand label text. Empty by default.
    #[builder(default, into)]
    pub label: String,
    /// Badge visual style.
    #[builder(default = Style::Flat)]
    pub style: Style,
    /// Message-side color override.
    #[builder(into)]
    pub color: Option<Color>,
    /// Label-side color override.
    #[builder(into)]
    pub label_color: Option<Color>,
    /// Optional link for the left slot.
    ///
    /// This is the Rust-side left-slot projection of `badge-maker`'s `links`.
    ///
    /// When this is the only link provided, the full badge body is wrapped by
    /// one anchor to match `badge-maker` compatibility behavior.
    #[builder(into)]
    pub left_link: Option<String>,
    /// Optional link for the right slot.
    ///
    /// This is the Rust-side right-slot projection of `badge-maker`'s `links`.
    ///
    /// When only `right_link` is set, only the right half is linked.
    #[builder(into)]
    pub right_link: Option<String>,
    /// Optional logo data URL embedded as an SVG `<image>`.
    ///
    /// This corresponds to the original `badge-maker` field name `logoBase64`, but
    /// uses a Rust-side name that describes the actual expected value shape.
    #[builder(into)]
    pub logo_data_url: Option<String>,
    /// Explicit logo width in pixels.
    ///
    /// This corresponds to the original `badge-maker` field name `logoWidth`.
    ///
    /// It remains a Rust-side convenience override: only set it when the
    /// desired rendered logo width is already known.
    pub logo_width: Option<u32>,
    /// Optional suffix appended to generated SVG IDs to avoid collisions on the same page.
    ///
    /// This corresponds to the original `badge-maker` field name `idSuffix`.
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
/// pixel-level parity against `badge-maker` used as the main target.
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
