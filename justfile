format:
    cargo +nightly fmt --all
    cargo autoinherit
    cargo sort --workspace
    cargo sort-derives

check: format
    cargo check --all --all-targets --workspace
    cargo clippy --all-targets --all-features

fix: format && format
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

test DAY:
    cargo test -p day{{DAY}} -- --nocapture

run DAY:
    cargo run -r -p day{{DAY}}

new DAY: && format
    -rm -rf day{{DAY}}
    touch inputs/day{{DAY}}.txt inputs/day{{DAY}}-example.txt
    cargo new day{{DAY}} --bin
    cat template/main.rs | sed "s/000000/{{DAY}}/g" > day{{DAY}}/src/main.rs
    cargo add -p day{{DAY}} anyhow util