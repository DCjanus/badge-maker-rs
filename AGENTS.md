# Agent Guide

## Compatibility

- Treat upstream `badge-maker` as the compatibility reference.
- Prefer final rendered output parity over source-level SVG parity unless the
  task is explicitly about SVG text semantics.
- Do not add compatibility surface that only exists to mirror the JavaScript
  package shape when it does not improve this crate.

## Testing

- Use `just test` as the default full-suite entry point unless there is a clear
  reason to run something narrower first.
- Prefer upstream-driven integration tests over implementation-coupled unit
  tests.
- Keep generated comparison artifacts out of version control.

## External Sources

- Clone third-party repositories used for reading or comparison into
  `.references/` at the repository root.
- `.references/` is for local research only and must not be committed.
- Reuse existing checkouts in `.references/` before cloning again.
- If upstream files, data, or derived content are brought into this repository,
  record the source nearby in documentation, comments, or another explicit
  location.
