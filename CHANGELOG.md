# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-09-11

### Changed

- **Breaking:** Align named colors, default badge colors, and Flat/Plastic SVG
  shadow structure with the renderer deployed by Shields.io and represented by
  the verified `badge-maker` 6.0.0 fixture.
- **Breaking:** Raise the minimum supported Rust version from 1.85 to 1.89.
- Refresh project dependencies and adopt a rolling one-year MSRV policy.

### Fixed

- Apply `id_suffix` to Flat/Plastic blur filter IDs and references, preserving
  collision-free inline SVG output.

### Maintenance

- Check Cargo, Bun, and GitHub Actions dependencies daily, group patch updates,
  and keep `badge-maker` updates standalone for compatibility review.

## [0.1.0] - 2026-03-29

### Added

- Initial release of the Shields-compatible Rust badge renderer.

[Unreleased]: https://github.com/DCjanus/badge-maker-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/DCjanus/badge-maker-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/DCjanus/badge-maker-rs/releases/tag/v0.1.0
