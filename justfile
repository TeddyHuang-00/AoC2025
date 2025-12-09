LATEST := `fd -t d "day*" . | sed 's/.*day\([0-9]*\).*/\1/' | sort -n | tail -n 1`
NEXT := `fd -t d "day*" . | sed 's/.*day\([0-9]*\).*/\1/' | sort -n | tail -n 1 | awk '{printf "%02d", $1 + 1}'`

_default:
    @just --choose

[group("housekeeping")]
format:
    cargo +nightly fmt --all
    cargo autoinherit --prefer-simple-dotted
    cargo sort --workspace
    cargo sort-derives
    just --fmt --unstable

[group("housekeeping")]
check: format
    typos **/*.rs
    cargo check --all --all-targets --workspace
    cargo clippy --all-targets --all-features

[group("housekeeping")]
fix: format && format
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

[group("housekeeping")]
coverage:
    cargo tarpaulin --lib --out Stdout

[group("puzzle")]
test DAY=LATEST:
    cargo test -p day{{ DAY }} -- --no-capture

[group("puzzle")]
run DAY=LATEST:
    cargo run -r -p day{{ DAY }}

[group("puzzle")]
new DAY=NEXT: && format
    -rm -rf day{{ DAY }}
    touch inputs/day{{ DAY }}.txt inputs/day{{ DAY }}-example.txt
    cargo new day{{ DAY }} --bin
    cat template/main.rs | sed "s/000000/{{ DAY }}/g" > day{{ DAY }}/src/main.rs
    cargo add -p day{{ DAY }} anyhow util
