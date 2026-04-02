#!/bin/bash
# Quick test script to check if ztlgr compiles
# Run this after fixing compilation errors

echo "Checking for compilation errors..."
echo ""

# Try to find cargo
if command -v cargo &> /dev/null; then
    echo "Found cargo at: $(which cargo)"
    echo ""
    echo "Running cargo check..."
    cargo check 2>&1 | head -50
    EXIT_CODE=$?
    
    echo ""
    echo "Exit code: $EXIT_CODE"
    
    if [ $EXIT_CODE -eq 0 ]; then
        echo "✅ Compilation successful!"
        echo ""
        echo "To run the project:"
        echo "  cargo run"
    else
        echo "❌ Compilation failed with errors above"
        echo ""
        echo "If you see import errors, check:"
        echo "  1. Module declarations in mod.rs files"
        echo "  2. Public exports in lib.rs"
        echo "  3. Use statements for external crates"
    fi
else
    echo "❌ Cargo not found in PATH"
    echo ""
    echo "Install Rust with:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "Or use the Nix environment:"
    echo "  nix-shell"
    echo "  # or"
    echo "  direnv allow"
fi