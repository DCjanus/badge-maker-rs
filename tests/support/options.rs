use badge_maker_rs::{BadgeOptions, Color, Style};

use super::runner_core::ReferenceCase;

fn parse_style(style: Option<&str>) -> Style {
    match style.unwrap_or("flat") {
        "plastic" => Style::Plastic,
        "flat" => Style::Flat,
        "flat-square" => Style::FlatSquare,
        "for-the-badge" => Style::ForTheBadge,
        "social" => Style::Social,
        other => panic!("unsupported style {other}"),
    }
}

pub fn to_badge_options(case: &ReferenceCase) -> BadgeOptions {
    let mut options = BadgeOptions::new(case.message.clone())
        .label(case.label.clone().unwrap_or_default())
        .style(parse_style(case.style.as_deref()))
        .build();

    options.logo_data_url = case.logo_data_url.clone();
    options.logo_width = case.logo_width;
    options.id_suffix = case.id_suffix.clone();

    if let Some(color) = &case.color {
        options.color = Some(Color::literal(color.clone()));
    }
    if let Some(label_color) = &case.label_color {
        options.label_color = Some(Color::literal(label_color.clone()));
    }

    match case.links.as_slice() {
        [] => {}
        [left] => options.left_link = Some(left.clone()),
        [left, right] if left.is_empty() => options.right_link = Some(right.clone()),
        [left, right, ..] => {
            options.left_link = Some(left.clone());
            options.right_link = Some(right.clone());
        }
    }

    options
}
