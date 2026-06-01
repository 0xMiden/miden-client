#!/usr/bin/env bash
#
# Starts a testing node from the standalone Miden node executables (validator, sequencer,
# ntx-builder).
#
# The binaries are installed with `cargo install` from the exact git revision our workspace pins
# for the `miden-node-*` crates (read from Cargo.lock), so they stay in lockstep with our library
# deps. `cargo install` is a no-op when that rev is already installed; CI caches the install dir
# keyed on Cargo.lock so warm runs skip the build entirely (same approach as the note-transport).
#
# Genesis content is produced by the `gen-genesis` helper (deterministic test faucets + the
# `too_many_assets` account) and handed to `miden-validator bootstrap`.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

CACHE="$ROOT/target/test-node"
INSTALL_ROOT="$CACHE/install"   # cargo install --root; binaries land in $INSTALL_ROOT/bin
BIN="$INSTALL_ROOT/bin"
DATA="$CACHE/data"              # runtime data, wiped on each start
LOG_DIR="$DATA/logs"
PID_FILE="$CACHE/pids"
GEN_GENESIS="$ROOT/target/release/gen-genesis"

# Component listen addresses. The RPC port matches the client default (`MIDEN_NODE_PORT`).
RPC_LISTEN="127.0.0.1:57291"
VALIDATOR_LISTEN="127.0.0.1:50101"
NTX_BUILDER_LISTEN="127.0.0.1:50301"
RPC_PORT="${RPC_LISTEN##*:}"

# --- resolve the pinned node revision from Cargo.lock --------------------------------------------

# Example: source = "git+https://github.com/0xMiden/node.git?branch=<b>#<sha>"
SRC_LINE="$(grep -m1 'source = "git+https://github.com/0xMiden/node' "$ROOT/Cargo.lock" || true)"
if [ -z "$SRC_LINE" ]; then
    echo "error: could not find a 0xMiden/node git source in Cargo.lock" >&2
    exit 1
fi
SRC="${SRC_LINE#*\"git+}"   # strip leading: source = "git+
SRC="${SRC%\"}"             # strip trailing quote
NODE_REV="${SRC##*#}"       # everything after '#'
NODE_URL="${SRC%%#*}"       # everything before '#'
NODE_URL="${NODE_URL%%\?*}" # strip the ?branch=... query

# --- install binaries ----------------------------------------------------------------------------

echo "==> installing node binaries ($NODE_URL @ $NODE_REV)"
cargo install --locked --root "$INSTALL_ROOT" \
    --git "$NODE_URL" --rev "$NODE_REV" \
    miden-validator miden-node miden-ntx-builder

echo "==> building gen-genesis"
cargo build --release -p node-builder --bin gen-genesis

# --- fresh runtime state -------------------------------------------------------------------------

echo "==> resetting runtime data at $DATA"
rm -rf "$DATA"
# Each component opens a SQLite DB directly under its data directory and does not create the
# directory itself, so create them up front.
mkdir -p "$DATA" "$LOG_DIR" "$DATA/validator" "$DATA/node" "$DATA/ntx-builder"

GENESIS_CONFIG="$DATA/genesis-config"  # .mac files + genesis.toml
GENESIS_BLOCK="$DATA/genesis"          # genesis.dat lands here
ACCOUNTS_DIR="$DATA/accounts"          # bootstrap writes account secret files here

echo "==> generating genesis fixtures"
"$GEN_GENESIS" "$GENESIS_CONFIG"

# --- bootstrap -----------------------------------------------------------------------------------

echo "==> bootstrapping validator (builds and signs genesis.dat)"
"$BIN/miden-validator" bootstrap \
    --data-directory "$DATA/validator" \
    --genesis-block-directory "$GENESIS_BLOCK" \
    --accounts-directory "$ACCOUNTS_DIR" \
    --genesis-config-file "$GENESIS_CONFIG/genesis.toml"

GENESIS_DAT="$GENESIS_BLOCK/genesis.dat"

echo "==> bootstrapping node"
"$BIN/miden-node" bootstrap --data-directory "$DATA/node" --file "$GENESIS_DAT"

echo "==> bootstrapping ntx-builder"
"$BIN/miden-ntx-builder" bootstrap --data-directory "$DATA/ntx-builder" --file "$GENESIS_DAT"

# --- start the long-running services -------------------------------------------------------------

: > "$PID_FILE"
start_service() {
    local name="$1"; shift
    echo "==> starting $name"
    RUST_LOG="${RUST_LOG:-info}" nohup "$@" >"$LOG_DIR/$name.log" 2>&1 &
    echo "$!" >> "$PID_FILE"
}

start_service validator "$BIN/miden-validator" start \
    --listen "$VALIDATOR_LISTEN" \
    --data-directory "$DATA/validator"

start_service sequencer "$BIN/miden-node" sequencer \
    --rpc.listen "$RPC_LISTEN" \
    --data-directory "$DATA/node" \
    --validator.url "http://$VALIDATOR_LISTEN" \
    --ntx-builder.url "http://$NTX_BUILDER_LISTEN"

start_service ntx-builder "$BIN/miden-ntx-builder" start \
    --listen "$NTX_BUILDER_LISTEN" \
    --rpc.url "http://$RPC_LISTEN" \
    --data-directory "$DATA/ntx-builder"

# --- wait until the RPC port accepts connections -------------------------------------------------

echo "==> waiting for RPC on $RPC_LISTEN"
for _ in $(seq 1 60); do
    if (exec 3<>"/dev/tcp/127.0.0.1/$RPC_PORT") 2>/dev/null; then
        exec 3>&- 3<&-
        echo "==> node is up (RPC on http://$RPC_LISTEN); logs in $LOG_DIR"
        exit 0
    fi
    # Bail out early if any service died during startup.
    while read -r pid; do
        if ! kill -0 "$pid" 2>/dev/null; then
            echo "error: a node service exited during startup; see logs in $LOG_DIR" >&2
            exit 1
        fi
    done < "$PID_FILE"
    sleep 1
done

echo "error: RPC did not become ready within 60s; see logs in $LOG_DIR" >&2
exit 1
