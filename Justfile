default:
    just check

js-ref-install:
    bun install --cwd tools/js-ref --frozen-lockfile

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    just test

fix:
    cargo fmt --all
    cargo fix --allow-dirty --all-targets --all-features

test:
    just js-ref-install
    cargo nextest run --config-file .config/nextest.toml --cargo-quiet --status-level fail --final-status-level fail --failure-output final --success-output never --show-progress none --no-tests pass
    cargo test --doc --quiet
