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

# Cross-platform builds
build-windows:
    ./build.sh windows build

build-android:
    ./build.sh android build

build-ios:
    ./build.sh ios build

build-all:
    ./build.sh all build

# Cross-platform runs
run-windows:
    ./build.sh windows run

run-android:
    ./build.sh android run

run-ios:
    ./build.sh ios run

# Release builds
release-windows:
    ./build.sh windows release

release-android:
    ./build.sh android release

release-ios:
    ./build.sh ios release

release-all:
    ./build.sh all release

# Package artifacts for distribution
package:
    #!/bin/bash
    echo "Packaging artifacts for distribution..."
    mkdir -p dist
    cp target/release/unhaunter_game dist/
    echo "Artifacts packaged in dist/ directory"

# Platform testing
test-windows:
    ./build.sh windows test

test-android:
    ./build.sh android test

test-ios:
    ./build.sh ios test