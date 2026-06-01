#!/usr/bin/env bash
#
# Builds the standalone Miden node binaries (validator, sequencer, ntx-builder) and the genesis
# generator used by the testing node. Run by start-test-node.sh and by CI's build job.
#
# The node is built from the exact git revision our workspace pins for the `miden-node-*` library
# crates (read from Cargo.lock). Build artifacts go to a persistent CARGO_TARGET_DIR under
# `target/test-node/build` so they are cached and incrementally reused across runs.

set -euo pipefail

# shellcheck source=scripts/test-node-common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/test-node-common.sh"

resolve_node_src
echo "==> node source: $NODE_URL @ $NODE_REV"

mkdir -p "$CACHE"
if [ ! -d "$NODE_SRC/.git" ]; then
    echo "==> cloning node repo into $NODE_SRC"
    git clone --quiet "$NODE_URL" "$NODE_SRC"
fi

if ! git -C "$NODE_SRC" cat-file -e "${NODE_REV}^{commit}" 2>/dev/null; then
    echo "==> fetching $NODE_REV"
    # Fetch the exact commit. The pinned rev may not be a branch tip (e.g. the branch was
    # force-pushed), so fetch it by SHA; fall back to a full fetch if the server refuses.
    git -C "$NODE_SRC" fetch --quiet origin "$NODE_REV" \
        || git -C "$NODE_SRC" fetch --quiet origin
fi
git -C "$NODE_SRC" checkout --quiet --detach "$NODE_REV"

echo "==> building node binaries (CARGO_TARGET_DIR=$BUILD_DIR)"
CARGO_TARGET_DIR="$BUILD_DIR" cargo build --release --locked \
    --manifest-path "$NODE_SRC/Cargo.toml" \
    -p miden-validator -p miden-node -p miden-ntx-builder

echo "==> building gen-genesis"
cargo build --release -p node-builder --bin gen-genesis
