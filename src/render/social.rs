use crate::Error;
use crate::anafanafo::Font as WidthFont;

use super::xml::fragment;
use super::{
    FONT_SCALE_DOWN_VALUE, FONT_SCALE_UP_FACTOR, RenderParams, SOCIAL_FONT_FAMILY, attr,
    capitalize, create_accessible_text, element, has_links, logo_element, preferred_width_of,
    render_badge, should_wrap_body_with_link, text,
};

pub(super) fn render_social(params: &RenderParams) -> Result<String, Error> {
    let label = capitalize(&params.label);
    let message = params.message.clone();
    let external_height = 20_u32;
    let internal_height = 19_u32;
    let label_horiz_padding = 5_u32;
    let message_horiz_padding = 4_u32;
    let horiz_gutter = 6_u32;
    let total_logo_width = params.logo_width + params.logo_padding;
    let has_message = !message.is_empty();

    let label_text_width = preferred_width_of(&label, WidthFont::Helvetica11Bold)?;
    let message_text_width = preferred_width_of(&message, WidthFont::Helvetica11Bold)?;
    let label_rect_width = label_text_width + total_logo_width + 2 * label_horiz_padding;
    let message_rect_width = message_text_width + 2 * message_horiz_padding;

    let [left_link, right_link] = &params.links;
    let (has_link, has_left_link, has_right_link) = has_links(&params.links);
    let accessible_text = create_accessible_text(&label, &message);

    let mut content = vec![
        element(
            "style",
            vec![],
            vec![text(format!(
                "a:hover #llink{}{{fill:url(#b{});stroke:#ccc}}a:hover #rlink{}{{fill:#4183c4}}",
                params.id_suffix, params.id_suffix, params.id_suffix
            ))],
        ),
        fragment(vec![
            element(
                "linearGradient",
                vec![
                    attr("id", format!("a{}", params.id_suffix)),
                    attr("x2", "0"),
                    attr("y2", "100%"),
                ],
                vec![
                    element(
                        "stop",
                        vec![
                            attr("offset", 0),
                            attr("stop-color", "#fcfcfc"),
                            attr("stop-opacity", 0),
                        ],
                        vec![],
                    ),
                    element(
                        "stop",
                        vec![attr("offset", 1), attr("stop-opacity", ".1")],
                        vec![],
                    ),
                ],
            ),
            element(
                "linearGradient",
                vec![
                    attr("id", format!("b{}", params.id_suffix)),
                    attr("x2", "0"),
                    attr("y2", "100%"),
                ],
                vec![
                    element(
                        "stop",
                        vec![
                            attr("offset", 0),
                            attr("stop-color", "#ccc"),
                            attr("stop-opacity", ".1"),
                        ],
                        vec![],
                    ),
                    element(
                        "stop",
                        vec![attr("offset", 1), attr("stop-opacity", ".1")],
                        vec![],
                    ),
                ],
            ),
        ]),
    ];

    let mut background_group_content = vec![element(
        "rect",
        vec![
            attr("stroke", "none"),
            attr("fill", "#fcfcfc"),
            attr("x", 0.5),
            attr("y", 0.5),
            attr("width", label_rect_width),
            attr("height", internal_height),
            attr("rx", 2),
        ],
        vec![],
    )];
    if has_message {
        let bubble_main_x = f64::from(label_rect_width + horiz_gutter) + 0.5;
        let bubble_notch_x = label_rect_width + horiz_gutter;
        background_group_content.extend([
            element(
                "rect",
                vec![
                    attr("x", bubble_main_x),
                    attr("y", 0.5),
                    attr("width", message_rect_width),
                    attr("height", internal_height),
                    attr("rx", 2),
                    attr("fill", "#fafafa"),
                ],
                vec![],
            ),
            element(
                "rect",
                vec![
                    attr("x", bubble_notch_x),
                    attr("y", 7.5),
                    attr("width", 0.5),
                    attr("height", 5),
                    attr("stroke", "#fafafa"),
                ],
                vec![],
            ),
            element(
                "path",
                vec![
                    attr("d", format!("M{} 6.5 l-3 3v1 l3 3", bubble_main_x)),
                    attr("fill", "#fafafa"),
                ],
                vec![],
            ),
        ]);
    }

    let label_text_x = f64::from(FONT_SCALE_UP_FACTOR)
        * (f64::from(total_logo_width)
            + f64::from(label_text_width) / 2.0
            + f64::from(label_horiz_padding));
    let label_text_length = FONT_SCALE_UP_FACTOR * label_text_width as i32;
    let label_shadow = element(
        "text",
        vec![
            attr("aria-hidden", "true"),
            attr("x", label_text_x),
            attr("y", 150),
            attr("fill", "#fff"),
            attr("transform", FONT_SCALE_DOWN_VALUE),
            attr("textLength", label_text_length),
        ],
        vec![text(&label)],
    );
    let label_text = element(
        "text",
        vec![
            attr("x", label_text_x),
            attr("y", 140),
            attr("transform", FONT_SCALE_DOWN_VALUE),
            attr("textLength", label_text_length),
        ],
        vec![text(&label)],
    );
    let label_rect = element(
        "rect",
        vec![
            attr("id", format!("llink{}", params.id_suffix)),
            attr("stroke", "#d5d5d5"),
            attr("fill", format!("url(#a{})", params.id_suffix)),
            attr("x", ".5"),
            attr("y", ".5"),
            attr("width", label_rect_width),
            attr("height", internal_height),
            attr("rx", 2),
        ],
        vec![],
    );

    let label_group = if has_left_link && !should_wrap_body_with_link(&params.links) {
        element(
            "a",
            vec![attr("target", "_blank"), attr("href", left_link)],
            vec![label_shadow, label_text, label_rect],
        )
    } else {
        fragment(vec![label_rect, label_shadow, label_text])
    };

    let message_group = if has_message {
        let message_text_x = f64::from(FONT_SCALE_UP_FACTOR)
            * (f64::from(label_rect_width + horiz_gutter) + f64::from(message_rect_width) / 2.0);
        let message_text_length = FONT_SCALE_UP_FACTOR * message_text_width as i32;
        let message_shadow = element(
            "text",
            vec![
                attr("aria-hidden", "true"),
                attr("x", message_text_x),
                attr("y", 150),
                attr("fill", "#fff"),
                attr("transform", FONT_SCALE_DOWN_VALUE),
                attr("textLength", message_text_length),
            ],
            vec![text(&message)],
        );
        let message_text = element(
            "text",
            vec![
                attr("id", format!("rlink{}", params.id_suffix)),
                attr("x", message_text_x),
                attr("y", 140),
                attr("transform", FONT_SCALE_DOWN_VALUE),
                attr("textLength", message_text_length),
            ],
            vec![text(&message)],
        );

        if has_right_link {
            element(
                "a",
                vec![attr("target", "_blank"), attr("href", right_link)],
                vec![
                    element(
                        "rect",
                        vec![
                            attr("width", message_rect_width + 1),
                            attr("x", label_rect_width + horiz_gutter),
                            attr("height", internal_height + 1),
                            attr("fill", "rgba(0,0,0,0)"),
                        ],
                        vec![],
                    ),
                    message_shadow,
                    message_text,
                ],
            )
        } else {
            fragment(vec![message_shadow, message_text])
        }
    } else {
        fragment(vec![])
    };

    content.push(element(
        "g",
        vec![attr("stroke", "#d5d5d5")],
        background_group_content,
    ));

    if let Some(logo) = &params.logo {
        content.push(logo_element(
            logo,
            label_horiz_padding,
            external_height,
            params.logo_width,
        ));
    }

    let foreground_attrs = vec![
        attr("aria-hidden", (!has_link).to_string()),
        attr("fill", "#333"),
        attr("text-anchor", "middle"),
        attr("font-family", SOCIAL_FONT_FAMILY),
        attr("text-rendering", "geometricPrecision"),
        attr("font-weight", 700),
        attr("font-size", "110px"),
        attr("line-height", "14px"),
    ];
    content.push(element(
        "g",
        foreground_attrs,
        vec![label_group, message_group],
    ));

    Ok(render_badge(
        &params.links,
        f64::from(label_rect_width + 1),
        if has_message {
            f64::from(horiz_gutter + message_rect_width)
        } else {
            0.0
        },
        external_height,
        &accessible_text,
        content,
    ))
}
