use std::{fs, path::Path};

use badge_maker_rs::{BadgeOptions, NamedColor, Style, make_badge};

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
    BadgeOptions::new(message)
        .label("example")
        .color(NamedColor::Blue)
        .style(style)
        .build()
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
fn tracked_style_preview_svgs_match_docs_assets() {
    for (name, options) in preview_cases() {
        let actual = make_badge(&options).expect("failed to render preview badge");
        let expected = tracked_preview_svg(name);
        assert_eq!(
            actual, expected,
            "tracked style preview SVG is stale for `{name}`"
        );
    }
}
