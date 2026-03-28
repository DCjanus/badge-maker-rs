use std::{fs, path::Path};

use badge_maker_rs::{BadgeOptions, Style, make_badge};

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn preview_cases() -> [(&'static str, BadgeOptions); 5] {
    [
        ("plastic", preview_options("plastic", Style::Plastic)),
        ("flat", preview_options("flat", Style::Flat)),
        (
            "flat-square",
            preview_options("flat-square", Style::FlatSquare),
        ),
        (
            "for-the-badge",
            preview_options("for-the-badge", Style::ForTheBadge),
        ),
        ("social", preview_options("social", Style::Social)),
    ]
}

fn preview_options(message: &str, style: Style) -> BadgeOptions {
    let mut options = BadgeOptions::new(message);
    options.label = "example".to_owned();
    options.color = Some("blue".to_owned());
    options.style = style;
    options
}

fn tracked_preview_svg(name: &str) -> String {
    let path = repo_root()
        .join("docs/style-previews")
        .join(format!("{name}.svg"));
    let file = fs::read_to_string(&path).expect("failed to read tracked preview SVG");
    file.lines()
        .skip_while(|line| *line != "-->")
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

#[test]
fn style_preview_markdown_references_all_tracked_svgs() {
    let markdown = fs::read_to_string(repo_root().join("docs/style-previews.md"))
        .expect("failed to read style preview markdown");

    for (name, _) in preview_cases() {
        let expected = format!("../docs/style-previews/{name}.svg");
        assert!(
            markdown.contains(&expected),
            "missing markdown preview reference for `{name}`"
        );
    }
}

#[test]
fn tracked_style_preview_svgs_match_current_renderer_output() {
    for (name, options) in preview_cases() {
        let actual = make_badge(&options).expect("failed to render preview badge");
        let expected = tracked_preview_svg(name);
        assert_eq!(
            actual, expected,
            "tracked style preview SVG is stale for `{name}`"
        );
    }
}
