// Rust-side compatibility entry point for `anafanafo`.
//
// Reference sources:
// - `metabolize/anafanafo`
// - `metabolize/anafanafo/packages/anafanafo`
// - `metabolize/anafanafo/packages/char-width-table-consumer`
//
// The goal is to reproduce upstream behavior and boundaries first, not to turn
// this module into a general-purpose text measurement library.

use core::fmt;

/// Limited font set currently supported by upstream `anafanafo`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Font {
    Verdana10,
    Verdana10Bold,
    Verdana11,
    Helvetica11Bold,
}

impl Font {
    /// Returns the font descriptor string used by the upstream Node.js package.
    #[allow(dead_code)]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verdana10 => "10px Verdana",
            Self::Verdana10Bold => "bold 10px Verdana",
            Self::Verdana11 => "11px Verdana",
            Self::Helvetica11Bold => "bold 11px Helvetica",
        }
    }
}

/// Input options for text measurement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeasureOptions {
    pub font: Font,
    pub guess: bool,
}

impl MeasureOptions {
    pub const fn new(font: Font) -> Self {
        Self { font, guess: true }
    }

    #[allow(dead_code)]
    pub const fn with_guess(mut self, guess: bool) -> Self {
        self.guess = guess;
        self
    }
}

impl Default for MeasureOptions {
    fn default() -> Self {
        Self::new(Font::Verdana11)
    }
}

/// Error returned by the `anafanafo` compatibility layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    MissingWidth { char_code: u32 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingWidth { char_code } => {
                write!(f, "No width available for character code {char_code}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Consumer for compressed character-width tables.
///
/// The underlying font tables are generated at build time from the JSON files
/// in `data/anafanafo/`, so measurement does not perform runtime JSON parsing.
#[derive(Clone, Debug)]
pub struct CharWidthTableConsumer {
    data: &'static [WidthTableRange],
    em_width: f32,
}

impl CharWidthTableConsumer {
    pub const fn new_static(data: &'static [WidthTableRange], em_width: f32) -> Self {
        Self { data, em_width }
    }

    pub fn width_of(&self, text: &str, options: WidthOfOptions) -> Result<f32, Error> {
        text.chars().try_fold(0.0, |accum_width, ch| {
            let char_code = ch as u32;
            match self.width_of_char_code(char_code) {
                Some(char_width) => Ok(accum_width + char_width),
                None if options.guess => Ok(accum_width + self.em_width),
                None => Err(Error::MissingWidth { char_code }),
            }
        })
    }

    pub fn width_of_char_code(&self, char_code: u32) -> Option<f32> {
        if is_control_char(char_code) {
            return Some(0.0);
        }

        let index = self.data.partition_point(|range| range.lower <= char_code);
        if index == 0 {
            return None;
        }

        let candidate = &self.data[index - 1];
        candidate.contains(char_code).then_some(candidate.width)
    }
}

/// Optional parameters for `width_of()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WidthOfOptions {
    pub guess: bool,
}

impl WidthOfOptions {
    pub const fn new() -> Self {
        Self { guess: true }
    }

    #[allow(dead_code)]
    pub const fn with_guess(mut self, guess: bool) -> Self {
        self.guess = guess;
        self
    }
}

impl Default for WidthOfOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// One range entry in a compressed width table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidthTableRange {
    pub lower: u32,
    pub upper: u32,
    pub width: f32,
}

impl WidthTableRange {
    pub const fn new(lower: u32, upper: u32, width: f32) -> Self {
        Self {
            lower,
            upper,
            width,
        }
    }

    const fn contains(self, char_code: u32) -> bool {
        self.lower <= char_code && char_code <= self.upper
    }
}

/// Measures text width.
///
/// This is the Rust-side wrapper around upstream
/// `anafanafo(text, { font, guess })`.
pub fn measure(text: &str, options: MeasureOptions) -> Result<f32, Error> {
    builtin_consumer(options.font).width_of(
        text,
        WidthOfOptions {
            guess: options.guess,
        },
    )
}

fn is_control_char(char_code: u32) -> bool {
    char_code <= 31 || char_code == 127
}

fn builtin_consumer(font: Font) -> &'static CharWidthTableConsumer {
    generated_tables::consumer(font)
}

#[allow(clippy::approx_constant)]
mod generated_tables {
    use super::{CharWidthTableConsumer, Font, WidthTableRange};

    include!(concat!(env!("OUT_DIR"), "/anafanafo_tables.rs"));

    pub(super) fn consumer(font: Font) -> &'static CharWidthTableConsumer {
        match font {
            Font::Verdana10 => &VERDANA_10_CONSUMER,
            Font::Verdana10Bold => &VERDANA_10_BOLD_CONSUMER,
            Font::Verdana11 => &VERDANA_11_CONSUMER,
            Font::Helvetica11Bold => &HELVETICA_11_BOLD_CONSUMER,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        io::Write,
        path::{Path, PathBuf},
        process::{Command, Stdio},
    };

    use serde::{Deserialize, Serialize};

    use super::{Font, MeasureOptions};

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
        repo_root().join("tools/js-ref")
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

    fn run_reference_batch(cases: &[ReferenceCase]) -> Vec<ReferenceResult> {
        let mut child = Command::new(bun_bin())
            .arg("--cwd")
            .arg(bun_tool_dir())
            .arg("--silent")
            .arg("--install=fallback")
            .arg("./index.js")
            .arg("anafanafo")
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
            let actual = super::measure(
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
                        "case `{}` ({id}) unexpectedly errored with {} but reference returned width {}",
                        case.id, error, width
                    );
                }
                (_, other) => panic!(
                    "unexpected reference result for case `{}`: {other:?}",
                    case.id
                ),
            }
        }
    }
}
