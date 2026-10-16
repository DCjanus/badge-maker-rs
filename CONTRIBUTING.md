# Contributing

## Prerequisites

- Rust stable with Cargo
- `just`
- `cargo-nextest`
- Bun

If you prefer not to install `just`, run the commands from [Justfile](Justfile)
directly.

## Development Workflow

- Run the full local suite with `just test`.
- Keep user-facing documentation, comments, and release-facing text in English.
- Follow [`RELEASING.md`](RELEASING.md) when preparing or publishing a release.

## Minimum Supported Rust Version

Dependency refreshes also roll the minimum supported Rust version forward to
the latest stable release that was available one year before the update date.
This keeps an approximately one-year compatibility window without permanently
pinning the crate to an old compiler. Update `package.rust-version` and the MSRV
CI toolchain together.

## Test Layout

This repository intentionally uses a small number of integration-style test
layers instead of many implementation-coupled unit tests.

- `tests/integration.rs`
  Single integration-test crate entry point. Shared helpers live in
  `tests/support/mod.rs`, and test modules are grouped under `tests/cases/`.
- `tests/cases/badge_maker_reference.rs`
  Data-driven upstream reference tests. Cases from
  `tests/data/badge_maker_cases.json` are rendered by both Rust and the
  upstream `badge-maker` package, then compared by rasterized pixels, and the
  run also writes the comparison report described below. Cases from
  `tests/data/deps_rs_cases.json` join the same comparison in the optional
  `deps-rs` group (see [`tests/data/README.md`](tests/data/README.md)). This is
  the default home for cases that can be validated by final rendered output.
- `tests/cases/svg_text_reference.rs`
  Focused upstream reference tests for cases where exact SVG text matters more
  than pixel output, such as escaping or source-level fallback behavior.
- `tests/cases/reference_error_semantics.rs`
  Focused upstream reference tests for cases where Rust should match useful
  upstream error semantics.
- `tests/cases/public_api_contract.rs`
  Rust API contract tests. Use this file for Rust-specific behavior that cannot
  be validated through upstream reference execution, and for semantics that are
  not well protected by pixel comparison alone, such as link structure or
  Rust-only option handling.
- `tests/cases/docs_asset_consistency.rs`
  Verifies that tracked documentation preview assets still match the current
  renderer output.

## Where New Tests Should Go

- If a case can be compared against upstream `badge-maker` by final rendered
  output, add it to `tests/data/badge_maker_cases.json`.
- If a case needs exact SVG source comparison, keep it in a dedicated
  reference test file instead of forcing it into the data-driven pixel suite.
- If a case is about Rust-only API behavior, typed parsing, or link and error
  semantics that upstream reference rendering does not cover well, put it in
  `tests/cases/public_api_contract.rs`.
- If a case is about tracked rustdoc assets, keep it in
  `tests/cases/docs_asset_consistency.rs`.

## Reference Runner Notes

- Upstream reference execution is routed through Bun in
  [tools/js-ref](tools/js-ref/README.md).
- Reference fixtures should describe behavior, not checked-in generated output.
- Prefer pixel parity over byte-for-byte SVG parity unless the test is
  explicitly about SVG text semantics.
- Raster comparisons require identical width, height, and RGBA pixels. They use
  the bundled DejaVu Sans regular/bold fonts in `tests/data/fonts`, not fonts
  installed on the host. This verifies both renderers under the same font
  environment, rather than promising identical pixels across all browsers.
- Logo fixtures use a self-contained red square with a white cross. The
  intentionally truncated SVG remains only in explicitly named malformed-logo
  cases. A separate check requires normal logo fixtures to produce visible pixels.
- `just test` writes `target/reference-report/index.html`, paired SVG/PNG files,
  magenta pixel-difference images, input/results JSON, and run metadata. The PNGs
  are the actual pixels compared by the test; viewing an SVG in a browser may
  substitute different fonts. CI uploads this directory as `badge-reference-report`
  even when comparisons fail. Download and extract the artifact, then open
  `index.html`. Generated files stay outside version control.
- `index.html` is a single self-contained page: the comparison PNGs, the SVG
  sources, and the JSON/lock evidence are embedded, so the report keeps working
  when it is opened on its own. The raw PNG/SVG/JSON files above stay next to it
  as machine-readable evidence. Typography falls back to system fonts when the
  optional Google Fonts stylesheet cannot be loaded.
- The report shows one compact row per case (`Case`, `Rust`, `Upstream`,
  `Difference`); badges render at native size with optional 1x/2x/4x pixelated
  zoom, and matching rows omit the difference image. Rows are grouped by the
  optional `group` field of a case: generic fixtures have no group and appear
  under `Compatibility`, while `"group": "deps-rs"` renders as `deps.rs`. Each
  group has a collapsible header row, the toolbar selects a group, and
  `index.html#group=deps-rs` deep-links to one group. Clicking a case id expands
  that case's input JSON, dimensions, and SVG downloads; the top-right info
  button holds provenance (CI run attempt, tested commit, pull request),
  run metadata, and the embedded evidence downloads. The layout assets live in
  `tests/support/report/` (`page.html`, `report.css`, `report.js`) and are filled
  by `tests/support/report.rs`, with provenance derived in
  `tests/support/report/source.rs`.
- The `group` field is report metadata only: it is not passed to either renderer
  and does not relax the zero-tolerance pixel or dimension requirements for the
  cases in that group.
