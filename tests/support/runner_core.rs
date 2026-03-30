use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

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
    pub logo_data_url: Option<String>,
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
