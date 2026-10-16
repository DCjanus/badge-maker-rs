//! Upstream reference comparison report.
//!
//! `finish` writes a self-contained `index.html` (comparison PNGs, SVG sources,
//! metadata, results, and dependency locks embedded as data URLs) next to the
//! raw per-case PNG/SVG evidence and the JSON result files. The page layout,
//! styling, and script live in `tests/support/report/` and are filled from the
//! real comparison results; see `CONTRIBUTING.md`.

use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use resvg::tiny_skia::{Pixmap, PremultipliedColorU8};
use serde_json::{Value, json};

use super::{RasterizedSvg, ReferenceCase, group_filter_value, group_name, repo_root};
use html::{escape_html, render_template};
use source::{RepoState, SourceContext, SourceInput, derive as derive_source};

mod html;
mod source;

const PAGE_TEMPLATE: &str = include_str!("report/page.html");
const STYLESHEET: &str = include_str!("report/report.css");
const SCRIPT: &str = include_str!("report/report.js");

/// Machine-readable evidence kept next to the report and embedded into it as
/// downloads, so the single HTML file stays usable on its own.
const EVIDENCE_FILES: [(&str, &str); 4] = [
    ("metadata.json", "application/json"),
    ("results.json", "application/json"),
    ("Cargo.lock", "text/plain"),
    ("bun.lock", "text/plain"),
];

struct GroupCount {
    value: String,
    name: String,
    cases: usize,
    failures: usize,
}

struct CaseRow {
    id: String,
    group: Option<String>,
    style: String,
    passed: bool,
    search_text: String,
    input_json: String,
    detail: RowDetail,
}

enum RowDetail {
    Render {
        rust: (u32, u32),
        upstream: (u32, u32),
        different_pixels: u64,
    },
    Error {
        rust: String,
        upstream: String,
    },
}

pub struct ReferenceReport {
    directory: PathBuf,
    rows: Vec<CaseRow>,
    results: Vec<Value>,
    failures: Vec<String>,
}

fn pixmap(image: &RasterizedSvg) -> Pixmap {
    let mut pixmap = Pixmap::new(image.width, image.height).unwrap();
    pixmap.data_mut().copy_from_slice(&image.pixels);
    pixmap
}

fn assert_case_id(case: &ReferenceCase) {
    // IDs become evidence file names, so reject path separators.
    assert!(
        !case.id.is_empty()
            && case
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'),
        "case id must be a safe file name component: {:?}",
        case.id
    );
}

fn case_search_text(case: &ReferenceCase, status: &str, outcome: &str) -> String {
    let mut parts = vec![
        case.id.clone(),
        case.style.clone().unwrap_or_else(|| "default".to_owned()),
        group_filter_value(case.group.as_deref()).to_owned(),
        group_name(case.group.as_deref()).to_owned(),
        status.to_owned(),
        outcome.to_owned(),
    ];
    for value in [
        case.label.as_deref(),
        Some(case.message.as_str()),
        case.color.as_deref(),
        case.label_color.as_deref(),
        case.id_suffix.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        parts.push(value.to_owned());
    }
    if case
        .logo_data_url
        .as_deref()
        .is_some_and(|logo| !logo.is_empty())
    {
        parts.push("logo".to_owned());
    }
    parts.extend(case.links.iter().cloned());
    parts
        .join(" ")
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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
            rows: Vec::new(),
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
        assert_case_id(case);
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
        let outcome = if different_pixels == 0 {
            "identical"
        } else {
            "differing pixels"
        };
        self.rows.push(CaseRow {
            id: case.id.clone(),
            group: case.group.clone(),
            style: case.style.clone().unwrap_or_else(|| "default".to_owned()),
            passed: matches,
            search_text: case_search_text(case, if matches { "pass" } else { "fail" }, outcome),
            input_json: serde_json::to_string_pretty(case).unwrap(),
            detail: RowDetail::Render {
                rust: (actual.width, actual.height),
                upstream: (reference.width, reference.height),
                different_pixels,
            },
        });
    }

    pub fn record_error(&mut self, case: &ReferenceCase, actual: &str, expected: &str) {
        assert_case_id(case);
        let matches = actual == expected;
        self.record_result(
            case,
            matches,
            json!({"rust_error": actual, "upstream_error": expected}),
        );
        self.rows.push(CaseRow {
            id: case.id.clone(),
            group: case.group.clone(),
            style: case.style.clone().unwrap_or_else(|| "default".to_owned()),
            passed: matches,
            search_text: case_search_text(
                case,
                if matches { "pass" } else { "fail" },
                if matches {
                    "error matched"
                } else {
                    "error differs"
                },
            ),
            input_json: serde_json::to_string_pretty(case).unwrap(),
            detail: RowDetail::Error {
                rust: actual.to_owned(),
                upstream: expected.to_owned(),
            },
        });
    }

    fn record_result(&mut self, case: &ReferenceCase, matches: bool, detail: Value) {
        if !matches {
            self.failures.push(case.id.clone());
        }
        self.results
            .push(json!({"id": case.id, "input": case, "matches": matches, "detail": detail}));
    }

    /// Reads an evidence file back out of the report directory as a data URL.
    fn data_uri(&self, file: &str, mime: &str) -> Option<String> {
        let bytes = fs::read(self.directory.join(file)).ok()?;
        Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
    }

    fn badge_cell(&self, id: &str, suffix: &str, size: (u32, u32), alt: &str) -> String {
        let (width, height) = size;
        let image = match self.data_uri(&format!("{id}-{suffix}.png"), "image/png") {
            Some(url) => format!(
                "<img class=\"badge\" src=\"{url}\" alt=\"{}\" width=\"{width}\" height=\"{height}\" loading=\"lazy\" decoding=\"async\" style=\"--badge-width:{width}px\">",
                escape_html(alt)
            ),
            None => "<span class=\"error-inline\">image missing</span>".to_owned(),
        };
        format!("<td class=\"cell-image\"><div class=\"scroll\">{image}</div></td>")
    }

    fn row_html(&self, row: &CaseRow) -> String {
        let id = row.id.as_str();
        let escaped_id = escape_html(id);
        let detail_id = format!("detail-{id}");
        let group = group_name(row.group.as_deref());
        let mut cells = format!(
            "<td class=\"cell-case\"><button type=\"button\" class=\"case-toggle\" aria-expanded=\"false\" aria-controls=\"{detail_id}\" title=\"Show case details.\"><span class=\"case-id\">{escaped_id}</span><span class=\"chev\" aria-hidden=\"true\">&#9656;</span></button></td>"
        );
        let mut detail;

        match &row.detail {
            RowDetail::Render {
                rust,
                upstream,
                different_pixels,
            } => {
                cells.push_str(&self.badge_cell(
                    id,
                    "rust",
                    *rust,
                    &format!("{id} rendered by badge-maker-rs"),
                ));
                cells.push_str(&self.badge_cell(
                    id,
                    "upstream",
                    *upstream,
                    &format!("{id} rendered by upstream badge-maker"),
                ));
                if row.passed {
                    cells.push_str(
                        "<td class=\"cell-diff\"><span class=\"identical\" title=\"Pixel-identical to upstream.\">Identical</span></td>",
                    );
                } else {
                    let diff_width = rust.0.max(upstream.0);
                    let diff_height = rust.1.max(upstream.1);
                    let image = match self.data_uri(&format!("{id}-diff.png"), "image/png") {
                        Some(url) => format!(
                            "<img class=\"badge badge--diff\" src=\"{url}\" alt=\"Difference image for {escaped_id}; magenta marks differing pixels\" width=\"{diff_width}\" height=\"{diff_height}\" loading=\"lazy\" decoding=\"async\" style=\"--badge-width:{diff_width}px\">"
                        ),
                        None => "<span class=\"error-inline\">image missing</span>".to_owned(),
                    };
                    cells.push_str(&format!(
                        "<td class=\"cell-diff\"><div class=\"diff-fail\"><div class=\"scroll\">{image}</div><span class=\"differing\" title=\"Differing pixels\">{different_pixels} px</span></div></td>"
                    ));
                }

                let mut links = String::new();
                for (suffix, label) in [("rust", "Rust"), ("upstream", "Upstream")] {
                    if let Some(url) = self.data_uri(&format!("{id}-{suffix}.svg"), "image/svg+xml")
                    {
                        links.push_str(&format!(
                            "<a href=\"{url}\" download=\"{escaped_id}-{suffix}.svg\" title=\"Download {label} SVG.\">{label} SVG</a>"
                        ));
                    }
                }
                if links.is_empty() {
                    links.push_str("<span class=\"identical\">SVG sources unavailable</span>");
                }
                detail = format!(
                    "<p class=\"detail-meta\">{} · Rust {}×{} · upstream {}×{} · {different_pixels} differing pixels</p><p class=\"detail-links\">{links}</p>",
                    escape_html(group),
                    rust.0,
                    rust.1,
                    upstream.0,
                    upstream.1
                );
            }
            RowDetail::Error { rust, upstream } => {
                for message in [rust, upstream] {
                    cells.push_str(&format!(
                        "<td class=\"cell-image\"><span class=\"error-inline\" title=\"{}\">{}</span></td>",
                        escape_html(message),
                        escape_html(message)
                    ));
                }
                cells.push_str(if row.passed {
                    "<td class=\"cell-diff\"><span class=\"identical\">Matching error</span></td>"
                } else {
                    "<td class=\"cell-diff\"><span class=\"differing\">Different errors</span></td>"
                });
                detail = format!(
                    "<p class=\"detail-meta\">{} · error comparison</p><pre class=\"detail-json\">rust error: {}\nupstream error: {}</pre>",
                    escape_html(group),
                    escape_html(rust),
                    escape_html(upstream)
                );
            }
        }

        detail.push_str(&format!(
            "<pre class=\"detail-json\">{}</pre>",
            escape_html(&row.input_json)
        ));
        format!(
            "<tr class=\"row{}\" data-case data-detail=\"{detail_id}\" data-status=\"{}\" data-group=\"{}\" data-style=\"{}\" data-search=\"{}\">{cells}</tr><tr class=\"detail-row\" id=\"{detail_id}\" hidden><td colspan=\"4\"><div class=\"detail-body\">{detail}</div></td></tr>",
            if row.passed { "" } else { " row--fail" },
            if row.passed { "pass" } else { "fail" },
            escape_html(group_filter_value(row.group.as_deref())),
            escape_html(&row.style),
            escape_html(&row.search_text),
        )
    }

    fn aggregate(&self, reference_version: &str) -> String {
        let total = self.rows.len();
        let failures = self.failures.len();
        let tip = format!("Exact pixels · badge-maker {reference_version} · DejaVu Sans 2.37");
        let text = if failures == 0 {
            format!("{total}/{total} match")
        } else {
            format!("{}/{total} match · {failures} failing", total - failures)
        };
        format!(
            "<span class=\"aggregate{} tip\" tabindex=\"0\" role=\"status\" data-tip=\"{}\" data-summary=\"cases\" data-cases=\"{total}\" data-failures=\"{failures}\">{text}</span>",
            if failures == 0 {
                ""
            } else {
                " aggregate--fail"
            },
            escape_html(&tip)
        )
    }

    fn info_panel(
        &self,
        source: &SourceContext,
        metadata: &Value,
        groups: &[GroupCount],
    ) -> String {
        let mut rows: Vec<(String, String)> = Vec::new();
        let ci_value = match &source.ci_url {
            Some(url) => format!(
                "<a href=\"{}\">{}</a>",
                escape_html(url),
                escape_html(&source.ci_label)
            ),
            None => escape_html(&source.ci_label),
        };
        rows.push(("CI".to_owned(), ci_value));

        let commit_value = match &source.commit_url {
            Some(url) => {
                let title = if source.is_ci() && source.merge_commit {
                    " title=\"Tested PR merge commit.\""
                } else {
                    ""
                };
                format!(
                    "<a href=\"{}\"{title}><code>{}</code></a>",
                    escape_html(url),
                    escape_html(&source.commit_short)
                )
            }
            None => format!("<code>{}</code>", escape_html(&source.commit_short)),
        };
        let commit_value = if source.local_changes {
            format!("{commit_value} <span class=\"identical\">uncommitted changes</span>")
        } else {
            commit_value
        };
        rows.push(("Commit".to_owned(), commit_value));

        if let Some(number) = source.pull_request_number {
            let value = match &source.pull_request_url {
                Some(url) => format!("<a href=\"{}\">#{number}</a>", escape_html(url)),
                None => format!("#{number}"),
            };
            rows.push(("PR".to_owned(), value));
        }
        rows.push((
            "Tested revision".to_owned(),
            escape_html(&source.revision_note),
        ));
        if let Some(git_ref) = source.git_ref.as_deref().filter(|_| source.is_ci()) {
            rows.push((
                "Git ref".to_owned(),
                format!("<code>{}</code>", escape_html(git_ref)),
            ));
        }
        rows.push((
            "Comparison".to_owned(),
            "Exact RGBA · zero tolerance · resvg".to_owned(),
        ));
        rows.push((
            "Fonts".to_owned(),
            escape_html(metadata["fonts"].as_str().unwrap_or_default()),
        ));
        rows.push((
            "Rust".to_owned(),
            escape_html(metadata["rustc"].as_str().unwrap_or_default()),
        ));
        rows.push((
            "Platform".to_owned(),
            format!(
                "{} / {}",
                escape_html(metadata["os"].as_str().unwrap_or_default()),
                escape_html(metadata["arch"].as_str().unwrap_or_default())
            ),
        ));
        let dependencies = metadata["reference_dependencies"]
            .as_object()
            .map(|dependencies| {
                dependencies
                    .iter()
                    .map(|(name, version)| {
                        escape_html(&format!("{name} {}", version.as_str().unwrap_or_default()))
                    })
                    .collect::<Vec<_>>()
                    .join(" · ")
            })
            .unwrap_or_default();
        rows.push(("Reference deps".to_owned(), dependencies));
        rows.push((
            "Groups".to_owned(),
            groups
                .iter()
                .map(|group| escape_html(&format!("{} {}", group.name, group.cases)))
                .collect::<Vec<_>>()
                .join(" · "),
        ));
        rows.push((
            "Cases".to_owned(),
            format!("{} · {} failing", self.rows.len(), self.failures.len()),
        ));

        let downloads = EVIDENCE_FILES
            .iter()
            .filter_map(|(file, mime)| {
                self.data_uri(file, mime)
                    .map(|url| format!("<a href=\"{url}\" download=\"{file}\">{file}</a>"))
            })
            .collect::<Vec<_>>()
            .join("");
        let list = rows
            .iter()
            .map(|(term, value)| format!("<dt>{term}</dt><dd>{value}</dd>"))
            .collect::<String>();

        format!(
            "<div id=\"report-info\" class=\"info-panel\" popover role=\"dialog\" aria-labelledby=\"report-info-title\"><div class=\"info-head\"><h2 id=\"report-info-title\">Report info</h2><button type=\"button\" class=\"info-close\" popovertarget=\"report-info\" popovertargetaction=\"hide\" aria-label=\"Close report info\">&#215;</button></div><dl class=\"info-list\">{list}</dl><p class=\"info-downloads\"><span class=\"info-downloads-label\">Downloads</span>{downloads}</p></div>"
        )
    }

    /// Writes the report and panics with every mismatch once the evidence a
    /// reviewer needs is on disk.
    pub fn finish(self) {
        let source_input = source_input();
        let source = derive_source(&source_input);
        let reference_package: Value =
            serde_json::from_str(include_str!("../../tools/js-ref/package.json")).unwrap();
        let reference_version = reference_package["dependencies"]["badge-maker"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        let groups = group_counts(&self.rows);
        let styles = style_counts(&self.rows);

        let metadata = json!({
            "revision": source.commit_sha,
            "rustc": command_stdout("rustc", &["--version"]).unwrap_or_else(|| "unknown".to_owned()),
            "os": env::consts::OS,
            "arch": env::consts::ARCH,
            "github_run_id": env::var("GITHUB_RUN_ID").ok(),
            "github_run_attempt": env::var("GITHUB_RUN_ATTEMPT").ok(),
            "reference_dependencies": reference_package["dependencies"],
            "fonts": "DejaVu Sans 2.37 regular and bold (bundled)",
            "rasterizer": "resvg; exact version recorded in Cargo.lock",
            "pixel_tolerance": 0,
            "cases": self.rows.len(),
            "failures": self.failures.len(),
            "source": &source,
            "working_tree_dirty": source_input.repo_state.dirty,
            "groups": groups
                .iter()
                .map(|group| json!({
                    "id": group.value,
                    "name": group.name,
                    "cases": group.cases,
                    "failures": group.failures,
                }))
                .collect::<Vec<_>>(),
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

        let rows = groups
            .iter()
            .map(|group| {
                let mut html = format!(
                    "<tr class=\"group-row\" data-group-row=\"{}\"><td colspan=\"4\"><button type=\"button\" class=\"group-toggle\" data-group-toggle=\"{}\" aria-expanded=\"true\" title=\"Collapse or expand this group\"><span class=\"group-chev\" aria-hidden=\"true\">&#9662;</span><span class=\"group-name\">{}</span><span class=\"group-count\" data-group-count=\"{}\">{} cases</span></button></td></tr>",
                    escape_html(&group.value),
                    escape_html(&group.value),
                    escape_html(&group.name),
                    escape_html(&group.value),
                    group.cases,
                );
                for row in self.rows.iter().filter(|row| {
                    group_filter_value(row.group.as_deref()) == group.value.as_str()
                }) {
                    html.push_str(&self.row_html(row));
                }
                html
            })
            .collect::<String>();
        let html = render_template(
            PAGE_TEMPLATE,
            &[
                ("TITLE", "badge-maker-rs reference comparisons"),
                ("STYLESHEET", STYLESHEET),
                ("SCRIPT", SCRIPT),
                ("AGGREGATE", &self.aggregate(&reference_version)),
                ("INFO_PANEL", &self.info_panel(&source, &metadata, &groups)),
                ("GROUP_OPTIONS", &group_options(&groups, self.rows.len())),
                ("STYLE_OPTIONS", &style_options(&styles)),
                ("ROWS", &rows),
            ],
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

fn group_counts(rows: &[CaseRow]) -> Vec<GroupCount> {
    let mut counts: BTreeMap<String, GroupCount> = BTreeMap::new();
    for row in rows {
        let value = group_filter_value(row.group.as_deref()).to_owned();
        let entry = counts.entry(value.clone()).or_insert_with(|| GroupCount {
            value,
            name: group_name(row.group.as_deref()).to_owned(),
            cases: 0,
            failures: 0,
        });
        entry.cases += 1;
        if !row.passed {
            entry.failures += 1;
        }
    }
    counts.into_values().collect()
}

fn style_counts(rows: &[CaseRow]) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows {
        *counts.entry(row.style.clone()).or_default() += 1;
    }
    let mut counts = counts.into_iter().collect::<Vec<_>>();
    counts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    counts
}

fn group_options(groups: &[GroupCount], total: usize) -> String {
    let mut html = format!("<option value=\"all\">All cases ({total})</option>");
    for group in groups {
        html.push_str(&format!(
            "<option value=\"{}\">{} ({})</option>",
            escape_html(&group.value),
            escape_html(&group.name),
            group.cases
        ));
    }
    html
}

fn style_options(styles: &[(String, usize)]) -> String {
    let mut html = "<option value=\"all\">All styles</option>".to_owned();
    for (style, count) in styles {
        html.push_str(&format!(
            "<option value=\"{}\">{} ({count})</option>",
            escape_html(style),
            escape_html(style)
        ));
    }
    html
}

fn env_value(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn command_stdout(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(repo_root())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn source_input() -> SourceInput {
    let commit_sha = command_stdout("git", &["rev-parse", "HEAD"])
        .or_else(|| env_value("GITHUB_SHA"))
        .unwrap_or_else(|| "unknown".to_owned());
    let dirty = command_stdout("git", &["status", "--porcelain"])
        .is_some_and(|output| !output.trim().is_empty());
    let merge_commit = checkout_is_merge(repo_root());
    SourceInput {
        server_url: env_value("GITHUB_SERVER_URL"),
        repository: env_value("GITHUB_REPOSITORY"),
        run_id: env_value("GITHUB_RUN_ID"),
        run_attempt: env_value("GITHUB_RUN_ATTEMPT"),
        git_ref: env_value("GITHUB_REF"),
        event_payload: env_value("GITHUB_EVENT_PATH")
            .and_then(|path| fs::read_to_string(path).ok()),
        cargo_repository: Some(env!("CARGO_PKG_REPOSITORY").to_owned())
            .filter(|repository| !repository.is_empty()),
        commit_sha,
        repo_state: RepoState {
            dirty,
            merge_commit,
        },
    }
}

fn checkout_is_merge(directory: &Path) -> bool {
    Command::new("git")
        // Read the stored headers: revision traversal hides parents at a
        // shallow boundary, including GitHub's default PR checkout.
        .args(["cat-file", "-p", "HEAD"])
        .current_dir(directory)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .take_while(|line| !line.is_empty())
                .filter(|line| line.starts_with("parent "))
                .count()
                > 1
        })
}

#[test]
fn shallow_checkout_preserves_tested_merge_commit_identity() {
    let directory = repo_root().join(format!(
        "target/report-shallow-checkout-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    let git = |root: &Path, args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .env("GIT_AUTHOR_NAME", "Report fixture")
            .env("GIT_COMMITTER_NAME", "Report fixture")
            .env("GIT_AUTHOR_EMAIL", "report@example.invalid")
            .env("GIT_COMMITTER_EMAIL", "report@example.invalid")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git fixture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    };
    git(
        &directory,
        &["init", "--quiet", "--initial-branch=main", "source"],
    );
    let source = directory.join("source");
    let tree = git(&source, &["mktree"]);
    let first = git(
        &source,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit-tree",
            &tree,
            "-m",
            "first",
        ],
    );
    let second = git(
        &source,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit-tree",
            &tree,
            "-p",
            &first,
            "-m",
            "second",
        ],
    );
    let merge = git(
        &source,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit-tree",
            &tree,
            "-p",
            &first,
            "-p",
            &second,
            "-m",
            "merge",
        ],
    );
    git(&source, &["update-ref", "refs/heads/main", &merge]);
    assert!(checkout_is_merge(&source));
    let url = format!("file://{}", source.display());
    git(
        &directory,
        &["clone", "--quiet", "--depth=1", &url, "shallow"],
    );
    let shallow = directory.join("shallow");
    assert_eq!(
        git(&shallow, &["rev-parse", "--is-shallow-repository"]),
        "true"
    );
    let detected = checkout_is_merge(&shallow);
    fs::remove_dir_all(directory).unwrap();
    assert!(
        detected,
        "CI's shallow checkout still contains the merge commit's parent headers"
    );
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
        rows: Vec::new(),
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
        assert_eq!(
            result["input"]["group"],
            serde_json::Value::Null,
            "generic fixture cases carry no group"
        );
    }
    let diff = Pixmap::load_png(directory.join("different-shape-diff.png")).unwrap();
    assert_eq!((diff.width(), diff.height()), (2, 2));
    let magenta = PremultipliedColorU8::from_rgba(255, 0, 255, 255);
    assert_eq!(diff.pixel(1, 0), magenta);
    assert_eq!(diff.pixel(0, 1), magenta);
    assert_eq!(diff.pixel(0, 0).unwrap().alpha(), 0);

    let html = fs::read_to_string(directory.join("index.html")).unwrap();
    assert!(
        !html.contains("{{"),
        "every template placeholder must be filled"
    );
    assert!(html.contains("data-cases=\"2\""));
    assert!(html.contains("data-failures=\"2\""));
    assert_eq!(html.matches("data-status=\"fail\"").count(), 2);
    let metadata: Value =
        serde_json::from_slice(&fs::read(directory.join("metadata.json")).unwrap()).unwrap();
    assert_eq!(metadata["cases"], 2);
    assert_eq!(metadata["failures"], 2);
    assert!(metadata["source"]["commit_sha"].is_string());
    assert!(matches!(
        metadata["source"]["kind"].as_str(),
        Some("local" | "ci")
    ));
    assert_eq!(
        metadata["groups"][0]["name"].as_str(),
        Some("Compatibility"),
        "ungrouped cases report as the compatibility group"
    );
    fs::remove_dir_all(directory).unwrap();
}
