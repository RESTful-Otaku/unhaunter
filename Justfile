# Minimal Justfile for Unhaunter development

# Default recipe - run simple tests
default: test-simple

# Run simple tests (CI-safe, no ALSA issues)
test-simple:
    ./scripts/test_simple.sh

# Run all tests (full local testing)
test:
    ./scripts/test.sh

# Run CI-safe tests
test-ci:
    ./scripts/test_ci.sh

# Check code quality
check:
    cargo fmt --all -- --check
    cargo clippy -p uncore --lib -- -D warnings
    ./scripts/test_simple.sh

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

# Quick development cycle (ALSA-free)
dev: fmt test-simple