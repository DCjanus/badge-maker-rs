use badge_maker_rs::{BadgeOptions, Error, Style, make_badge};

fn base_options() -> BadgeOptions {
    let mut options = BadgeOptions::new("passing");
    options.label = "build".to_owned();
    options.color = Some("brightgreen".to_owned());
    options.style = Style::Flat;
    options
}

#[test]
fn trims_and_escapes_text_input() {
    let mut options = BadgeOptions::new("  <passing> & ready  ");
    options.label = "  build \"ci\"  ".to_owned();

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.contains("aria-label=\"build &quot;ci&quot;: &lt;passing&gt; &amp; ready\""));
    assert!(svg.contains(">build &quot;ci&quot;<"));
    assert!(svg.contains(">&lt;passing&gt; &amp; ready<"));
}

#[test]
fn invalid_id_suffix_is_rejected() {
    let mut options = base_options();
    options.id_suffix = Some("\\".to_owned());

    let error = make_badge(&options).expect_err("expected invalid id suffix to fail");
    assert_eq!(error, Error::InvalidIdSuffix);
}

#[test]
fn only_first_two_links_are_used() {
    let mut options = base_options();
    options.links = vec![
        "https://example.com/left".to_owned(),
        "https://example.com/right".to_owned(),
        "https://example.com/ignored".to_owned(),
    ];

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.contains("https://example.com/left"));
    assert!(svg.contains("https://example.com/right"));
    assert!(!svg.contains("https://example.com/ignored"));
}

#[test]
fn invalid_colors_fall_back_to_style_defaults() {
    let mut invalid = base_options();
    invalid.color = Some("definitely-not-a-color".to_owned());
    invalid.label_color = Some("still-not-a-color".to_owned());

    let mut defaulted = base_options();
    defaulted.color = None;
    defaulted.label_color = None;

    let invalid_svg = make_badge(&invalid).expect("badge render should succeed");
    let default_svg = make_badge(&defaulted).expect("badge render should succeed");

    assert_eq!(invalid_svg, default_svg);
}

#[test]
fn logo_width_is_a_rust_side_override() {
    let mut base = base_options();
    base.logo_base64 = Some("data:image/svg+xml;base64,PHN2ZyB4bWxu".to_owned());

    let mut wide_logo = base.clone();
    wide_logo.logo_width = Some(28);

    let base_svg = make_badge(&base).expect("badge render should succeed");
    let wide_svg = make_badge(&wide_logo).expect("badge render should succeed");

    assert_ne!(base_svg, wide_svg);
    assert!(wide_svg.contains("width=\"28\""));
}

#[test]
fn empty_label_and_message_are_allowed() {
    let mut options = BadgeOptions::new("");
    options.label = String::new();
    options.logo_base64 = Some("data:image/svg+xml;base64,PHN2ZyB4bWxu".to_owned());

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.starts_with("<svg "));
    assert!(svg.contains("<image "));
}

#[test]
fn typed_links_preserve_right_only_semantics() {
    let options = base_options().with_links(None::<String>, Some("https://example.com/right-only"));

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.contains("href=\"https://example.com/right-only\""));
    assert!(svg.contains("<a target=\"_blank\" href=\"https://example.com/right-only\">"));
}

#[test]
fn set_links_overwrites_existing_vector_links() {
    let mut options = base_options();
    options.links = vec![
        "https://example.com/left".to_owned(),
        "https://example.com/right".to_owned(),
    ];

    options.set_links(Some("https://example.com/replaced"), None::<String>);

    let svg = make_badge(&options).expect("badge render should succeed");

    assert!(svg.contains("https://example.com/replaced"));
    assert!(!svg.contains("https://example.com/right"));
}

#[test]
fn semantic_color_aliases_render_successfully() {
    let aliases = [
        "success",
        "important",
        "critical",
        "informational",
        "inactive",
    ];

    for alias in aliases {
        let mut options = base_options();
        options.color = Some(alias.to_owned());

        let svg = make_badge(&options).expect("badge render should succeed");

        assert!(svg.starts_with("<svg "), "alias `{alias}` did not render");
    }
}
