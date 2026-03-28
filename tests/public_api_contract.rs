use badge_maker_rs::{BadgeOptions, Color, Error, NamedColor, Style, make_badge};

#[test]
fn trims_and_escapes_text_input() {
    let options = BadgeOptions::builder()
        .message("  <passing> & ready  ")
        .label("  build \"ci\"  ")
        .build();

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.contains("aria-label=\"build &quot;ci&quot;: &lt;passing&gt; &amp; ready\""));
    assert!(svg.contains(">build &quot;ci&quot;<"));
    assert!(svg.contains(">&lt;passing&gt; &amp; ready<"));
}

#[test]
fn invalid_id_suffix_is_rejected() {
    let options = BadgeOptions::builder()
        .message("passing")
        .label("build")
        .id_suffix("\\")
        .build();

    let error = make_badge(&options).expect_err("expected invalid id suffix to fail");
    assert_eq!(error, Error::InvalidIdSuffix);
}

#[test]
fn left_and_right_links_render_in_distinct_slots() {
    let options = BadgeOptions::builder()
        .message("passing")
        .label("build")
        .color(NamedColor::Brightgreen)
        .style(Style::Flat)
        .left_link("https://example.com/left")
        .right_link("https://example.com/right")
        .build();

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.contains("https://example.com/left"));
    assert!(svg.contains("https://example.com/right"));
}

#[test]
fn invalid_colors_fall_back_to_style_defaults() {
    let invalid = BadgeOptions::builder()
        .message("passing")
        .label("build")
        .color(Color::literal("definitely-not-a-color"))
        .label_color(Color::literal("still-not-a-color"))
        .style(Style::Flat)
        .build();

    let defaulted = BadgeOptions::builder()
        .message("passing")
        .label("build")
        .style(Style::Flat)
        .build();

    let invalid_svg = make_badge(&invalid).expect("badge render should succeed");
    let default_svg = make_badge(&defaulted).expect("badge render should succeed");

    assert_eq!(invalid_svg, default_svg);
}

#[test]
fn logo_width_is_a_rust_side_override() {
    let base = BadgeOptions::builder()
        .message("passing")
        .label("build")
        .color(NamedColor::Brightgreen)
        .style(Style::Flat)
        .logo_base64("data:image/svg+xml;base64,PHN2ZyB4bWxu")
        .build();

    let wide_logo = BadgeOptions::builder()
        .message("passing")
        .label("build")
        .color(NamedColor::Brightgreen)
        .style(Style::Flat)
        .logo_base64("data:image/svg+xml;base64,PHN2ZyB4bWxu")
        .logo_width(28)
        .build();

    let base_svg = make_badge(&base).expect("badge render should succeed");
    let wide_svg = make_badge(&wide_logo).expect("badge render should succeed");

    assert_ne!(base_svg, wide_svg);
    assert!(wide_svg.contains("width=\"28\""));
}

#[test]
fn empty_label_and_message_are_allowed() {
    let options = BadgeOptions::builder()
        .message("")
        .label("")
        .logo_base64("data:image/svg+xml;base64,PHN2ZyB4bWxu")
        .build();

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.starts_with("<svg "));
    assert!(svg.contains("<image "));
}

#[test]
fn right_link_preserves_right_only_semantics() {
    let options = BadgeOptions::builder()
        .message("passing")
        .label("build")
        .color(NamedColor::Brightgreen)
        .style(Style::Flat)
        .right_link("https://example.com/right-only")
        .build();

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.contains("href=\"https://example.com/right-only\""));
    assert!(svg.contains("<a target=\"_blank\" href=\"https://example.com/right-only\">"));
}

#[test]
fn left_link_wraps_the_full_badge_body() {
    let options = BadgeOptions::builder()
        .message("passing")
        .label("build")
        .color(NamedColor::Brightgreen)
        .style(Style::Flat)
        .left_link("https://example.com/replaced")
        .build();

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.contains("https://example.com/replaced"));
    assert!(svg.contains("<a target=\"_blank\" href=\"https://example.com/replaced\">"));
}

#[test]
fn semantic_color_aliases_render_successfully() {
    let aliases = [
        NamedColor::Success,
        NamedColor::Important,
        NamedColor::Critical,
        NamedColor::Informational,
        NamedColor::Inactive,
    ];

    for alias in aliases {
        let options = BadgeOptions::builder()
            .message("passing")
            .label("build")
            .color(alias)
            .style(Style::Flat)
            .build();

        let svg = make_badge(&options).expect("badge render should succeed");

        assert!(
            svg.starts_with("<svg "),
            "alias `{}` did not render",
            alias.as_str()
        );
    }
}

#[test]
fn builder_requires_message_and_applies_defaults() {
    let svg = make_badge(
        &BadgeOptions::builder()
            .message("passing")
            .label("build")
            .color(NamedColor::Brightgreen)
            .build(),
    )
    .expect("badge render should succeed");

    assert!(svg.contains("aria-label=\"build: passing\""));
    assert!(svg.contains("#4c1"));
}

#[test]
fn css_variable_colors_are_emitted_verbatim() {
    let options = BadgeOptions::builder()
        .message("token")
        .label("theme")
        .color(Color::css_variable("--badge-color"))
        .build();

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.contains("fill=\"var(--badge-color)\""));
}

#[test]
fn color_parsing_prefers_strict_typed_paths() {
    let named = "success"
        .parse::<Color>()
        .expect("named color should parse");
    let hex = "#4c1".parse::<Color>().expect("hex color should parse");
    let css = "papayawhip"
        .parse::<Color>()
        .expect("css color should parse");
    let var = "--badge-color"
        .parse::<Color>()
        .expect("css variable should parse");

    assert_eq!(named, Color::from(NamedColor::Success));
    assert!(matches!(hex, Color::Hex(value) if value == "#4c1"));
    assert!(matches!(css, Color::Css(value) if value == "papayawhip"));
    assert_eq!(var, Color::css_variable("--badge-color"));
    assert!("definitely-not-a-color".parse::<Color>().is_err());
}

#[test]
fn invalid_color_parse_error_is_actionable() {
    let error = "definitely-not-a-color"
        .parse::<Color>()
        .expect_err("invalid color should fail parsing");

    assert_eq!(
        error.to_string(),
        "invalid badge color: expected a named color, #rgb/#rrggbb, a CSS color, or a CSS variable"
    );
}
