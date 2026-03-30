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

## Test Layout

This repository intentionally uses a small number of integration-style test
layers instead of many implementation-coupled unit tests.

- `tests/integration.rs`
  Single integration-test crate entry point. Shared helpers live in
  `tests/support/mod.rs`, and test modules are grouped under `tests/cases/`.
- `tests/cases/badge_maker_reference.rs`
  Data-driven upstream reference tests. Cases from
  `tests/data/badge_maker_cases.json` are rendered by both Rust and the
  upstream `badge-maker` package, then compared by rasterized pixels. This is
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
