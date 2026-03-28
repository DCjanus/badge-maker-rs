use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use badge_maker_rs::{self, BadgeOptions, Style};
use resvg::{
    tiny_skia::{Pixmap, Transform},
    usvg::{Options, Tree},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReferenceCase {
    id: String,
    label: Option<String>,
    message: String,
    color: Option<String>,
    #[serde(rename = "labelColor")]
    label_color: Option<String>,
    style: Option<String>,
    #[serde(rename = "logoBase64")]
    logo_base64: Option<String>,
    #[serde(rename = "logoWidth")]
    logo_width: Option<u32>,
    links: Vec<String>,
    #[serde(rename = "idSuffix")]
    id_suffix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReferenceResult {
    id: String,
    ok: bool,
    output: Option<String>,
    error: Option<String>,
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn bun_bin() -> PathBuf {
    if let Some(value) = env::var_os("BUN_BIN") {
        return PathBuf::from(value);
    }
    if let Some(home) = env::var_os("HOME") {
        let candidate = PathBuf::from(home).join(".bun/bin/bun");
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from("bun")
}

fn bun_tool_dir() -> PathBuf {
    repo_root().join("tools/js-ref")
}

fn run_reference_batch(cases: &[ReferenceCase]) -> Vec<ReferenceResult> {
    let mut child = Command::new(bun_bin())
        .arg("--cwd")
        .arg(bun_tool_dir())
        .arg("--silent")
        .arg("--install=fallback")
        .arg("./index.js")
        .arg("badge-maker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn badge-maker reference runner");

    let payload = serde_json::to_vec(cases).expect("failed to serialize badge-maker cases");
    child
        .stdin
        .as_mut()
        .expect("child stdin unavailable")
        .write_all(&payload)
        .expect("failed to write badge-maker cases to bun stdin");
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .expect("failed to wait for badge-maker reference runner");
    assert!(
        output.status.success(),
        "reference runner failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("failed to parse badge-maker runner output")
}

fn load_cases() -> Vec<ReferenceCase> {
    let text = fs::read_to_string(repo_root().join("tests/data/badge_maker_cases.json"))
        .expect("failed to read badge-maker cases");
    serde_json::from_str(&text).expect("failed to parse badge-maker cases")
}

fn parse_style(style: Option<&str>) -> Style {
    match style.unwrap_or("flat") {
        "plastic" => Style::Plastic,
        "flat" => Style::Flat,
        "flat-square" => Style::FlatSquare,
        "for-the-badge" => Style::ForTheBadge,
        "social" => Style::Social,
        other => panic!("unsupported style {other}"),
    }
}

fn to_badge_options(case: &ReferenceCase) -> BadgeOptions {
    let mut options = BadgeOptions::new(case.message.clone());
    options.label = case.label.clone().unwrap_or_default();
    options.color = case.color.clone();
    options.label_color = case.label_color.clone();
    options.style = parse_style(case.style.as_deref());
    options.logo_base64 = case.logo_base64.clone();
    options.logo_width = case.logo_width;
    options.links = case.links.clone();
    options.id_suffix = case.id_suffix.clone();
    options
}

fn render_svg_to_rgba(svg: &str) -> Vec<u8> {
    let tree = Tree::from_str(svg, &Options::default()).expect("failed to parse SVG with usvg");
    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height()).expect("failed to create pixmap");
    resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());
    pixmap.data().to_vec()
}

#[test]
fn badge_maker_raster_output_matches_reference_cases() {
    let cases = load_cases();
    let reference_results = run_reference_batch(&cases);
    assert_eq!(cases.len(), reference_results.len());

    for (case, reference) in cases.iter().zip(reference_results.iter()) {
        let actual_svg =
            badge_maker_rs::make_badge(&to_badge_options(case)).expect("Rust badge render failed");

        let ReferenceResult {
            id,
            ok,
            output,
            error,
        } = reference;
        assert_eq!(id, &case.id);
        assert!(
            *ok,
            "reference runner errored for `{}`: {:?}",
            case.id, error
        );
        let expected_svg = output
            .as_deref()
            .expect("reference runner returned no SVG output");

        let actual_pixels = render_svg_to_rgba(&actual_svg);
        let expected_pixels = render_svg_to_rgba(expected_svg);

        assert_eq!(
            actual_pixels, expected_pixels,
            "rasterized pixel mismatch for case `{}`",
            case.id
        );
    }
}
