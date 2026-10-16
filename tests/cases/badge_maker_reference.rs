use std::collections::BTreeSet;

use crate::support::{
    ReferenceReport, ReferenceResult, group_filter_value, group_name, load_cases,
    render_svg_to_rgba, run_reference_batch, to_badge_options,
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
fn case_group_helpers_render_product_names() {
    assert_eq!(group_name(None), "Compatibility");
    assert_eq!(group_name(Some("deps-rs")), "deps.rs");
    assert_eq!(group_name(Some("future-group")), "future-group");
    assert_eq!(group_filter_value(None), "compatibility");
    assert_eq!(group_filter_value(Some("deps-rs")), "deps-rs");
}

#[test]
fn fixture_cases_declare_usable_groups() {
    let cases = load_cases();
    for case in &cases {
        if let Some(group) = &case.group {
            assert!(
                !group.trim().is_empty(),
                "case `{}` declares a blank group",
                case.id
            );
            assert!(
                !group_name(case.group.as_deref()).is_empty(),
                "case `{}` has no display name for its group",
                case.id
            );
        }
    }
    assert!(
        cases.iter().any(|case| case.group.is_none()),
        "the generic badge-maker fixture is the Compatibility group"
    );
}

#[test]
fn deps_rs_showcase_fixture_covers_typical_states() {
    let cases = load_cases();
    let grouped = cases
        .iter()
        .filter(|case| case.group.as_deref() == Some("deps-rs"))
        .collect::<Vec<_>>();
    assert!(
        !grouped.is_empty(),
        "tests/data/deps_rs_cases.json must load a deps.rs showcase"
    );
    assert!(
        grouped.len() <= 20,
        "the deps.rs group is a quick visual showcase, not a full matrix: {} cases",
        grouped.len()
    );
    for case in &grouped {
        assert!(
            case.id.starts_with("deps-rs-"),
            "deps.rs case ids carry the group prefix: {}",
            case.id
        );
        assert!(
            case.style.is_some(),
            "deps.rs cases name a style: {}",
            case.id
        );
    }

    // The showcase is scanned by eye, so it keeps one example per typical state
    // instead of expanding every state/style combination.
    let messages = grouped
        .iter()
        .map(|case| case.message.as_str())
        .collect::<BTreeSet<_>>();
    for state in [
        "unknown",
        "none",
        "up to date",
        "insecure",
        "maybe insecure",
        "3 of 18 outdated",
    ] {
        assert!(
            messages.contains(state),
            "deps.rs showcase covers the `{state}` state"
        );
    }

    let colors = grouped
        .iter()
        .filter_map(|case| case.color.as_deref())
        .collect::<BTreeSet<_>>();
    for color in ["#4c1", "#dfb317", "#e05d44"] {
        assert!(
            colors.contains(color),
            "deps.rs showcase covers the `{color}` status color"
        );
    }

    let flat_default = grouped
        .iter()
        .filter(|case| {
            case.style.as_deref() == Some("flat") && case.label.as_deref() == Some("dependencies")
        })
        .count();
    assert_eq!(flat_default, 6, "six default flat states");
    let compact = grouped
        .iter()
        .filter(|case| case.label.as_deref() == Some("deps"))
        .count();
    assert_eq!(compact, 2, "two compact `deps` subjects");
    let other_styles = grouped
        .iter()
        .filter(|case| case.style.as_deref() != Some("flat"))
        .count();
    assert_eq!(other_styles, 4, "one up-to-date badge per remaining style");

    let styles = grouped
        .iter()
        .filter_map(|case| case.style.as_deref())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        styles,
        BTreeSet::from(["flat", "flat-square", "plastic", "for-the-badge", "social"]),
        "the deps.rs fixture covers every badge style"
    );
}

#[test]
fn badge_maker_matches_reference_cases() {
    let cases = load_cases();
    let reference_results = run_reference_batch(&cases);
    assert_eq!(cases.len(), reference_results.len());
    let mut report = ReferenceReport::new();

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
                report.record_render(
                    case,
                    &actual_output,
                    expected_output,
                    &actual_pixels,
                    &expected_pixels,
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
                report.record_error(case, &error.to_string(), expected_error);
            }
            (_, other) => panic!("unexpected result for case `{}`: {other:?}", case.id),
        }
    }
    report.finish();
}
