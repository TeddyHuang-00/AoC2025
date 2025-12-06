format:
    cargo +nightly fmt --all
    cargo autoinherit
    cargo sort --workspace
    cargo sort-derives

check: format
    cargo +nightly check --all --all-targets --workspace
    cargo clippy --all --all-targets --workspace -- -D warnings

test DAY:
    cargo test -p day{{DAY}} -- --nocapture

run DAY:
    cargo run -r -p day{{DAY}}