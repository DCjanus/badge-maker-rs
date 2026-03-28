use badge_maker_rs::make_badge;

#[allow(dead_code)]
mod support;

use support::{
    ReferenceCase, ReferenceResult, render_svg_to_rgba, run_reference_batch, to_badge_options,
};

#[test]
fn for_the_badge_unicode_spacing_matches_reference_pixels() {
    let case = ReferenceCase {
        id: "for-the-badge-unicode-spacing".to_owned(),
        label: Some("éé".to_owned()),
        message: "中".to_owned(),
        color: Some("#4c1".to_owned()),
        label_color: Some("#555".to_owned()),
        style: Some("for-the-badge".to_owned()),
        logo_data_url: None,
        logo_width: None,
        links: vec![],
        id_suffix: None,
    };

    let reference_results = run_reference_batch(std::slice::from_ref(&case));
    let reference_svg = match &reference_results[0] {
        ReferenceResult {
            id,
            ok: true,
            output: Some(output),
            error: None,
        } => {
            assert_eq!(id, "for-the-badge-unicode-spacing");
            output
        }
        other => panic!("unexpected reference result: {other:?}"),
    };

    let actual_svg =
        make_badge(&to_badge_options(&case)).expect("Rust renderer should render unicode badge");

    assert_eq!(
        render_svg_to_rgba(&actual_svg),
        render_svg_to_rgba(reference_svg),
        "unicode for-the-badge pixels diverged from badge-maker"
    );
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

    let reference_results = run_reference_batch(std::slice::from_ref(&case));
    let reference_svg = match &reference_results[0] {
        ReferenceResult {
            id,
            ok: true,
            output: Some(output),
            error: None,
        } => {
            assert_eq!(id, "social-astral-capitalize");
            output
        }
        other => panic!("unexpected reference result: {other:?}"),
    };

    let actual_svg = make_badge(&to_badge_options(&case))
        .expect("Rust renderer should render social astral label");

    assert_eq!(
        actual_svg, *reference_svg,
        "social capitalization diverged from badge-maker"
    );
}
