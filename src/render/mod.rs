// Internal SVG rendering implementation for the public crate-root API.
//
// Compatibility target:
// - Match the upstream rendered badge image as closely as possible.
// - Pixel parity in our raster comparison pipeline is more important than
//   byte-for-byte SVG source parity.
//
// Reference sources:
// - `badges/shields`
// - `badges/shields/badge-maker`
// - `badges/shields/badge-maker/lib/index.js`
// - `badges/shields/badge-maker/lib/make-badge.js`
// - `badges/shields/badge-maker/lib/badge-renderers.js`
// - `badges/shields/badge-maker/lib/color.js`
// - `badges/shields/badge-maker/lib/xml.js`

mod color;
mod for_the_badge;
mod layout;
mod social;
mod xml;

use crate::anafanafo::{self, Font as WidthFont, MeasureOptions};
use crate::{BadgeOptions, Error, Style};

use color::to_svg_color;
use for_the_badge::render_for_the_badge;
use layout::BadgeLayout;
use social::render_social;
use xml::{Node, attr, element, text};

const FONT_SCALE_UP_FACTOR: i32 = 10;
const FONT_SCALE_DOWN_VALUE: &str = "scale(.1)";
const FONT_FAMILY: &str = "Verdana,Geneva,DejaVu Sans,sans-serif";
const SOCIAL_FONT_FAMILY: &str = "Helvetica Neue,Helvetica,Arial,sans-serif";
const DEFAULT_LOGO_HEIGHT: u32 = 14;

pub(crate) fn render_svg(
    options: &BadgeOptions,
    label: String,
    message: String,
    id_suffix: String,
) -> Result<String, Error> {
    let logo = options
        .logo_data_url
        .as_ref()
        .filter(|logo| !logo.is_empty())
        .cloned();
    let has_logo = logo.is_some();
    let logo_width = if has_logo {
        options.logo_width.unwrap_or(DEFAULT_LOGO_HEIGHT)
    } else {
        0
    };
    let color = to_svg_color(options.color.as_ref());
    let label_color = to_svg_color(options.label_color.as_ref());
    let links = [
        options.left_link.clone().unwrap_or_default(),
        options.right_link.clone().unwrap_or_default(),
    ];

    let params = RenderParams {
        label,
        message,
        links,
        logo,
        logo_width,
        logo_padding: if has_logo && !options.label.trim().is_empty() {
            3
        } else {
            0
        },
        color,
        label_color,
        id_suffix,
    };

    match options.style {
        Style::Flat => Ok(Flat::render(&params)?),
        Style::FlatSquare => Ok(FlatSquare::render(&params)?),
        Style::Plastic => Ok(Plastic::render(&params)?),
        Style::ForTheBadge => Ok(render_for_the_badge(&params)?),
        Style::Social => Ok(render_social(&params)?),
    }
}

#[derive(Clone, Debug)]
struct RenderParams {
    label: String,
    message: String,
    links: [String; 2],
    logo: Option<String>,
    logo_width: u32,
    logo_padding: u32,
    color: Option<String>,
    label_color: Option<String>,
    id_suffix: String,
}

trait BadgeStyleImpl {
    const HEIGHT: u32;
    const VERTICAL_MARGIN: i32;
    const SHADOW: bool;
    const CLIP_RADIUS: Option<u32>;
    const INCLUDE_GRADIENT: bool;
    const SHAPE_RENDERING: Option<&'static str>;

    fn gradient(id_suffix: &str) -> Option<Node>;

    fn render(params: &RenderParams) -> Result<String, Error> {
        let badge = BadgeLayout::new(params, Self::HEIGHT)?;
        let mut content = Vec::new();

        if let Some(gradient) = Self::gradient(&badge.id_suffix) {
            content.push(gradient);
        }
        if let Some(radius) = Self::CLIP_RADIUS {
            content.push(badge.clip_path(radius));
        }
        content.push(badge.background_group(Self::INCLUDE_GRADIENT, Self::SHAPE_RENDERING));
        content.push(badge.foreground_group(Self::VERTICAL_MARGIN, Self::SHADOW));

        Ok(render_badge(
            &badge.links,
            f64::from(badge.left_width),
            f64::from(badge.right_width),
            Self::HEIGHT,
            &badge.accessible_text,
            content,
        ))
    }
}

struct Flat;
struct FlatSquare;
struct Plastic;

impl BadgeStyleImpl for Plastic {
    const HEIGHT: u32 = 18;
    const VERTICAL_MARGIN: i32 = -10;
    const SHADOW: bool = true;
    const CLIP_RADIUS: Option<u32> = Some(4);
    const INCLUDE_GRADIENT: bool = true;
    const SHAPE_RENDERING: Option<&'static str> = None;

    fn gradient(id_suffix: &str) -> Option<Node> {
        Some(element(
            "linearGradient",
            vec![
                attr("id", format!("s{id_suffix}")),
                attr("x2", "0"),
                attr("y2", "100%"),
            ],
            vec![
                element(
                    "stop",
                    vec![
                        attr("offset", "0"),
                        attr("stop-color", "#fff"),
                        attr("stop-opacity", ".7"),
                    ],
                    vec![],
                ),
                element(
                    "stop",
                    vec![
                        attr("offset", ".1"),
                        attr("stop-color", "#aaa"),
                        attr("stop-opacity", ".1"),
                    ],
                    vec![],
                ),
                element(
                    "stop",
                    vec![
                        attr("offset", ".9"),
                        attr("stop-color", "#000"),
                        attr("stop-opacity", ".3"),
                    ],
                    vec![],
                ),
                element(
                    "stop",
                    vec![
                        attr("offset", "1"),
                        attr("stop-color", "#000"),
                        attr("stop-opacity", ".5"),
                    ],
                    vec![],
                ),
            ],
        ))
    }
}

impl BadgeStyleImpl for Flat {
    const HEIGHT: u32 = 20;
    const VERTICAL_MARGIN: i32 = 0;
    const SHADOW: bool = true;
    const CLIP_RADIUS: Option<u32> = Some(3);
    const INCLUDE_GRADIENT: bool = true;
    const SHAPE_RENDERING: Option<&'static str> = None;

    fn gradient(id_suffix: &str) -> Option<Node> {
        Some(element(
            "linearGradient",
            vec![
                attr("id", format!("s{id_suffix}")),
                attr("x2", "0"),
                attr("y2", "100%"),
            ],
            vec![
                element(
                    "stop",
                    vec![
                        attr("offset", "0"),
                        attr("stop-color", "#bbb"),
                        attr("stop-opacity", ".1"),
                    ],
                    vec![],
                ),
                element(
                    "stop",
                    vec![attr("offset", "1"), attr("stop-opacity", ".1")],
                    vec![],
                ),
            ],
        ))
    }
}

impl BadgeStyleImpl for FlatSquare {
    const HEIGHT: u32 = 20;
    const VERTICAL_MARGIN: i32 = 0;
    const SHADOW: bool = false;
    const CLIP_RADIUS: Option<u32> = None;
    const INCLUDE_GRADIENT: bool = false;
    const SHAPE_RENDERING: Option<&'static str> = Some("crispEdges");

    fn gradient(_id_suffix: &str) -> Option<Node> {
        None
    }
}

fn render_badge(
    links: &[String; 2],
    left_width: f64,
    right_width: f64,
    height: u32,
    accessible_text: &str,
    content: Vec<Node>,
) -> String {
    let width = left_width + right_width;
    let left_link = links[0].as_str();
    let has_link = has_links(links).0;

    let mut body_content = content;
    if should_wrap_body_with_link(links) {
        body_content = vec![element(
            "a",
            vec![attr("target", "_blank"), attr("href", left_link)],
            body_content,
        )];
    }

    let mut svg_content = Vec::new();
    if !has_link {
        svg_content.push(element("title", vec![], vec![text(accessible_text)]));
    }
    svg_content.extend(body_content);

    let mut svg_attrs = vec![
        attr("xmlns", "http://www.w3.org/2000/svg"),
        attr("width", width),
        attr("height", height),
    ];
    if !has_link {
        svg_attrs.push(attr("role", "img"));
        svg_attrs.push(attr("aria-label", accessible_text));
    }

    element("svg", svg_attrs, svg_content).render()
}

fn has_links(links: &[String; 2]) -> (bool, bool, bool) {
    let has_left = !links[0].is_empty();
    let has_right = !links[1].is_empty();
    (has_left && has_right, has_left, has_right)
}

fn should_wrap_body_with_link(links: &[String; 2]) -> bool {
    let (_, has_left, has_right) = has_links(links);
    has_left && !has_right
}

fn create_accessible_text(label: &str, message: &str) -> String {
    if label.is_empty() {
        message.to_owned()
    } else {
        format!("{label}: {message}")
    }
}

fn js_utf16_len(input: &str) -> usize {
    input.encode_utf16().count()
}

fn capitalize(input: &str) -> String {
    let mut chars = input.chars();
    match chars.next() {
        // badge-maker uses `charAt(0).toUpperCase() + slice(1)`, which operates
        // on UTF-16 code units. When the first scalar spans two code units, the
        // JavaScript path leaves the string unchanged.
        Some(first) if first.len_utf16() > 1 => input.to_owned(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn logo_element(logo: &str, horiz_padding: u32, badge_height: u32, logo_width: u32) -> Node {
    let logo_height = 14_u32;
    let y = 0.5_f64 * f64::from(badge_height - logo_height);
    element(
        "image",
        vec![
            attr("x", horiz_padding),
            attr("y", y),
            attr("width", logo_width),
            attr("height", logo_height),
            attr("href", logo),
        ],
        vec![],
    )
}

fn preferred_width_of(text: &str, font: WidthFont) -> Result<u32, Error> {
    let measured = anafanafo::measure(text, MeasureOptions::new(font))
        .map_err(|error| Error::WidthComputation(error.to_string()))?;
    Ok(round_up_to_odd(measured.floor() as u32))
}

fn measured_width_floor(text: &str, font: WidthFont) -> Result<f64, Error> {
    let measured = anafanafo::measure(text, MeasureOptions::new(font))
        .map_err(|error| Error::WidthComputation(error.to_string()))?;
    Ok(f64::from(measured.floor()))
}

fn round_up_to_odd(value: u32) -> u32 {
    if value % 2 == 0 { value + 1 } else { value }
}
