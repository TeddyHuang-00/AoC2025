format:
    cargo +nightly fmt --all
    cargo autoinherit
    cargo sort --workspace
    cargo sort-derives

check: format
    cargo +nightly check --all --all-targets --workspace
    cargo clippy --all-targets --all-features

fix: format && format
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

test DAY:
    cargo test -p day{{DAY}} -- --nocapture

run DAY:
    cargo run -r -p day{{DAY}}