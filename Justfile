# Simple commands

test:
    cargo test -p uncore --lib -- --skip ghost_setfinder

build:
    cargo build --release

run:
    cargo run

fmt:
    cargo fmt --all