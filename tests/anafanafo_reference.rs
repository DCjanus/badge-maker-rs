use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
};

use badge_maker_rs::anafanafo::{self, Font, MeasureOptions};
use serde::{Deserialize, Serialize};

static BUN_SETUP: OnceLock<()> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReferenceCase {
    id: String,
    text: String,
    font: String,
    #[serde(default = "default_guess")]
    guess: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReferenceResult {
    id: String,
    ok: bool,
    width: Option<f32>,
    error: Option<String>,
}

fn default_guess() -> bool {
    true
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn bun_tool_dir() -> PathBuf {
    repo_root().join("tools/anafanafo-ref")
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

fn ensure_bun_dependencies() {
    BUN_SETUP.get_or_init(|| {
        let output = Command::new(bun_bin())
            .arg("install")
            .arg("--frozen-lockfile")
            .current_dir(bun_tool_dir())
            .output()
            .expect("failed to run `bun install --frozen-lockfile`");

        assert!(
            output.status.success(),
            "bun install failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    });
}

fn run_reference_batch(cases: &[ReferenceCase]) -> Vec<ReferenceResult> {
    ensure_bun_dependencies();

    let mut child = Command::new(bun_bin())
        .arg("run")
        .arg("index.js")
        .current_dir(bun_tool_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn Bun reference runner");

    let stdin = child.stdin.as_mut().expect("child stdin is unavailable");
    let payload = serde_json::to_vec(cases).expect("failed to serialize reference cases");
    stdin
        .write_all(&payload)
        .expect("failed to write reference batch to Bun stdin");
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .expect("failed to wait for Bun reference runner");

    assert!(
        output.status.success(),
        "reference runner failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("failed to parse reference runner output")
}

fn load_cases() -> Vec<ReferenceCase> {
    let path = repo_root().join("tests/data/anafanafo_cases.json");
    let text = fs::read_to_string(path).expect("failed to read test case file");
    serde_json::from_str(&text).expect("failed to parse test case file")
}

fn parse_font(font: &str) -> Font {
    match font {
        "10px Verdana" => Font::Verdana10,
        "bold 10px Verdana" => Font::Verdana10Bold,
        "11px Verdana" => Font::Verdana11,
        "bold 11px Helvetica" => Font::Helvetica11Bold,
        other => panic!("unsupported font in test case: {other}"),
    }
}

#[test]
fn reference_runner_supports_empty_batches() {
    let results = run_reference_batch(&[]);
    assert!(results.is_empty());
}

#[test]
fn reference_runner_returns_widths_for_known_inputs() {
    let cases = vec![ReferenceCase {
        id: "smoke-known-width".to_owned(),
        text: "m".to_owned(),
        font: "11px Verdana".to_owned(),
        guess: true,
    }];

    let results = run_reference_batch(&cases);
    assert_eq!(results.len(), 1);

    match &results[0] {
        ReferenceResult {
            id,
            ok: true,
            width: Some(width),
            error: None,
        } => {
            assert_eq!(id, "smoke-known-width");
            assert!((*width - 10.7).abs() <= 0.0001);
        }
        other => panic!("unexpected runner result: {other:?}"),
    }
}

#[test]
fn seed_cases_are_not_empty() {
    let cases = load_cases();
    assert!(
        !cases.is_empty(),
        "anafanafo reference cases must not be empty"
    );
}

#[test]
fn seed_case_ids_are_unique() {
    use std::collections::BTreeSet;

    let cases = load_cases();
    let mut seen = BTreeSet::new();

    for case in cases {
        assert!(
            seen.insert(case.id.clone()),
            "duplicate case id: {}",
            case.id
        );
    }
}

#[test]
fn anafanafo_matches_reference_cases() {
    let cases = load_cases();
    let reference_results = run_reference_batch(&cases);

    assert_eq!(
        cases.len(),
        reference_results.len(),
        "reference runner result count does not match input count"
    );

    for (case, reference) in cases.iter().zip(reference_results.iter()) {
        let actual = anafanafo::measure(
            &case.text,
            MeasureOptions::new(parse_font(&case.font)).with_guess(case.guess),
        );

        match (actual, reference) {
            (
                Ok(width),
                ReferenceResult {
                    id,
                    ok: true,
                    width: Some(expected_width),
                    error: None,
                },
            ) => {
                assert_eq!(id, &case.id);
                assert!(
                    (width - expected_width).abs() <= 0.0001,
                    "width mismatch for case `{}`: actual={}, expected={}",
                    case.id,
                    width,
                    expected_width
                );
            }
            (
                Err(error),
                ReferenceResult {
                    id,
                    ok: false,
                    width: None,
                    error: Some(expected_error),
                },
            ) => {
                assert_eq!(id, &case.id);
                assert_eq!(
                    error.to_string(),
                    *expected_error,
                    "error mismatch for case `{}`",
                    case.id
                );
            }
            (
                Ok(width),
                ReferenceResult {
                    id,
                    ok: false,
                    width: None,
                    error: Some(error),
                },
            ) => {
                panic!(
                    "case `{}` ({id}) unexpectedly succeeded with width {} but reference errored: {}",
                    case.id, width, error
                );
            }
            (
                Err(error),
                ReferenceResult {
                    id,
                    ok: true,
                    width: Some(width),
                    error: None,
                },
            ) => {
                panic!(
                    "case `{}` ({id}) unexpectedly errored: {} but reference returned width {}",
                    case.id, error, width
                );
            }
            (_, other) => panic!(
                "malformed reference result for case `{}`: {other:?}",
                case.id
            ),
        }
    }
}
