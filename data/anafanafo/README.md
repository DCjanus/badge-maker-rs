# anafanafo width tables

These JSON files are copied from the upstream `metabolize/anafanafo` project and
treated as build-time source data inside `badge-maker-rs`.

Source repository:
- https://github.com/metabolize/anafanafo

Original source paths:
- `packages/anafanafo/data/verdana-10px-normal.json`
- `packages/anafanafo/data/verdana-10px-bold.json`
- `packages/anafanafo/data/verdana-11px-normal.json`
- `packages/anafanafo/data/helvetica-11px-bold.json`

At the time of import, the local reference checkout was:
- commit `2c1de81564707678eb17f0f0c80d1df03cbbd402`

Upstream license:
- MIT License

These files are compiled into generated Rust static tables by `build.rs` to
reproduce upstream `anafanafo` text-width behavior without runtime JSON
parsing. They should be updated deliberately, with provenance retained.
