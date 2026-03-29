# badge-maker-rs

`badge-maker-rs` is a Rust badge renderer designed to match
[Shields.io](https://shields.io) badge output.

It follows the rendered output of the upstream
[`badge-maker`](https://github.com/badges/shields/tree/master/badge-maker)
implementation and validates compatibility with upstream-backed reference
tests.

## Features

- Covers all five official `badge-maker` styles: `plastic`, `flat`,
  `flat-square`, `for-the-badge`, and `social`
- Includes upstream-backed regression tests against the JavaScript renderer
- Treats final raster parity as a higher priority than byte-for-byte SVG parity
- Compiles upstream `anafanafo` width-table data into Rust static tables at
  build time
- Exposes a Rust-first API instead of mirroring the JavaScript package surface

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
cargo run --example render_svg > badge.svg
```

## API

- The public API is intentionally Rust-first and does not aim to reproduce the
  full JavaScript package surface.
- `message` is required through `BadgeOptions::new(message)`. `label` and
  `message` are trimmed before layout, and emitted text is XML-escaped.
- Omitted `color` and `label_color` follow Shields defaults. Invalid
  `Color::literal(...)` input falls back to style defaults instead of
  returning an error.
- `left_link` and `right_link` model link structure directly, while
  `logo_data_url`, `logo_width`, and `id_suffix` map to the upstream
  `logoBase64`, `logoWidth`, and `idSuffix` concepts.
- `id_suffix` is currently the only public input that returns a dedicated
  validation error.

## Testing

Run the full local suite with:

```sh
just test
```

The test suite is upstream-driven and uses final rendered output as the main
compatibility gate. Contributor workflow and test layout details are documented
in [CONTRIBUTING.md](CONTRIBUTING.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Reference Projects

- [Shields.io](https://github.com/badges/shields)
- [badge-maker](https://github.com/badges/shields/tree/master/badge-maker)
- [anafanafo](https://github.com/metabolize/anafanafo)

## License

MIT. See [LICENSE](LICENSE).
