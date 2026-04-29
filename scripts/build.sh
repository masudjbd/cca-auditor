#!/bin/bash
set -e

echo "🔨 CCAudit Production Build"

# Determine target
TARGET=${1:-}

if [ -z "$TARGET" ]; then
    echo "Usage: ./scripts/build.sh [universal-apple-darwin|x86_64-apple-darwin|x86_64-pc-windows-gnu|x86_64-unknown-linux-gnu]"
    exit 1
fi

# Validate target
case "$TARGET" in
    universal-apple-darwin|x86_64-apple-darwin|x86_64-pc-windows-gnu|x86_64-unknown-linux-gnu)
        ;;
    *)
        echo "❌ Unknown target: $TARGET"
        exit 1
        ;;
esac

echo "📦 Building for target: $TARGET"
echo ""

# Check prerequisites
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Install from https://rustup.rs"
    exit 1
fi

# Install target if needed
echo "📥 Ensuring Rust target is installed..."
rustup target add "$TARGET"

# Run checks
echo ""
echo "✅ Running checks..."
cargo check --workspace --release
cargo clippy --workspace -- -D warnings

# Build
echo ""
echo "🔨 Building release binary..."
cd apps/desktop
cargo tauri build --target "$TARGET"

echo ""
echo "✅ Build complete!"
echo "   Binary: apps/desktop/src-tauri/target/$TARGET/release/bundle/"
