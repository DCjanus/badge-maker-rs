use badge_maker_rs::{BadgeOptions, make_badge};

#[path = "support/error_case.rs"]
mod support;

use support::{ReferenceCase, reference_error_for_case};

#[test]
fn invalid_id_suffix_matches_badge_maker_error() {
    let case = ReferenceCase {
        id: "invalid-id-suffix".to_owned(),
        label: Some("build".to_owned()),
        message: "passing".to_owned(),
        color: None,
        label_color: None,
        style: None,
        logo_data_url: None,
        logo_width: None,
        links: vec![],
        id_suffix: Some("\\".to_owned()),
    };

    let reference_error = reference_error_for_case(&case);
    let error = make_badge(
        &BadgeOptions::new("passing")
            .label("build")
            .id_suffix("\\")
            .build(),
    )
    .expect_err("expected invalid idSuffix");

    assert_eq!(error.to_string(), reference_error);
}
