# anafanafo-ref

Minimal Bun-based reference runner for upstream `anafanafo`.

It exists only to support Rust integration tests in this repository:
- Rust sends a batch of test cases over `stdin`
- Bun evaluates them with npm package `anafanafo`
- Bun writes a batch of JSON results to `stdout`

Reference package:
- https://www.npmjs.com/package/anafanafo

This tool should stay small, explicit, and lockfile-driven.
