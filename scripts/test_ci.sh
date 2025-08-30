#!/bin/bash
set -euxo pipefail

echo "=== Testing CI Fast Lane locally ==="

# 1. Format check
echo "Checking formatting..."
cargo fmt --all -- --check || { echo "FAILED: Formatting check"; exit 1; }

# 2. Clippy check
echo "Running clippy..."
cargo clippy --target x86_64-unknown-none --no-default-features --features "ci-lint-min" -Z build-std=core,alloc -- -A warnings || { echo "FAILED: Clippy check"; exit 1; }

# 3. Minimal build
echo "Building minimal features..."
cargo build --target x86_64-unknown-none --no-default-features -Z build-std=core,alloc || { echo "FAILED: Minimal build"; exit 1; }

echo "=== All CI Fast Lane checks passed! ==="
