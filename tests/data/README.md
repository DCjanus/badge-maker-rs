# Rendering scenarios

`badge_maker_cases.json` contains the general upstream compatibility matrix.
`deps_rs_cases.json` contains a small visual showcase of badge inputs from the deps.rs
renderer migration in [deps-rs/deps.rs#291](https://github.com/deps-rs/deps.rs/pull/291).
Both files run through the same Rust/upstream pixel comparison, with no
consumer-specific allowances.

## deps.rs examples

Inputs are derived from these pinned sources at
[`7ff1f53427d78a2e7e0ea0bd1af3648954b00150`](https://github.com/deps-rs/deps.rs/commit/7ff1f53427d78a2e7e0ea0bd1af3648954b00150):

- [Badge messages and colors](https://github.com/deps-rs/deps.rs/blob/7ff1f53427d78a2e7e0ea0bd1af3648954b00150/src/server/views/badge.rs)
- [Default, compact, and custom subjects](https://github.com/deps-rs/deps.rs/blob/7ff1f53427d78a2e7e0ea0bd1af3648954b00150/src/server/mod.rs)

The 12 display cases show:

- Six common states in the default flat style: unknown, none, up to date,
  insecure, maybe insecure, and 3 of 18 outdated.
- Compact `deps` labels for up-to-date and outdated states.
- An up-to-date badge in the four other styles.

This group is deliberately small so readers can scan the expected deps.rs
appearance quickly. Broad compatibility coverage belongs to the general matrix;
the display group does not expand every combination of states and styles.

These are renderer input fixtures, not an execution of deps.rs analysis or
query parsing. End-to-end validation of that application belongs in its own
repository. They describe the pinned migration PR, not the historical renderer
deployed before that change.

## Report groups

The optional `group` field is report metadata; it is not passed to either
renderer. Cases without a group appear under Compatibility. The deps.rs
examples use `"group": "deps-rs"`. All groups retain the same zero-tolerance
dimension and pixel requirements.
