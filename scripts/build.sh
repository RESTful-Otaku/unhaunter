#!/bin/bash
set -euo pipefail

echo "🏗️ Building Unhaunter Game..."
echo "=============================="

# Build the main game in release mode
echo "Building release binary..."
cargo build --release --bin unhaunter_game

# Verify the binary was created
if [ -f "target/release/unhaunter_game" ]; then
    echo "✅ Build successful!"
    echo "Binary size: $(du -h target/release/unhaunter_game | cut -f1)"
    echo "Binary location: target/release/unhaunter_game"
else
    echo "❌ Build failed - binary not found"
    exit 1
fi

echo ""
echo "🎯 Build completed successfully!"
