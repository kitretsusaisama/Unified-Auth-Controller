#!/bin/bash
# Build script for the SSO platform

set -e

echo "Building SSO Platform..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check Rust version
echo "Rust version:"
rustc --version

# Format code
echo "Formatting code..."
cargo fmt --all

# Run clippy
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Build the project
echo "Building project..."
cargo build --release

# Run tests
echo "Running tests..."
cargo test --all

echo "Build completed successfully!"