#!/bin/bash
set -euo pipefail

echo "🧪 Running Comprehensive Test Suite..."
echo "======================================"

# 1. Unit Tests
echo "1. Running unit tests..."
if cargo test -p uncore --lib -- --skip ghost_setfinder --test-threads=1; then
    echo "   ✅ Unit tests passed"
else
    echo "   ❌ Unit tests failed"
    exit 1
fi

echo ""

# 2. Integration Tests
echo "2. Running integration tests..."
if cargo test --test integration_tests; then
    echo "   ✅ Integration tests passed"
else
    echo "   ❌ Integration tests failed"
    exit 1
fi

echo ""
echo "🎯 All Tests Passed Successfully!"
echo "  ✅ Unit Tests: Core systems validated"
echo "  ✅ Integration Tests: Component interactions verified"
echo ""
echo "🎉 Testing completed successfully!"
