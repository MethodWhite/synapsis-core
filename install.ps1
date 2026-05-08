# Synapsis-Core Windows Installer
# PROPRIETARY - All Rights Reserved

Write-Host "╔══════════════════════════════════════════════════════════╗"
Write-Host "║  Synapsis-Core Windows Installer                         ║"
Write-Host "║  PROPRIETARY SOFTWARE - LICENSED, NOT SOLD               ║"
Write-Host "╚══════════════════════════════════════════════════════════╝"
Write-Host ""

# Check Rust
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "📦 Installing Rust..."
    winget install Rustlang.Rustup
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}

# Build Synapsis-Core
Write-Host "📦 Building Synapsis-Core..."
Set-Location $PSScriptRoot
cargo build --release

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════╗"
Write-Host "║  Installation Complete ✅                                ║"
Write-Host "╚══════════════════════════════════════════════════════════╝"
Write-Host ""
Write-Host "Synapsis-Core is a library crate."
Write-Host "Add to Cargo.toml:"
Write-Host "  synapsis-core = { path = `"../synapsis-core`" }"
Write-Host ""
