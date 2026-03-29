## Branch Policy

- Unless the user explicitly says otherwise, pull requests for ongoing
  development should target `develop`, not `master`.
- The long-lived development branch is currently `develop`. If the user names a
  different branch later, follow the user instruction.
- The default feature workflow is:
  - branch from `develop`
  - merge back into `develop`
  - avoid merging directly into `master` until the release line is ready
- Before merging release-ready work from the development branch into `master`,
  remove remaining Chinese user-facing content and keep public documentation,
  comments, and release-facing text in English.

## External References

- Clone third-party repositories used for reading or comparison into
  `.references/` at the repository root.
- `.references/` is for local research only and must not be committed.
- Reuse existing checkouts in `.references/` before cloning again.
- If upstream files, data, or derived content are brought into this repository,
  record the source nearby in documentation, comments, or another explicit
  location.

## Testing Strategy

- Prefer integration-style tests over a large number of implementation-coupled
  unit tests.
- Use upstream reference coverage whenever the behavior can be compared against
  `badge-maker`.
- `public_api_contract` should focus on Rust-specific API semantics and on text,
  link, or error behavior that raster comparison does not protect well.
- Keep cases that require exact SVG text comparison in dedicated test files
  instead of forcing them into `tests/data/badge_maker_cases.json`.
- Use `just test` as the default full-suite entry point, and keep local and CI
  behavior aligned.
- JavaScript reference execution should continue to use Bun with JSON exchanged
  over `stdin` and `stdout`.
- Do not commit generated comparison outputs. Generate reference results at test
  time from locked JavaScript dependencies.
- When verification is stable, trust rasterized output parity over raw SVG text
  parity.

## Engineering Direction

- Prefer maintainability, clarity, and long-term architecture over short-term
  speed.
- Do not introduce hacks, brittle special cases, or designs that push
  complexity downward just to land a quick fix.
- If the current architecture blocks a natural implementation, refactor the
  abstraction rather than stacking more exceptions on top.
- Prefer small, explicit repository conventions over hidden machine-local
  configuration.
- Keep [README.md](README.md) up to date as the project evolves.
- Do not preserve compatibility layers that only exist to mirror the upstream
  JavaScript package shape when they do not serve this crate's goals.
