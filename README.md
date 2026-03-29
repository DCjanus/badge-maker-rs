# badge-maker-rs

`badge-maker-rs` is a Rust badge renderer for
[Shields.io](https://shields.io)-style badges, built to stay visually aligned
with the upstream
[`badge-maker`](https://github.com/badges/shields/tree/master/badge-maker)
renderer and enforced by upstream-backed reference tests.

## Features

- Built for rendered-output parity with upstream `badge-maker`, with
  compatibility judged by what the badge actually looks like
- Backed by a broad upstream reference suite that continuously checks the Rust
  renderer against the JavaScript implementation

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Reference Projects

- [Shields.io](https://github.com/badges/shields)
- [badge-maker](https://github.com/badges/shields/tree/master/badge-maker)
- [anafanafo](https://github.com/metabolize/anafanafo)

## License

MIT. See [LICENSE](LICENSE).
