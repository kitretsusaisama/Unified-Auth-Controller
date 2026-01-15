#!/bin/bash
# Test script for the SSO platform

set -e

echo "Running SSO Platform tests..."

# Set test environment
export AUTH__ENVIRONMENT=test

# Run unit tests
echo "Running unit tests..."
cargo test --lib

# Run integration tests
echo "Running integration tests..."
cargo test --test '*'

# Run property-based tests (with more iterations)
echo "Running property-based tests..."
cargo test --release -- --ignored proptest

# Generate test coverage (if tarpaulin is installed)
if command -v cargo-tarpaulin &> /dev/null; then
    echo "Generating test coverage..."
    cargo tarpaulin --out Html --output-dir coverage/
    echo "Coverage report generated in coverage/tarpaulin-report.html"
fi

echo "All tests completed!"