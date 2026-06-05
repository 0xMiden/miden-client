#!/usr/bin/env bash
#
# Starts a testing node from the standalone Miden node executables (validator, sequencer,
# ntx-builder).
#
# Binaries are built with `cargo install` from the exact git rev our workspace pins for the
# `miden-node-*` crates (read from Cargo.lock), so they stay in lockstep with our library deps —
# including unreleased revs, which have no published binaries to download. `cargo install` is a
# no-op when that rev is already installed.
#
# To keep the compile cheap, the install reuses a persistent `--target-dir` ($CACHE/build) so
# only crates that changed between revs recompile. In CI a dedicated `build-test-node` job builds
# the binaries once (caching both the install dir and the build dir, keyed on the node rev) and
# shares them with the test jobs as an artifact; see `.github/workflows/test.yml`. Genesis content
# comes from the `gen-genesis` helper.
#
# Modes:
#   (no args)        build/install if needed, then bootstrap and start the node (default)
#   --install-only   build/install the node binaries and exit (used by the CI build job)
#   --print-rev      print the pinned node rev resolved from Cargo.lock and exit

set -euo pipefail

MODE="run"
case "${1:-}" in
    --install-only) MODE="install-only" ;;
    --print-rev)    MODE="print-rev" ;;
    "")             ;;
    *) echo "error: unknown argument '$1'" >&2; exit 2 ;;
esac

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE="$ROOT/target/test-node"
BIN="$CACHE/install/bin"
BUILD="$CACHE/build"
GEN_GENESIS="$ROOT/target/release/gen-genesis"
DATA="$CACHE/data"
LOG_DIR="$DATA/logs"
PID_FILE="$CACHE/pids"

RPC="127.0.0.1:57291"   # matches the client default (`MIDEN_NODE_PORT`)
VALIDATOR="127.0.0.1:50101"
NTX="127.0.0.1:50301"
TX_PROVER="${MIDEN_TX_PROVER:-127.0.0.1:50051}"

# Resolve the pinned node url + rev from Cargo.lock.
# Example: source = "git+https://github.com/0xMiden/node.git?branch=<b>#<sha>"
SRC_LINE="$(grep -m1 'source = "git+https://github.com/0xMiden/node' "$ROOT/Cargo.lock" || true)"
[ -n "$SRC_LINE" ] || { echo "error: no 0xMiden/node git source in Cargo.lock" >&2; exit 1; }
SRC="${SRC_LINE#*\"git+}"; SRC="${SRC%\"}"
NODE_REV="${SRC##*#}"
NODE_URL="${SRC%%#*}"; NODE_URL="${NODE_URL%%\?*}"

if [ "$MODE" = "print-rev" ]; then
    echo "$NODE_REV"
    exit 0
fi

node_binaries_installed() {
    local metadata="$CACHE/install/.crates.toml"
    [ -f "$metadata" ] || return 1

    for bin in miden-validator miden-node miden-ntx-builder; do
        [ -x "$BIN/$bin" ] || return 1
        if ! grep -F "\"$bin " "$metadata" | grep -Fq "#$NODE_REV)"; then
            return 1
        fi
    done
}

if node_binaries_installed; then
    echo "==> using cached node binaries ($NODE_URL @ $NODE_REV)"
else
    echo "==> installing node binaries ($NODE_URL @ $NODE_REV)"
    cargo install --locked --root "$CACHE/install" --target-dir "$BUILD" \
        --git "$NODE_URL" --rev "$NODE_REV" \
        miden-validator miden-node miden-ntx-builder
fi

if [ "$MODE" = "install-only" ]; then
    echo "==> install-only: node binaries ready in $BIN"
    exit 0
fi

echo "==> building gen-genesis"
cargo build --release -p test-node-genesis --bin gen-genesis

echo "==> generating genesis + bootstrapping"
rm -rf "$DATA"
# Each component opens its SQLite DB directly under its data dir and does not create it.
mkdir -p "$LOG_DIR" "$DATA/validator" "$DATA/node" "$DATA/ntx-builder"
"$GEN_GENESIS" "$DATA/genesis-config"
mkdir -p "$ROOT/data"
cp "$DATA/genesis-config/tst_faucet.mac" "$ROOT/data/account.mac"

"$BIN/miden-validator" bootstrap --data-directory "$DATA/validator" \
    --genesis-block-directory "$DATA/genesis" --accounts-directory "$DATA/accounts" \
    --genesis-config-file "$DATA/genesis-config/genesis.toml"
"$BIN/miden-node" bootstrap --data-directory "$DATA/node" --file "$DATA/genesis/genesis.dat"
"$BIN/miden-ntx-builder" bootstrap --data-directory "$DATA/ntx-builder" --file "$DATA/genesis/genesis.dat"

echo "==> starting components"
: > "$PID_FILE"
start() {
    local name="$1"; shift
    RUST_LOG="${RUST_LOG:-info}" nohup "$@" >"$LOG_DIR/$name.log" 2>&1 &
    echo "$!" >> "$PID_FILE"
}
start validator   "$BIN/miden-validator" start --listen "$VALIDATOR" --data-directory "$DATA/validator"
start sequencer   "$BIN/miden-node" sequencer --rpc.listen "$RPC" --data-directory "$DATA/node" \
    --validator.url "http://$VALIDATOR" --ntx-builder.url "http://$NTX"
start ntx-builder "$BIN/miden-ntx-builder" start --listen "$NTX" --rpc.url "http://$RPC" \
    --tx-prover.url "http://$TX_PROVER" --data-directory "$DATA/ntx-builder"

echo "==> waiting for RPC on $RPC"
for _ in $(seq 1 60); do
    if (exec 3<>"/dev/tcp/${RPC%:*}/${RPC##*:}") 2>/dev/null; then
        exec 3>&- 3<&-
        echo "==> node is up (RPC on http://$RPC); logs in $LOG_DIR"
        exit 0
    fi
    while read -r pid; do
        [ -n "$pid" ] || continue
        if ! kill -0 "$pid" 2>/dev/null; then
            echo "error: a node service exited during startup; see $LOG_DIR" >&2
            exit 1
        fi
    done < "$PID_FILE"
    sleep 1
done
echo "error: RPC did not become ready within 60s; see $LOG_DIR" >&2
exit 1
