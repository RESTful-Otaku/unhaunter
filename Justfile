# Simple commands

# Run core tests
test:
    cargo test -p uncore --lib -- --skip ghost_setfinder

# Run all tests (unit + integration)
test-all:
    cargo test -p uncore --lib -- --skip ghost_setfinder
    cargo test --test integration_tests

# Build game
build:
    cargo build --release

# Run game
run:
    cargo run

# Format code
fmt:
    cargo fmt --all

# Quick check (format + tests)
check: fmt test