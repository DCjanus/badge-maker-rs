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
cargo run --example render_svg > badge.svg
```

## API Notes

- The public API is intentionally Rust-first and targets rendered badge output
  compatibility, not full JavaScript package API parity.
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

The test suite is upstream-driven and prioritizes final rendered output, with
pixel parity used as the main compatibility gate. For contributor workflow and
test layout details, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Reference Projects

- [Shields.io](https://github.com/badges/shields)
- [badge-maker](https://github.com/badges/shields/tree/master/badge-maker)
- [anafanafo](https://github.com/metabolize/anafanafo)

## License

MIT. See [LICENSE](LICENSE).
