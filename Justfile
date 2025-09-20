# Simple Justfile for Unhaunter

# Run tests
test:
    cargo test -p uncore --lib -- --skip ghost_setfinder

# Format code
fmt:
    cargo fmt --all

# Build game
build:
    cargo build --release

# Run game
run:
    cargo run --bin unhaunter_game

# Quick check
check: fmt test