default:
    just check

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    just test

fix:
    cargo fmt --all
    cargo fix --allow-dirty --all-targets --all-features

test:
    bun install --cwd tools/js-ref --frozen-lockfile
    cargo nextest run --config-file .config/nextest.toml --cargo-quiet --status-level fail --final-status-level fail --failure-output final --success-output never --show-progress none --no-tests pass
    cargo test --doc --quiet
