default:
    just test

test:
    rm -rf tools/js-ref/node_modules
    cargo nextest run --config-file .config/nextest.toml --cargo-quiet --status-level fail --final-status-level fail --failure-output final --success-output never --show-progress none --no-tests pass
    cargo test --doc --quiet
