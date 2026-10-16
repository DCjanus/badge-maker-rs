use std::{env, fs, path::PathBuf, process::Command};

use resvg::tiny_skia::{Pixmap, PremultipliedColorU8};
use serde_json::{Value, json};

use super::{RasterizedSvg, ReferenceCase, repo_root};

pub struct ReferenceReport {
    directory: PathBuf,
    rows: String,
    results: Vec<Value>,
    failures: Vec<String>,
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn pixmap(image: &RasterizedSvg) -> Pixmap {
    let mut pixmap = Pixmap::new(image.width, image.height).unwrap();
    pixmap.data_mut().copy_from_slice(&image.pixels);
    pixmap
}

impl ReferenceReport {
    pub fn new() -> Self {
        let directory = repo_root().join("target/reference-report");
        if directory.exists() {
            fs::remove_dir_all(&directory).expect("failed to clear previous reference report");
        }
        fs::create_dir_all(&directory).expect("failed to create reference report directory");
        Self {
            directory,
            rows: String::new(),
            results: Vec::new(),
            failures: Vec::new(),
        }
    }

    pub fn record_render(
        &mut self,
        case: &ReferenceCase,
        actual_svg: &str,
        reference_svg: &str,
        actual: &RasterizedSvg,
        reference: &RasterizedSvg,
    ) {
        // IDs become relative file paths in the report, so reject path separators.
        assert!(
            !case.id.is_empty()
                && case
                    .id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        );
        let actual_image = pixmap(actual);
        let reference_image = pixmap(reference);
        let mut diff = Pixmap::new(
            actual.width.max(reference.width),
            actual.height.max(reference.height),
        )
        .unwrap();
        let mut different_pixels = 0;
        for y in 0..diff.height() {
            for x in 0..diff.width() {
                let actual_pixel =
                    (x < actual.width && y < actual.height).then(|| actual_image.pixel(x, y));
                let reference_pixel = (x < reference.width && y < reference.height)
                    .then(|| reference_image.pixel(x, y));
                if actual_pixel != reference_pixel {
                    different_pixels += 1;
                    let index = (y * diff.width() + x) as usize;
                    diff.pixels_mut()[index] =
                        PremultipliedColorU8::from_rgba(255, 0, 255, 255).unwrap();
                }
            }
        }
        let matches = actual == reference;
        for (suffix, image) in [
            ("rust", &actual_image),
            ("upstream", &reference_image),
            ("diff", &diff),
        ] {
            image
                .save_png(self.directory.join(format!("{}-{suffix}.png", case.id)))
                .unwrap();
        }
        for (suffix, svg) in [("rust", actual_svg), ("upstream", reference_svg)] {
            fs::write(
                self.directory.join(format!("{}-{suffix}.svg", case.id)),
                svg,
            )
            .unwrap();
        }
        self.record_result(
            case,
            matches,
            json!({
                "rust_dimensions": [actual.width, actual.height],
                "upstream_dimensions": [reference.width, reference.height],
                "different_pixels": different_pixels,
            }),
        );
        let id = &case.id;
        let input = escape_html(&serde_json::to_string_pretty(case).unwrap());
        self.rows.push_str(&format!(
            "<tr class='{class}'><td>{id}<details><summary>Input</summary><pre>{input}</pre></details></td><td>{status}<br>{different_pixels} differing pixels</td><td><img alt='Rust comparison pixels' src='{id}-rust.png'><br>{}×{}<br><a href='{id}-rust.svg'>SVG source</a></td><td><img alt='Upstream comparison pixels' src='{id}-upstream.png'><br>{}×{}<br><a href='{id}-upstream.svg'>SVG source</a></td><td><img alt='Magenta marks unequal pixels' src='{id}-diff.png'></td></tr>",
            actual.width, actual.height, reference.width, reference.height,
            class = if matches { "pass" } else { "fail" },
            status = if matches { "PASS" } else { "FAIL" },
        ));
    }

    pub fn record_error(&mut self, case: &ReferenceCase, actual: &str, expected: &str) {
        let matches = actual == expected;
        self.record_result(
            case,
            matches,
            json!({"rust_error": actual, "upstream_error": expected}),
        );
        self.rows.push_str(&format!(
            "<tr class='{}'><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>Error comparison</td></tr>",
            if matches { "pass" } else { "fail" }, escape_html(&case.id),
            if matches { "PASS" } else { "FAIL" }, escape_html(actual), escape_html(expected),
        ));
    }

    fn record_result(&mut self, case: &ReferenceCase, matches: bool, detail: Value) {
        if !matches {
            self.failures.push(case.id.clone());
        }
        self.results
            .push(json!({"id": case.id, "input": case, "matches": matches, "detail": detail}));
    }

    pub fn finish(self) {
        let revision = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_root())
            .output()
            .unwrap();
        let rustc = Command::new("rustc").arg("--version").output().unwrap();
        let reference_package: Value =
            serde_json::from_str(include_str!("../../tools/js-ref/package.json")).unwrap();
        let metadata = json!({
            "revision": String::from_utf8_lossy(&revision.stdout).trim(),
            "rustc": String::from_utf8_lossy(&rustc.stdout).trim(),
            "os": env::consts::OS, "arch": env::consts::ARCH,
            "github_run_id": env::var("GITHUB_RUN_ID").ok(),
            "github_run_attempt": env::var("GITHUB_RUN_ATTEMPT").ok(),
            "reference_dependencies": reference_package["dependencies"],
            "fonts": "DejaVu Sans 2.37 regular and bold (bundled)",
            "rasterizer": "resvg; exact version recorded in Cargo.lock",
            "pixel_tolerance": 0,
            "cases": self.results.len(), "failures": self.failures.len(),
        });
        fs::write(
            self.directory.join("results.json"),
            serde_json::to_vec_pretty(&self.results).unwrap(),
        )
        .unwrap();
        fs::write(
            self.directory.join("metadata.json"),
            serde_json::to_vec_pretty(&metadata).unwrap(),
        )
        .unwrap();
        fs::copy(
            repo_root().join("Cargo.lock"),
            self.directory.join("Cargo.lock"),
        )
        .unwrap();
        fs::copy(
            repo_root().join("tools/js-ref/bun.lock"),
            self.directory.join("bun.lock"),
        )
        .unwrap();
        let html = format!(
            r#"<!doctype html>
<html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>badge-maker-rs reference comparisons</title>
<style>body{{font:16px system-ui,sans-serif;margin:2rem}}table{{border-collapse:collapse}}td,th{{border:1px solid #bbb;padding:.7rem;text-align:left}}th{{background:#eee}}pre{{font-size:12px;max-width:28rem;overflow:auto}}img{{image-rendering:pixelated;background:repeating-conic-gradient(#eee 0% 25%,#fff 0% 50%) 0/8px 8px}}img:hover{{transform:scale(3);transform-origin:left center;position:relative;z-index:1}}tr.fail{{background:#fff0f0}}body:has(#failures:checked) tr.pass{{display:none}}</style>
<h1>badge-maker-rs reference comparisons</h1><p>{} cases; {} failures.</p>
<p>Images are the exact native-size PNG pixels compared by the test, including text and embedded logos. Both renderers use the same bundled fonts. Hover to magnify; magenta marks unequal pixels. Opening the SVG source separately may use your browser's fonts.</p>
<p><a href="metadata.json">Run metadata</a> · <a href="results.json">Inputs and results</a> · <a href="Cargo.lock">Rust dependency lock</a> · <a href="bun.lock">Reference dependency lock</a></p>
<p>This suite compares pinned upstream badge-maker code, not the changing live Shields.io deployment.</p>
<label><input type="checkbox" id="failures"> Show only mismatches</label>
<table><thead><tr><th>Case</th><th>Result</th><th>Rust</th><th>Upstream JavaScript</th><th>Difference</th></tr></thead><tbody>{}</tbody></table></html>"#,
            self.results.len(),
            self.failures.len(),
            self.rows,
        );
        fs::write(self.directory.join("index.html"), html).unwrap();
        assert!(
            self.failures.is_empty(),
            "reference mismatches: {:?}; inspect {}",
            self.failures,
            self.directory.join("index.html").display()
        );
    }
}

#[test]
fn mismatch_report_preserves_all_cases_and_marks_pixel_and_dimension_differences() {
    let directory = repo_root().join(format!(
        "target/reference-report-smoke-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    let mut report = ReferenceReport {
        directory: directory.clone(),
        rows: String::new(),
        results: Vec::new(),
        failures: Vec::new(),
    };
    let mut case = super::load_cases().remove(0);
    case.id = "different-color".into();
    let blank = RasterizedSvg {
        width: 2,
        height: 1,
        pixels: vec![0; 8],
    };
    let colored = RasterizedSvg {
        width: 2,
        height: 1,
        pixels: vec![255; 8],
    };
    report.record_render(&case, "", "", &blank, &colored);
    case.id = "different-shape".into();
    let tall = RasterizedSvg {
        width: 1,
        height: 2,
        pixels: vec![0; 8],
    };
    report.record_render(&case, "", "", &blank, &tall);
    assert!(std::panic::catch_unwind(|| report.finish()).is_err());
    let results: Vec<Value> =
        serde_json::from_slice(&fs::read(directory.join("results.json")).unwrap()).unwrap();
    assert_eq!(results.len(), 2);
    for result in &results {
        assert_eq!(result["matches"], false);
        assert_eq!(result["detail"]["different_pixels"], 2);
    }
    let diff = Pixmap::load_png(directory.join("different-shape-diff.png")).unwrap();
    assert_eq!((diff.width(), diff.height()), (2, 2));
    let magenta = PremultipliedColorU8::from_rgba(255, 0, 255, 255);
    assert_eq!(diff.pixel(1, 0), magenta);
    assert_eq!(diff.pixel(0, 1), magenta);
    assert_eq!(diff.pixel(0, 0).unwrap().alpha(), 0);
    let html = fs::read_to_string(directory.join("index.html")).unwrap();
    assert!(html.contains("2 cases; 2 failures."));
    fs::remove_dir_all(directory).unwrap();
}
