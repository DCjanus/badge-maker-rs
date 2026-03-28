use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use badge_maker_rs::{BadgeOptions, Color, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceCase {
    pub id: String,
    pub label: Option<String>,
    pub message: String,
    pub color: Option<String>,
    #[serde(rename = "labelColor")]
    pub label_color: Option<String>,
    pub style: Option<String>,
    #[serde(rename = "logoBase64")]
    pub logo_base64: Option<String>,
    #[serde(rename = "logoWidth")]
    pub logo_width: Option<u32>,
    pub links: Vec<String>,
    #[serde(rename = "idSuffix")]
    pub id_suffix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceResult {
    pub id: String,
    pub ok: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

pub fn repo_root() -> &'static Path {
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

pub fn run_reference_batch(cases: &[ReferenceCase]) -> Vec<ReferenceResult> {
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

pub fn load_cases() -> Vec<ReferenceCase> {
    let text = fs::read_to_string(repo_root().join("tests/data/badge_maker_cases.json"))
        .expect("failed to read badge-maker cases");
    serde_json::from_str(&text).expect("failed to parse badge-maker cases")
}

pub fn parse_style(style: Option<&str>) -> Style {
    match style.unwrap_or("flat") {
        "plastic" => Style::Plastic,
        "flat" => Style::Flat,
        "flat-square" => Style::FlatSquare,
        "for-the-badge" => Style::ForTheBadge,
        "social" => Style::Social,
        other => panic!("unsupported style {other}"),
    }
}

pub fn to_badge_options(case: &ReferenceCase) -> BadgeOptions {
    let mut options = BadgeOptions::builder()
        .message(case.message.clone())
        .label(case.label.clone().unwrap_or_default())
        .style(parse_style(case.style.as_deref()))
        .build();

    options.logo_base64 = case.logo_base64.clone();
    options.logo_width = case.logo_width;
    options.id_suffix = case.id_suffix.clone();

    if let Some(color) = &case.color {
        options.color = Some(Color::literal(color.clone()));
    }
    if let Some(label_color) = &case.label_color {
        options.label_color = Some(Color::literal(label_color.clone()));
    }

    match case.links.as_slice() {
        [] => {}
        [left] => options.left_link = Some(left.clone()),
        [left, right] if left.is_empty() => options.right_link = Some(right.clone()),
        [left, right, ..] => {
            options.left_link = Some(left.clone());
            options.right_link = Some(right.clone());
        }
    }

    options
}
