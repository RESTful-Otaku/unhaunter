#!/bin/bash
set -euo pipefail

echo "🧪 Simple Local Testing"
echo "======================"

# 1. Format check
echo "1. Format check..."
cargo fmt --all -- --check
echo "   ✅ Format OK"

# 2. Core library tests only
echo ""
echo "2. Core library tests..."
cargo test -p uncore --lib -- --skip ghost_setfinder --quiet
echo "   ✅ Core tests OK"

# 3. Integration tests
echo ""
echo "3. Integration tests..."
cargo test --test integration_tests --quiet
echo "   ✅ Integration OK"

echo ""
echo "🎉 All essential tests passed!"
echo "💡 For full system testing, use: ./scripts/test.sh"
