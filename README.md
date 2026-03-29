# badge-maker-rs

`badge-maker-rs` is a Rust badge renderer that targets Shields-compatible badge
output.

The crate focuses on the thing that matters most in practice: the final badge
image. Its compatibility target is the rendered result of the upstream
[`badge-maker`](https://github.com/badges/shields/tree/master/badge-maker)
implementation, validated by upstream-backed reference tests.

## Features

- Covers all five official `badge-maker` styles: `plastic`, `flat`,
  `flat-square`, `for-the-badge`, and `social`
- Uses upstream-backed regression tests to compare Rust output against the
  upstream JavaScript renderer
- Prioritizes final raster parity over byte-for-byte SVG source parity
- Compiles upstream `anafanafo` width-table data into Rust static tables at
  build time, with no runtime JSON parsing
- Exposes a compact Rust-first API instead of mirroring the JavaScript package
  surface

## Installation

```sh
cargo add badge-maker-rs
```

## Usage

```rust
use badge_maker_rs::{BadgeOptions, Color, Style, make_badge};

let svg = make_badge(
    &BadgeOptions::new("passing")
        .label("build")
        .color("brightgreen".parse::<Color>()?)
        .style(Style::Flat)
        .build(),
)?;

assert!(svg.starts_with("<svg "));
# Ok::<(), Box<dyn std::error::Error>>(())
```

To write an SVG file from the example program:

```sh
cargo run --example write_svg > badge.svg
```

## Scope

The public API is intentionally Rust-first.

- The compatibility target is the rendered badge output, not full JavaScript
  package API parity.
- Upstream `badge-maker` remains the rendering reference.
- Node-specific entry points such as validation wrappers or JSON-oriented
  helpers are intentionally out of scope.
- Rust-side inputs such as `logo_width` are exposed as crate conveniences, not
  as promises to match the upstream package shape exactly.

Current input semantics:

- `message` is required and is provided through `BadgeOptions::new(message)`.
- `label` and `message` are trimmed before layout.
- Text and attribute content are XML-escaped before SVG emission.
- Omitted `color` and `label_color` follow Shields defaults.
- Invalid `Color::literal(...)` input falls back to style defaults instead of
  returning an error.
- `left_link` and `right_link` model the badge link structure directly:
  `left_link` alone wraps the full badge body, `right_link` alone links only
  the right half, and both together create independent left and right links.
- `logo_data_url`, `logo_width`, and `id_suffix` correspond to the upstream
  `logoBase64`, `logoWidth`, and `idSuffix` concepts.
- `id_suffix` is currently the only public input that returns a dedicated
  validation error.

## Testing

Run the full local suite with:

```sh
just test
```

The repository uses integration-heavy, upstream-driven tests rather than a
large number of implementation-coupled unit tests.

The main layers are:

- Data-driven upstream reference tests with pixel-by-pixel raster comparison
- Focused reference tests for exact SVG text semantics
- Focused reference tests for useful upstream error semantics
- Rust API contract tests for Rust-specific behavior
- Consistency checks for tracked rustdoc preview assets

Upstream JavaScript reference execution is routed through Bun in
[tools/js-ref](tools/js-ref/README.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Reference Projects

- [Shields.io](https://github.com/badges/shields)
- [badge-maker](https://github.com/badges/shields/tree/master/badge-maker)
- [anafanafo](https://github.com/metabolize/anafanafo)

## License

MIT. See [LICENSE](LICENSE).
