#!/usr/bin/env bash
# Test the full recursive signal cascade.
# Starts the daemon, injects an experience, observes cascade output.

set -euo pipefail

echo "=== Noesis Cascade Test ==="
echo ""

# Build first
echo ">>> cargo build"
cargo build -q 2>&1
echo "✓ Build OK"
echo ""

# Start daemon in background
echo ">>> Starting daemon in background..."
cargo run -- start &
DAEMON_PID=$!
sleep 2
echo "✓ Daemon running (PID: $DAEMON_PID)"
echo ""

# Inject an experience
echo ">>> Injecting experience..."
cargo run -- inject "I went for a run in the park today. The weather was perfect." 2>&1
echo ""

# Give cascade time to propagate through all processors
sleep 2

# Inject another to hit the 3-episode narrative trigger and 5-episode curiosity trigger
echo ">>> Injecting more experiences to trigger cascades..."
cargo run -- inject "Read an interesting book about neural networks." 2>&1
sleep 1
echo ""

echo ">>> Injecting third experience..."
cargo run -- inject "Had a conversation about consciousness with a friend." 2>&1
sleep 1
echo ""

echo ">>> Injecting fourth experience..."
cargo run -- inject "Started learning Rust for systems programming." 2>&1
sleep 1
echo ""

echo ">>> Injecting fifth experience (should trigger curiosity)..."
cargo run -- inject "Experimented with a new recipe for dinner." 2>&1
sleep 2
echo ""

# Stop the daemon
echo ">>> Stopping daemon..."
kill $DAEMON_PID 2>/dev/null || true
wait $DAEMON_PID 2>/dev/null || true
echo "✓ Daemon stopped"
echo ""
echo "=== Cascade test complete ==="
