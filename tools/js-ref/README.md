# js-ref

Shared Bun-based reference runners for upstream JavaScript packages used by this repository's Rust integration tests.

Current reference modules:
- `anafanafo`
- `badge-maker`

Design goals:
- one lockfile and one installation path for all JavaScript reference dependencies
- one stable stdin/stdout batch protocol per reference tool
- thin per-package adapters instead of duplicated runner scaffolding
- keep `package.json` and `bun.lock`, but avoid relying on a persistent local `node_modules/`

Reference packages:
- https://www.npmjs.com/package/anafanafo
- https://www.npmjs.com/package/badge-maker

This tool exists only for test-time reference execution and should stay explicit, minimal, and lockfile-driven.
