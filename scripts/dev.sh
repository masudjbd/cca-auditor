#!/bin/bash
set -e

echo "🚀 CCAudit Development Setup"

# Check prerequisites
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Install from https://rustup.rs"
    exit 1
fi

if ! command -v pnpm &> /dev/null; then
    echo "❌ pnpm not found. Install from https://pnpm.io"
    exit 1
fi

if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found. Install from https://nodejs.org"
    exit 1
fi

echo "✅ Rust: $(cargo --version)"
echo "✅ Node: $(node --version)"
echo "✅ pnpm: $(pnpm --version)"

# Install cargo-tauri CLI
echo ""
echo "📦 Installing cargo-tauri CLI..."
cargo install tauri-cli

# Install frontend dependencies
echo ""
echo "📦 Installing frontend dependencies..."
pnpm install

# Start dev server
echo ""
echo "🎉 Starting development server..."
echo "   Frontend: http://localhost:5173"
echo "   Backend: Tauri development window"
echo ""

cd apps/desktop
cargo tauri dev
