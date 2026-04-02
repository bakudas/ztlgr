#!/bin/bash
# Quick setup script for ztlgr development

set -e

echo "🦀 Setting up ztlgr development environment..."

# Check if rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Error: Rust is not installed."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check Rust version
RUST_VERSION=$(rustc --version | awk '{print $2}')
echo "Rust version: $RUST_VERSION"

# Install rustfmt and clippy if not present
rustup component add rustfmt clippy 2>/dev/null || true

# Build
echo "Building..."
cargo build --release

# Run tests
echo "Running tests..."
cargo test --all-features || echo "Some tests failed, but continuing..."

# Run clippy
echo "Running clippy..."
cargo clippy --all-features -- -D warnings || echo "Clippy warnings found"

# Format
echo "Formatting..."
cargo fmt --all

echo ""
echo "✓ Setup complete!"
echo ""
echo "To run ztlgr:"
echo "  cargo run"
echo ""
echo "To create a new vault:"
echo "  cargo run -- new /path/to/vault"
echo ""
echo "To open a vault:"
echo "  cargo run -- open /path/to/vault"
echo ""