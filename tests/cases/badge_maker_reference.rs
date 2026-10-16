use crate::support::{
    ReferenceResult, load_cases, render_svg_to_rgba, run_reference_batch, to_badge_options,
};

#[test]
fn pixel_comparison_detects_text() {
    let blank = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="20"/>"#;
    for family in [
        "Verdana,Geneva,DejaVu Sans,sans-serif",
        "Helvetica Neue,Helvetica,Arial,sans-serif",
    ] {
        for weight in ["normal", "bold"] {
            let with_text = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="20"><text x="2" y="15" font-family="{family}" font-weight="{weight}" font-size="11" fill="#fff">passing</text></svg>"##
            );
            assert_ne!(render_svg_to_rgba(&with_text), render_svg_to_rgba(blank));
        }
    }
}

#[test]
fn pixel_comparison_detects_different_dimensions_with_equal_pixel_counts() {
    let wide = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="20"/>"#;
    let tall = r#"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="25"/>"#;
    assert_ne!(render_svg_to_rgba(wide), render_svg_to_rgba(tall));
}

#[test]
fn reference_logo_fixtures_render_visible_pixels() {
    for case in load_cases() {
        if case.id.ends_with("-malformed-logo") {
            continue;
        }
        let Some(logo) = case
            .logo_data_url
            .as_deref()
            .filter(|logo| !logo.is_empty())
        else {
            continue;
        };
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><image width="16" height="16" href="{logo}"/></svg>"#
        );
        let image = render_svg_to_rgba(&svg);
        assert!(
            image
                .pixels
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] != 0),
            "logo fixture must render visible pixels for `{}`",
            case.id
        );
    }
}

#[test]
fn badge_maker_seed_cases_are_not_empty() {
    assert!(!load_cases().is_empty());
}

#[test]
fn badge_maker_case_ids_are_unique() {
    use std::collections::BTreeSet;

    let cases = load_cases();
    let mut seen = BTreeSet::new();
    for case in cases {
        assert!(
            seen.insert(case.id.clone()),
            "duplicate badge-maker case id: {}",
            case.id
        );
    }
}

#[test]
fn badge_maker_matches_reference_cases() {
    let cases = load_cases();
    let reference_results = run_reference_batch(&cases);
    assert_eq!(cases.len(), reference_results.len());

    for (case, reference) in cases.iter().zip(reference_results.iter()) {
        let actual = badge_maker_rs::make_badge(&to_badge_options(case));
        match (actual, reference) {
            (
                Ok(actual_output),
                ReferenceResult {
                    id,
                    ok: true,
                    output: Some(expected_output),
                    error: None,
                },
            ) => {
                assert_eq!(id, &case.id);
                let actual_pixels = render_svg_to_rgba(&actual_output);
                let expected_pixels = render_svg_to_rgba(expected_output);
                assert_eq!(
                    (actual_pixels.width, actual_pixels.height),
                    (expected_pixels.width, expected_pixels.height),
                    "rasterized dimensions mismatch for case `{}`",
                    case.id
                );
                assert!(
                    actual_pixels.pixels == expected_pixels.pixels,
                    "rasterized pixel mismatch for case `{}` ({} differing pixels)",
                    case.id,
                    actual_pixels
                        .pixels
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .zip(expected_pixels.pixels.as_chunks::<4>().0.iter())
                        .filter(|(actual, expected)| actual != expected)
                        .count()
                );
            }
            (
                Err(error),
                ReferenceResult {
                    id,
                    ok: false,
                    output: None,
                    error: Some(expected_error),
                },
            ) => {
                assert_eq!(id, &case.id);
                assert_eq!(error.to_string(), *expected_error);
            }
            (_, other) => panic!("unexpected result for case `{}`: {other:?}", case.id),
        }
    }
}
