set shell := ["fish", "-c"]

# See tracking issue: https://github.com/casey/just/issues/2986
# format string in backticks are not yet supported, so we use shell commands instead.

LATEST := ```
fd -t d -E target "day*" . \
| sed 's/.*day\([0-9]*\).*/\1/' \
| sort -n \
| tail -n 1 \
| awk -v start="00" '{ print } END { if ( NR == 0 ) { print start } }'
```
NEXT := ```
fd -t d -E target "day*" . \
| sed 's/.*day\([0-9]*\).*/\1/' \
| sort -n | tail -n 1 \
| awk -v start="00" '{ print } END { if ( NR == 0 ) { print start } }' \
| awk '{ printf "%02d", $1 + 1 }'
```

_default:
    @just --choose

[doc("Format all code and sort Cargo.toml files")]
[group("housekeeping")]
format:
    cargo +nightly fmt --all
    cargo autoinherit --prefer-simple-dotted
    cargo sort --workspace
    cargo sort-derives
    just --fmt --unstable

[doc("Run all checks including type checking, linting, and typo checking")]
[group("housekeeping")]
check: format
    typos **/*.rs
    cargo check --all --all-targets --workspace
    cargo clippy --all-targets --all-features

[doc("Fix lint warnings automatically (safely)")]
[group("housekeeping")]
fix: format && format
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

[doc("Run test coverage on library crates")]
[group("housekeeping")]
coverage:
    cargo tarpaulin --out Stdout --lib -p util --workspace -e day* --exclude-files day*/**

[doc("Run tests for a specific day's puzzle with example input")]
[group("puzzle")]
test DAY=LATEST:
    cargo test -p day{{ DAY }} "test_" -- --no-capture

[doc("Run the solution for a specific day's puzzle with actual input")]
[group("puzzle")]
run DAY=LATEST:
    cargo run -r -p day{{ DAY }}

[doc("Run the benchmark for a specific day's puzzle and record performance")]
[group("puzzle")]
bench DAY=LATEST:
    cargo test -r -p day{{ DAY }} "benchmark" -- --no-capture

[doc("Create a new day's puzzle scaffold")]
[group("puzzle")]
new DAY=NEXT: && format
    -rm -rf day{{ DAY }}
    mkdir -p inputs
    touch inputs/day{{ DAY }}{,-example}.txt
    cargo new day{{ DAY }} --bin --vcs none
    cat template/main.rs \
    | awk -v day="{{ NEXT }}" 'BEGIN { d = int(day) } { gsub("000000", d); print }' \
    > day{{ DAY }}/src/main.rs
    cargo add -p day{{ DAY }} anyhow util
