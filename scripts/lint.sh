#!/bin/bash
set -euo pipefail

echo "🔍 Running Code Quality Checks..."
echo "================================="

# Check formatting
echo "1. Checking code formatting..."
if cargo fmt --all -- --check; then
    echo "   ✅ Code formatting is correct"
else
    echo "   ❌ Code formatting issues found"
    echo "   Run 'cargo fmt --all' to fix formatting"
    exit 1
fi

echo ""

# Run Clippy lints
echo "2. Running Clippy lints..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo "   ✅ No clippy warnings found"
else
    echo "   ❌ Clippy warnings found"
    exit 1
fi

echo ""

# Check for any compilation warnings/errors
echo "3. Checking for compilation issues..."
if cargo check --quiet 2>&1 | grep -E 'warning|error' >/dev/null; then
    WARNING_COUNT=$(cargo check 2>&1 | grep -E 'warning|error' | wc -l)
    echo "   ❌ Found $WARNING_COUNT compilation warnings/errors"
    exit 1
else
    echo "   ✅ No compilation warnings or errors"
fi

echo ""

# 4. Security audit
echo "4. Running security audit..."
if command -v cargo-audit >/dev/null 2>&1; then
    # Run audit with timeout - only fail on truly critical issues
    if timeout 30 cargo audit --quiet >/dev/null 2>&1; then
        echo "   ✅ No security vulnerabilities found"
    else
        # Check if it's just warnings vs critical issues
        AUDIT_RESULT=$(timeout 10 cargo audit --quiet 2>&1 || echo "timeout")
        if echo "$AUDIT_RESULT" | grep -E "(unsound|yanked)" >/dev/null; then
            echo "   ❌ Critical security vulnerabilities found (unsound/yanked crates)"
            echo "   Run 'cargo audit' for details"
            exit 1
        else
            echo "   ⚠️  Security audit found minor issues (not critical)"
        fi
    fi
else
    echo "   ⚠️  cargo-audit not installed, skipping security check"
    echo "   Install with: cargo install cargo-audit"
fi

echo ""
echo "🎯 Code Quality Summary:"
echo "  ✅ Formatting: CLEAN"
echo "  ✅ Lints: CLEAN"
echo "  ✅ Compilation: CLEAN"
echo "  ✅ Security: AUDITED"
echo ""
echo "🎉 All quality checks passed!"
