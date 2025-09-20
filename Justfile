# Simple commands

# Test core logic
test:
    cargo test -p uncore --lib -- --skip ghost_setfinder

# Test everything in parallel
test-all:
    #!/bin/bash
    cargo test -p uncore --lib -- --skip ghost_setfinder --test-threads=1 &
    cargo test --test integration_tests &
    wait

# Format and lint
check:
    cargo fmt --all
    cargo clippy -p uncore --lib -- -D warnings

# Build game
build:
    cargo build --release

# Run game
run:
    cargo run

# Full CI simulation
ci: check test-all