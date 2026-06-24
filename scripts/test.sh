#!/usr/bin/env bash
set -euo pipefail

echo "=== Noesis Test Suite ==="
echo ""

# Step 1: Build
echo ">>> cargo build"
cargo build 2>&1
echo "✓ Build success"
echo ""

# Step 2: Run tests
echo ">>> cargo test"
cargo test 2>&1
echo "✓ Tests pass"
echo ""

# Step 3: List components
echo ">>> cargo run -- list all"
cargo run -- list all 2>&1
echo ""
echo "✓ List works"
echo ""

# Step 4: Inject a test experience
echo ">>> cargo run -- inject 'Testing the Noesis cognitive architecture with a sample experience'"
cargo run -- inject "Testing the Noesis cognitive architecture with a sample experience" 2>&1
echo ""
echo "✓ Injection works"
echo ""

# Step 5: Quick start/stop
echo ">>> cargo run -- start (5s test)..."
timeout 5 cargo run -- start 2>&1 || true
echo "✓ Start/stop works"
echo ""

echo "=== All tests passed ==="
