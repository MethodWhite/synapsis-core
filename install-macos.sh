#!/bin/bash
# Synapsis-Core macOS Installer
# PROPRIETARY - All Rights Reserved

set -e

echo "╔══════════════════════════════════════════════════════════╗"
echo "║  Synapsis-Core macOS Installer                           ║"
echo "║  PROPRIETARY SOFTWARE - LICENSED, NOT SOLD               ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Check Homebrew
if ! command -v brew &> /dev/null; then
    echo "📦 Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# Check Rust
if ! command -v rustc &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Build Synapsis-Core
echo "📦 Building Synapsis-Core..."
cd "$(dirname "$(realpath "$0" 2>/dev/null || echo "$0")")"
cargo build --release

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║  Installation Complete ✅                                ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "Synapsis-Core is a library crate."
echo "Add to Cargo.toml:"
echo "  synapsis-core = { path = \"../synapsis-core\" }"
echo ""
