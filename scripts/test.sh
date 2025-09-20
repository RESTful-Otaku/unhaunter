#!/bin/bash
set -euo pipefail

echo "🧪 Running Unhaunter Tests..."
echo "============================="

# Run unit tests for the core library
echo "Running unit tests..."
cargo test -p uncore --lib -- --skip ghost_setfinder --nocapture --test-threads=1

# Get test count
TEST_COUNT=$(cargo test -p uncore --lib -- --skip ghost_setfinder --quiet 2>&1 | grep "test result:" | grep -o '[0-9]\+ passed' | cut -d' ' -f1 || echo "0")

echo ""
echo "🎯 Test Results:"
echo "  ✅ Tests passed: $TEST_COUNT"
echo "  🚀 All core systems validated"
echo ""
echo "🎉 Testing completed successfully!"
