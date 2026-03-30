use badge_maker_rs::make_badge;

#[path = "support/svg_case.rs"]
mod support;

use support::{ReferenceCase, reference_svg_for_case, to_badge_options};

#[test]
fn raw_css_variable_input_falls_back_like_badge_maker() {
    let case = ReferenceCase {
        id: "css-variable-fallback".to_owned(),
        label: Some("theme".to_owned()),
        message: "token".to_owned(),
        color: Some("var(--badge-color)".to_owned()),
        label_color: None,
        style: Some("flat".to_owned()),
        logo_data_url: None,
        logo_width: None,
        links: vec![],
        id_suffix: None,
    };

    let reference_svg = reference_svg_for_case(&case);
    let actual_svg = make_badge(&to_badge_options(&case))
        .expect("Rust renderer should match badge-maker fallback behavior");

    assert_eq!(actual_svg, reference_svg);
    assert!(!actual_svg.contains("var(--badge-color)"));
    assert!(actual_svg.contains("fill=\"#4c1\""));
}

#[test]
fn social_astral_capitalization_matches_reference_svg() {
    let case = ReferenceCase {
        id: "social-astral-capitalize".to_owned(),
        label: Some("𐐨badge".to_owned()),
        message: "ok".to_owned(),
        color: None,
        label_color: None,
        style: Some("social".to_owned()),
        logo_data_url: None,
        logo_width: None,
        links: vec![],
        id_suffix: None,
    };

    let reference_svg = reference_svg_for_case(&case);
    let actual_svg = make_badge(&to_badge_options(&case))
        .expect("Rust renderer should render social astral label");

    assert_eq!(
        actual_svg, reference_svg,
        "social capitalization diverged from badge-maker"
    );
}
