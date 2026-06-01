#!/usr/bin/env bash
#
# Starts a testing node from the standalone Miden node executables (validator, sequencer,
# ntx-builder). Genesis content is produced by the `gen-genesis` helper (deterministic test
# faucets + the `too_many_assets` account) and handed to `miden-validator bootstrap`.
#
# By default the binaries are built via build-test-node.sh. Set TEST_NODE_SKIP_BUILD=1 to use
# prebuilt binaries instead (CI downloads them as artifacts and points TEST_NODE_BIN_DIR /
# TEST_NODE_GEN_GENESIS at them).

set -euo pipefail

# shellcheck source=scripts/test-node-common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/test-node-common.sh"

# --- build (unless using prebuilt binaries) ------------------------------------------------------

if [ -z "${TEST_NODE_SKIP_BUILD:-}" ]; then
    "$_COMMON_DIR/build-test-node.sh"
else
    echo "==> TEST_NODE_SKIP_BUILD set; using prebuilt binaries"
fi

for bin in miden-validator miden-node miden-ntx-builder; do
    if [ ! -x "$BIN/$bin" ]; then
        echo "error: missing binary $BIN/$bin" >&2
        exit 1
    fi
done
if [ ! -x "$GEN_GENESIS" ]; then
    echo "error: missing gen-genesis at $GEN_GENESIS" >&2
    exit 1
fi

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
