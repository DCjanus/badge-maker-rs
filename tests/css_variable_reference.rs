use badge_maker_rs::{make_badge};

#[allow(dead_code)]
mod support;

use support::{ReferenceCase, ReferenceResult, run_reference_batch, to_badge_options};

#[test]
fn raw_css_variable_input_falls_back_like_badge_maker() {
    let reference_case = ReferenceCase {
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

    let reference_results = run_reference_batch(std::slice::from_ref(&reference_case));
    let reference_svg = match &reference_results[0] {
        ReferenceResult {
            id,
            ok: true,
            output: Some(output),
            error: None,
        } => {
            assert_eq!(id, "css-variable-fallback");
            output
        }
        other => panic!("unexpected reference result: {other:?}"),
    };

    let actual_svg = make_badge(&to_badge_options(&reference_case))
        .expect("Rust renderer should match badge-maker fallback behavior");

    assert_eq!(actual_svg, *reference_svg);
    assert!(!actual_svg.contains("var(--badge-color)"));
    assert!(actual_svg.contains("fill=\"#4c1\""));
}
