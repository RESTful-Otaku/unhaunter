#!/bin/bash
set -euo pipefail

echo "🧪 Running Comprehensive Test Suite..."
echo "======================================"

# 1. Unit Tests
echo "1. Running unit tests..."
if cargo test -p uncore --lib -- --skip ghost_setfinder --quiet --test-threads=1; then
    UNIT_COUNT=$(cargo test -p uncore --lib -- --skip ghost_setfinder --quiet 2>&1 | grep "test result:" | grep -o '[0-9]\+ passed' | cut -d' ' -f1 || echo "0")
    echo "   ✅ Unit tests: $UNIT_COUNT passed"
else
    echo "   ❌ Unit tests failed"
    exit 1
fi

echo ""

# 2. Integration Tests
echo "2. Running integration tests..."
if cargo test --test integration_tests --quiet; then
    INTEGRATION_COUNT=$(cargo test --test integration_tests --quiet 2>&1 | grep "test result:" | grep -o '[0-9]\+ passed' | cut -d' ' -f1 || echo "0")
    echo "   ✅ Integration tests: $INTEGRATION_COUNT passed"
else
    echo "   ❌ Integration tests failed"
    exit 1
fi

echo ""

# 3. Property-based Tests (subset of unit tests, but specifically call them out)
echo "3. Running property-based tests..."
if cargo test -p uncore --lib prop_ -- --quiet --test-threads=1; then
    PROP_COUNT=$(cargo test -p uncore --lib prop_ -- --quiet 2>&1 | grep "test result:" | grep -o '[0-9]\+ passed' | cut -d' ' -f1 || echo "0")
    echo "   ✅ Property-based tests: $PROP_COUNT passed"
else
    echo "   ❌ Property-based tests failed"
    exit 1
fi

echo ""

# 4. Performance Tests (subset of unit tests)
echo "4. Running performance tests..."
if cargo test -p uncore --lib performance -- --quiet --test-threads=1; then
    PERF_COUNT=$(cargo test -p uncore --lib performance -- --quiet 2>&1 | grep "test result:" | grep -o '[0-9]\+ passed' | cut -d' ' -f1 || echo "0")
    echo "   ✅ Performance tests: $PERF_COUNT passed"
else
    echo "   ❌ Performance tests failed"
    exit 1
fi

echo ""
echo "🎯 Test Summary:"
echo "  ✅ Unit Tests: $UNIT_COUNT passed"
echo "  ✅ Integration Tests: $INTEGRATION_COUNT passed"
echo "  ✅ Property-based Tests: $PROP_COUNT passed"
echo "  ✅ Performance Tests: $PERF_COUNT passed"
echo "  🚀 All test categories validated"
echo ""
echo "🎉 Comprehensive testing completed successfully!"
