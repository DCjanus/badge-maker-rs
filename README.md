# badge-maker-rs

[![CI](https://github.com/DCjanus/badge-maker-rs/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/DCjanus/badge-maker-rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/DCjanus/badge-maker-rs/graph/badge.svg?token=c5f8nWZBDh)](https://codecov.io/gh/DCjanus/badge-maker-rs)
[![Crates.io](https://img.shields.io/crates/v/badge-maker-rs.svg)](https://crates.io/crates/badge-maker-rs)
[![Docs.rs](https://img.shields.io/docsrs/badge-maker-rs)](https://docs.rs/badge-maker-rs)
[![dependency status](https://deps.rs/crate/badge-maker-rs/latest/status.svg)](https://deps.rs/crate/badge-maker-rs/latest)

`badge-maker-rs` is a Rust badge renderer for
[Shields.io](https://shields.io)-style badges, built to stay visually aligned
with the upstream
[`badge-maker`](https://github.com/badges/shields/tree/master/badge-maker)
renderer and enforced by upstream-backed reference tests.

The compatibility target is the renderer used by the deployed Shields.io
service. The pinned `badge-maker` package is used as a reproducible test fixture
only after its rendering sources are verified against that upstream version;
an npm release alone does not define compatibility.

The current baseline is `badge-maker` 6.0.0 from
[`badges/shields@b6dd9db`](https://github.com/badges/shields/commit/b6dd9db77a37580d6c0f65dc6d88fe2080032cc1).
Its rendering sources match the latest Shields.io deployment tag checked for
this update, `server-2026-08-21`.

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
