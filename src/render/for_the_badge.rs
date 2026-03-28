use crate::Error;
use crate::anafanafo::Font as WidthFont;

use super::color::colors_for_background;
use super::{
    FONT_FAMILY, FONT_SCALE_DOWN_VALUE, FONT_SCALE_UP_FACTOR, RenderParams, attr,
    create_accessible_text, element, has_links, js_utf16_len, logo_element,
    measured_width_floor, render_badge, should_wrap_body_with_link, text,
};

pub(super) fn render_for_the_badge(params: &RenderParams) -> Result<String, Error> {
    const FONT_SIZE: i32 = 10;
    const BADGE_HEIGHT: u32 = 28;
    const TEXT_MARGIN: f64 = 12.0;
    const LOGO_MARGIN: f64 = 9.0;
    const LOGO_TEXT_GUTTER: f64 = 6.0;
    const LETTER_SPACING: f64 = 1.25;

    let label = params.label.to_uppercase();
    let message = params.message.to_uppercase();
    let [left_link, right_link] = &params.links;
    let (_, has_left_link, has_right_link) = has_links(&params.links);

    let out_label_color = params
        .label_color
        .clone()
        .unwrap_or_else(|| "#555".to_owned());
    let color = params.color.clone().unwrap_or_else(|| "#4c1".to_owned());

    let label_text_width = if label.is_empty() {
        0.0
    } else {
        measured_width_floor(&label, WidthFont::Verdana10)?
            + LETTER_SPACING * js_utf16_len(&label) as f64
    };
    let message_text_width = if message.is_empty() {
        0.0
    } else {
        measured_width_floor(&message, WidthFont::Verdana10Bold)?
            + LETTER_SPACING * js_utf16_len(&message) as f64
    };

    let has_label = !label.is_empty();
    let no_text = !has_label && message.is_empty();
    let needs_label_rect = has_label || (params.logo.is_some() && params.label_color.is_some());
    let gutter = if no_text {
        LOGO_TEXT_GUTTER - LOGO_MARGIN
    } else {
        LOGO_TEXT_GUTTER
    };

    let (logo_min_x, label_text_min_x) = if params.logo.is_some() {
        let logo_min_x = LOGO_MARGIN;
        let label_text_min_x = logo_min_x + f64::from(params.logo_width) + gutter;
        (Some(logo_min_x), label_text_min_x)
    } else {
        (None, TEXT_MARGIN)
    };

    let (label_rect_width, message_text_min_x, message_rect_width) = if needs_label_rect {
        let label_rect_width = if has_label {
            label_text_min_x + label_text_width + TEXT_MARGIN
        } else {
            2.0 * LOGO_MARGIN + f64::from(params.logo_width)
        };
        let message_text_min_x = label_rect_width + TEXT_MARGIN;
        let message_rect_width = 2.0 * TEXT_MARGIN + message_text_width;
        (
            Some(label_rect_width),
            message_text_min_x,
            message_rect_width,
        )
    } else if params.logo.is_some() {
        let message_text_min_x = TEXT_MARGIN + f64::from(params.logo_width) + gutter;
        let message_rect_width =
            2.0 * TEXT_MARGIN + f64::from(params.logo_width) + gutter + message_text_width;
        (None, message_text_min_x, message_rect_width)
    } else {
        let message_text_min_x = TEXT_MARGIN;
        let message_rect_width = 2.0 * TEXT_MARGIN + message_text_width;
        (None, message_text_min_x, message_rect_width)
    };

    let mut foreground = Vec::new();
    if let Some(logo) = &params.logo {
        foreground.push(logo_element(
            logo,
            logo_min_x.unwrap_or(TEXT_MARGIN) as u32,
            BADGE_HEIGHT,
            params.logo_width,
        ));
    }

    if has_label {
        let color_pair = colors_for_background(&out_label_color);
        let mid_x = label_text_min_x + 0.5 * label_text_width;
        let label_text = element(
            "text",
            vec![
                attr("transform", FONT_SCALE_DOWN_VALUE),
                attr("x", FONT_SCALE_UP_FACTOR as f64 * mid_x),
                attr("y", 175),
                attr("textLength", FONT_SCALE_UP_FACTOR as f64 * label_text_width),
                attr("fill", color_pair.text_color),
            ],
            vec![text(&label)],
        );

        if has_left_link && !should_wrap_body_with_link(&params.links) {
            foreground.push(element(
                "a",
                vec![attr("target", "_blank"), attr("href", left_link)],
                vec![
                    element(
                        "rect",
                        vec![
                            attr(
                                "width",
                                label_rect_width.expect("label rect width should exist"),
                            ),
                            attr("height", BADGE_HEIGHT),
                            attr("fill", "rgba(0,0,0,0)"),
                        ],
                        vec![],
                    ),
                    label_text,
                ],
            ));
        } else {
            foreground.push(label_text);
        }
    }

    let color_pair = colors_for_background(&color);
    let message_mid_x = message_text_min_x + 0.5 * message_text_width;
    let message_text = element(
        "text",
        vec![
            attr("transform", FONT_SCALE_DOWN_VALUE),
            attr("x", FONT_SCALE_UP_FACTOR as f64 * message_mid_x),
            attr("y", 175),
            attr(
                "textLength",
                FONT_SCALE_UP_FACTOR as f64 * message_text_width,
            ),
            attr("fill", color_pair.text_color),
            attr("font-weight", "bold"),
        ],
        vec![text(&message)],
    );
    if has_right_link {
        foreground.push(element(
            "a",
            vec![attr("target", "_blank"), attr("href", right_link)],
            vec![
                element(
                    "rect",
                    vec![
                        attr("width", message_rect_width),
                        attr("height", BADGE_HEIGHT),
                        attr("x", label_rect_width.unwrap_or(0.0)),
                        attr("fill", "rgba(0,0,0,0)"),
                    ],
                    vec![],
                ),
                message_text,
            ],
        ));
    } else {
        foreground.push(message_text);
    }

    let background_content = if let Some(label_rect_width) = label_rect_width {
        vec![
            element(
                "rect",
                vec![
                    attr("width", label_rect_width),
                    attr("height", BADGE_HEIGHT),
                    attr("fill", out_label_color),
                ],
                vec![],
            ),
            element(
                "rect",
                vec![
                    attr("x", label_rect_width),
                    attr("width", message_rect_width),
                    attr("height", BADGE_HEIGHT),
                    attr("fill", color),
                ],
                vec![],
            ),
        ]
    } else {
        vec![element(
            "rect",
            vec![
                attr("width", message_rect_width),
                attr("height", BADGE_HEIGHT),
                attr("fill", color),
            ],
            vec![],
        )]
    };

    let content = vec![
        element(
            "g",
            vec![attr("shape-rendering", "crispEdges")],
            background_content,
        ),
        element(
            "g",
            vec![
                attr("fill", "#fff"),
                attr("text-anchor", "middle"),
                attr("font-family", FONT_FAMILY),
                attr("text-rendering", "geometricPrecision"),
                attr("font-size", FONT_SCALE_UP_FACTOR * FONT_SIZE),
            ],
            foreground,
        ),
    ];

    Ok(render_badge(
        &params.links,
        label_rect_width.unwrap_or(0.0),
        message_rect_width,
        BADGE_HEIGHT,
        &create_accessible_text(&label, &message),
        content,
    ))
}
