#!/usr/bin/env bash
#
# Shared configuration for the testing-node scripts (build-test-node.sh, start-test-node.sh,
# stop-test-node.sh). Source this file; do not execute it directly.

_COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$_COMMON_DIR/.." && pwd)"

CACHE="$ROOT/target/test-node"
NODE_SRC="$CACHE/node-src"   # checkout of the node repo at the pinned rev
BUILD_DIR="$CACHE/build"     # persistent CARGO_TARGET_DIR for the node binaries
DATA="$CACHE/data"           # runtime data, wiped on each start
PID_FILE="$CACHE/pids"
LOG_DIR="$DATA/logs"

# Binary locations. Overridable so CI can run from downloaded artifacts (see start-test-node.sh).
BIN="${TEST_NODE_BIN_DIR:-$BUILD_DIR/release}"
GEN_GENESIS="${TEST_NODE_GEN_GENESIS:-$ROOT/target/release/gen-genesis}"

# Component listen addresses. The RPC port matches the client default (`MIDEN_NODE_PORT`).
RPC_LISTEN="127.0.0.1:57291"
VALIDATOR_LISTEN="127.0.0.1:50101"
NTX_BUILDER_LISTEN="127.0.0.1:50301"
RPC_PORT="${RPC_LISTEN##*:}"

# Resolves the pinned node git URL and revision from Cargo.lock into NODE_URL / NODE_REV.
#
# The node binaries are built from the exact rev our workspace already pins for the
# `miden-node-*` library crates, so the binaries and our library deps stay in lockstep.
resolve_node_src() {
    # Example: source = "git+https://github.com/0xMiden/node.git?branch=<b>#<sha>"
    local src_line src
    src_line="$(grep -m1 'source = "git+https://github.com/0xMiden/node' "$ROOT/Cargo.lock" || true)"
    if [ -z "$src_line" ]; then
        echo "error: could not find a 0xMiden/node git source in Cargo.lock" >&2
        return 1
    fi
    src="${src_line#*\"git+}"   # strip leading: source = "git+
    src="${src%\"}"             # strip trailing quote
    NODE_REV="${src##*#}"       # everything after '#'
    NODE_URL="${src%%#*}"       # everything before '#'
    NODE_URL="${NODE_URL%%\?*}" # strip the ?branch=... query
}
