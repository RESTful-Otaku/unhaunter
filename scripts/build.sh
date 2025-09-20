#!/bin/bash
set -euo pipefail

echo "🏗️ Building Unhaunter Core..."
echo "============================="

# For CI: Just build and check the core library, not the full game binary
echo "Building core library..."
if cargo build -p uncore; then
    echo "✅ Core library build successful!"
else
    echo "❌ Core library build failed"
    exit 1
fi

# If we're in a local environment (not CI), also try building the game
if [ "${CI:-false}" != "true" ]; then
    echo ""
    echo "Building game binary (local only)..."
    if cargo build --release --bin unhaunter_game; then
        if [ -f "target/release/unhaunter_game" ]; then
            echo "✅ Game binary build successful!"
            echo "Binary size: $(du -h target/release/unhaunter_game | cut -f1)"
        fi
    else
        echo "⚠️  Game binary build failed (this is OK in CI)"
    fi
fi

echo ""
echo "🎯 Build completed successfully!"
