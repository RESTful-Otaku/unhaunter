# Minimal Justfile for Unhaunter development

# Default recipe - run tests
default: test

# Run all tests
test:
    cargo test -p uncore --lib -- --skip ghost_setfinder

# Check code quality
check:
    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    cargo test -p uncore --lib -- --skip ghost_setfinder

# Format code
fmt:
    cargo fmt --all

# Build release
build:
    cargo build --release

# Run the game
run:
    cargo run --bin unhaunter_game

# Clean build artifacts
clean:
    cargo clean

# Quick development cycle
dev: fmt check