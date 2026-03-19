#!/bin/bash
set -e

# Ensure Rust toolchain is available
if ! command -v cargo &> /dev/null; then
    echo "ERROR: cargo not found. Install Rust toolchain."
    exit 1
fi

# Ensure bun is available for frontend
if ! command -v bun &> /dev/null; then
    echo "ERROR: bun not found. Install bun for frontend."
    exit 1
fi

# Install frontend dependencies (idempotent)
cd frontend && bun install --frozen-lockfile 2>/dev/null || bun install
cd ..

# Verify backend compiles
cargo check --quiet 2>/dev/null || true

echo "Environment ready."
