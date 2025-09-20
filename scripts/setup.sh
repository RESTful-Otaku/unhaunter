#!/bin/bash
set -euo pipefail

echo "🚀 Setting up Unhaunter Development Environment..."
echo "=================================================="

# Install Rust components
echo "1. Installing required Rust components..."
rustup component add rustfmt clippy llvm-tools-preview

# Install additional tools if needed
echo ""
echo "2. Installing additional tools..."
if ! command -v just &> /dev/null; then
    echo "   Installing 'just' command runner..."
    cargo install just
else
    echo "   ✅ 'just' already installed"
fi

# Make scripts executable
echo ""
echo "3. Making scripts executable..."
chmod +x scripts/*.sh
echo "   ✅ All scripts are now executable"

# Run initial checks
echo ""
echo "4. Running initial quality checks..."
./scripts/lint.sh

echo ""
echo "5. Running initial tests..."
./scripts/test.sh

echo ""
echo "6. Testing build..."
./scripts/build.sh

echo ""
echo "🎯 Setup Complete!"
echo "=================="
echo ""
echo "Available commands:"
echo "  ./scripts/build.sh  - Build the game"
echo "  ./scripts/test.sh   - Run tests"
echo "  ./scripts/lint.sh   - Run quality checks"
echo "  just dev            - Quick development cycle"
echo "  just test           - Run tests"
echo "  just build          - Build release"
echo ""
echo "🎉 Ready for development!"
