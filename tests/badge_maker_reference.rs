use badge_maker_rs;

mod support;

use support::{
    ReferenceResult, load_cases, render_svg_to_rgba, run_reference_batch, to_badge_options,
};

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
                    actual_pixels, expected_pixels,
                    "rasterized pixel mismatch for case `{}`",
                    case.id
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
