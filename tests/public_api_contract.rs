use badge_maker_rs::{BadgeOptions, Color, Error, NamedColor, Style, make_badge};

#[test]
fn escapes_text_input_for_svg_and_accessibility() {
    let options = BadgeOptions::new("<passing> & ready")
        .label("build \"ci\"")
        .build();

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.contains("aria-label=\"build &quot;ci&quot;: &lt;passing&gt; &amp; ready\""));
    assert!(svg.contains(">build &quot;ci&quot;<"));
    assert!(svg.contains(">&lt;passing&gt; &amp; ready<"));
}

#[test]
fn invalid_id_suffix_is_rejected() {
    let options = BadgeOptions::new("passing")
        .label("build")
        .id_suffix("\\")
        .build();

    let error = make_badge(&options).expect_err("expected invalid id suffix to fail");
    assert_eq!(error, Error::InvalidIdSuffix);
}

#[test]
fn left_and_right_links_render_in_distinct_slots() {
    let options = BadgeOptions::new("passing")
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
fn logo_width_is_a_rust_side_override() {
    let base = BadgeOptions::new("passing")
        .label("build")
        .color(NamedColor::Brightgreen)
        .style(Style::Flat)
        .logo_data_url("data:image/svg+xml;base64,PHN2ZyB4bWxu")
        .build();

    let wide_logo = BadgeOptions::new("passing")
        .label("build")
        .color(NamedColor::Brightgreen)
        .style(Style::Flat)
        .logo_data_url("data:image/svg+xml;base64,PHN2ZyB4bWxu")
        .logo_width(28)
        .build();

    let base_svg = make_badge(&base).expect("badge render should succeed");
    let wide_svg = make_badge(&wide_logo).expect("badge render should succeed");

    assert_ne!(base_svg, wide_svg);
    assert!(wide_svg.contains("width=\"28\""));
}

#[test]
fn logo_width_without_logo_does_not_affect_output() {
    let base = BadgeOptions::new("passing")
        .label("build")
        .color(NamedColor::Brightgreen)
        .style(Style::Flat)
        .build();

    let ignored_logo_width = BadgeOptions::new("passing")
        .label("build")
        .color(NamedColor::Brightgreen)
        .style(Style::Flat)
        .logo_width(28)
        .build();

    let base_svg = make_badge(&base).expect("badge render should succeed");
    let ignored_svg = make_badge(&ignored_logo_width).expect("badge render should succeed");

    assert_eq!(ignored_svg, base_svg);
    assert!(!ignored_svg.contains("<image "));
}

#[test]
fn right_link_preserves_right_only_semantics() {
    let options = BadgeOptions::new("passing")
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
    let options = BadgeOptions::new("passing")
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
fn builder_applies_default_accessible_text() {
    let svg = make_badge(&BadgeOptions::new("passing").label("build").build())
        .expect("badge render should succeed");

    assert!(svg.contains("aria-label=\"build: passing\""));
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

    assert_eq!(named, Color::from(NamedColor::Success));
    assert!(matches!(hex, Color::Hex(value) if value == "#4c1"));
    assert!(matches!(css, Color::Css(value) if value == "papayawhip"));
    assert!("--badge-color".parse::<Color>().is_err());
    assert!("var(--badge-color)".parse::<Color>().is_err());
    assert!("definitely-not-a-color".parse::<Color>().is_err());
}

#[test]
fn invalid_color_parse_error_is_actionable() {
    let error = "definitely-not-a-color"
        .parse::<Color>()
        .expect_err("invalid color should fail parsing");

    assert_eq!(
        error.to_string(),
        "invalid badge color: expected a named color, #rgb/#rrggbb, or a CSS color"
    );
}
