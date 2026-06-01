#!/usr/bin/env bash
#
# Stops the testing node started by start-test-node.sh.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE="$ROOT/target/test-node"
PID_FILE="$CACHE/pids"
BIN="$CACHE/install/bin"

if [ -f "$PID_FILE" ]; then
    while read -r pid; do
        [ -n "$pid" ] || continue
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
    done < "$PID_FILE"
    rm -f "$PID_FILE"
fi

# Fallback in case the pid file is stale.
for bin in miden-validator miden-node miden-ntx-builder; do
    pkill -f "$BIN/$bin" 2>/dev/null || true
done

sleep 1
echo "Stopped testing node."
