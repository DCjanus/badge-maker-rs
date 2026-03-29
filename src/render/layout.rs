use crate::Error;
use crate::anafanafo::Font as WidthFont;

use super::color::colors_for_background;
use super::xml::{Node, fragment};
use super::{
    FONT_FAMILY, FONT_SCALE_DOWN_VALUE, FONT_SCALE_UP_FACTOR, RenderParams, attr,
    create_accessible_text, element, logo_element, preferred_width_of, should_wrap_body_with_link,
    text,
};

#[derive(Clone, Debug)]
pub(super) struct BadgeLayout {
    pub(super) height: u32,
    pub(super) links: [String; 2],
    horiz_padding: u32,
    label_margin: u32,
    message_margin: i32,
    label_width: u32,
    message_width: u32,
    pub(super) left_width: u32,
    pub(super) right_width: u32,
    width: u32,
    label_color: String,
    color: String,
    label: String,
    message: String,
    pub(super) accessible_text: String,
    pub(super) id_suffix: String,
    logo_element: Option<Node>,
}

impl BadgeLayout {
    pub(super) fn new(params: &RenderParams, height: u32) -> Result<Self, Error> {
        let horiz_padding = 5_u32;
        let has_logo = params.logo.is_some();
        let total_logo_width = params.logo_width + params.logo_padding;
        let accessible_text = create_accessible_text(&params.label, &params.message);

        let has_label = !params.label.is_empty() || params.label_color.is_some();
        let mut label_color = params
            .label_color
            .clone()
            .unwrap_or_else(|| "#555".to_owned());
        if !(has_label || has_logo) {
            label_color = params.color.clone().unwrap_or_else(|| "#4c1".to_owned());
        }
        let color = params.color.clone().unwrap_or_else(|| "#4c1".to_owned());

        let label_margin = total_logo_width + 1;
        let label_width = if params.label.is_empty() {
            0
        } else {
            preferred_width_of(&params.label, WidthFont::Verdana11)?
        };
        let left_width = if has_label {
            label_width + 2 * horiz_padding + total_logo_width
        } else {
            0
        };

        let message_width = preferred_width_of(&params.message, WidthFont::Verdana11)?;
        let mut message_margin = left_width as i32 - if params.message.is_empty() { 0 } else { 1 };
        if !has_label {
            if has_logo {
                message_margin += (total_logo_width + horiz_padding) as i32;
            } else {
                message_margin += 1;
            }
        }
        let mut right_width = message_width + 2 * horiz_padding;
        if has_logo && !has_label {
            right_width += total_logo_width
                + if params.message.is_empty() {
                    0
                } else {
                    horiz_padding - 1
                };
        }

        let width = left_width + right_width;
        let logo_element = params
            .logo
            .as_ref()
            .map(|logo| logo_element(logo, horiz_padding, height, params.logo_width));

        Ok(Self {
            height,
            links: params.links.clone(),
            horiz_padding,
            label_margin,
            message_margin,
            label_width,
            message_width,
            left_width,
            right_width,
            width,
            label_color,
            color,
            label: params.label.clone(),
            message: params.message.clone(),
            accessible_text,
            id_suffix: params.id_suffix.clone(),
            logo_element,
        })
    }

    pub(super) fn clip_path(&self, radius: u32) -> Node {
        element(
            "clipPath",
            vec![attr("id", format!("r{}", self.id_suffix))],
            vec![element(
                "rect",
                vec![
                    attr("width", self.width),
                    attr("height", self.height),
                    attr("rx", radius),
                    attr("fill", "#fff"),
                ],
                vec![],
            )],
        )
    }

    pub(super) fn background_group(
        &self,
        with_gradient: bool,
        shape_rendering: Option<&str>,
    ) -> Node {
        let mut attrs = Vec::new();
        if with_gradient {
            attrs.push(attr("clip-path", format!("url(#r{})", self.id_suffix)));
        } else if let Some(shape_rendering) = shape_rendering {
            attrs.push(attr("shape-rendering", shape_rendering));
        }

        let mut content = vec![
            element(
                "rect",
                vec![
                    attr("width", self.left_width),
                    attr("height", self.height),
                    attr("fill", self.label_color.clone()),
                ],
                vec![],
            ),
            element(
                "rect",
                vec![
                    attr("x", self.left_width),
                    attr("width", self.right_width),
                    attr("height", self.height),
                    attr("fill", self.color.clone()),
                ],
                vec![],
            ),
        ];
        if with_gradient {
            content.push(element(
                "rect",
                vec![
                    attr("width", self.width),
                    attr("height", self.height),
                    attr("fill", format!("url(#s{})", self.id_suffix)),
                ],
                vec![],
            ));
        }
        element("g", attrs, content)
    }

    pub(super) fn foreground_group(&self, vertical_margin: i32, shadow: bool) -> Node {
        let mut content = Vec::new();
        if let Some(logo) = &self.logo_element {
            content.push(logo.clone());
        }
        if let Some(label) = self.text_element(
            self.label_margin as i32,
            &self.label,
            if !should_wrap_body_with_link(&self.links) {
                let left = self.links[0].as_str();
                if left.is_empty() { None } else { Some(left) }
            } else {
                None
            },
            &self.label_color,
            self.label_width,
            self.left_width,
            vertical_margin,
            shadow,
        ) {
            content.push(label);
        }
        if let Some(message) = self.text_element(
            self.message_margin,
            &self.message,
            {
                let right = self.links[1].as_str();
                if right.is_empty() { None } else { Some(right) }
            },
            &self.color,
            self.message_width,
            self.right_width,
            vertical_margin,
            shadow,
        ) {
            content.push(message);
        }

        element(
            "g",
            vec![
                attr("fill", "#fff"),
                attr("text-anchor", "middle"),
                attr("font-family", FONT_FAMILY),
                attr("text-rendering", "geometricPrecision"),
                attr("font-size", "110"),
            ],
            content,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn text_element(
        &self,
        left_margin: i32,
        content: &str,
        link: Option<&str>,
        color: &str,
        text_width: u32,
        link_width: u32,
        vertical_margin: i32,
        shadow: bool,
    ) -> Option<Node> {
        if content.is_empty() {
            return None;
        }
        let color_pair = colors_for_background(color);
        let x = f64::from(FONT_SCALE_UP_FACTOR)
            * (f64::from(left_margin)
                + 0.5 * f64::from(text_width)
                + f64::from(self.horiz_padding));

        let text_node = element(
            "text",
            vec![
                attr("x", x),
                attr("y", 140 + vertical_margin),
                attr("transform", FONT_SCALE_DOWN_VALUE),
                attr("fill", color_pair.text_color),
                attr("textLength", FONT_SCALE_UP_FACTOR * text_width as i32),
            ],
            vec![text(content)],
        );

        let mut nodes = Vec::new();
        if shadow {
            nodes.push(element(
                "text",
                vec![
                    attr("aria-hidden", "true"),
                    attr("x", x),
                    attr("y", 150 + vertical_margin),
                    attr("fill", color_pair.shadow_color),
                    attr("fill-opacity", ".3"),
                    attr("transform", FONT_SCALE_DOWN_VALUE),
                    attr("textLength", FONT_SCALE_UP_FACTOR * text_width as i32),
                ],
                vec![text(content)],
            ));
        }
        nodes.push(text_node);

        if let Some(link) = link {
            let rect = element(
                "rect",
                vec![
                    attr("width", link_width),
                    attr("x", if left_margin > 1 { left_margin + 1 } else { 0 }),
                    attr("height", self.height),
                    attr("fill", "rgba(0,0,0,0)"),
                ],
                vec![],
            );
            let mut wrapped_content = vec![rect];
            wrapped_content.extend(nodes);
            Some(element(
                "a",
                vec![attr("target", "_blank"), attr("href", link)],
                wrapped_content,
            ))
        } else if nodes.len() == 1 {
            nodes.into_iter().next()
        } else {
            Some(fragment(nodes))
        }
    }
}
