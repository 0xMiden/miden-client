#!/bin/bash

# Starts the external Note Transport service in the foreground.
# - Default: clones/updates the note transport repo and runs it via cargo

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# NOTE: This should be in line with `miden-note-transport-proto-build`'s version in Cargo.toml
BRANCH=${TRANSPORT_BRANCH:-main}

TRANSPORT_DIR=${TRANSPORT_DIR:-.tmp/miden-note-transport}
REPO_URL=${REPO_URL:-https://github.com/0xMiden/miden-note-transport}
RUN_CMD=${TRANSPORT_RUN_CMD:-cargo run --release --locked}

# Shared target directory (important for CI speed: ends up under repo `target/` which is cached)
TRANSPORT_CARGO_TARGET_DIR=${TRANSPORT_CARGO_TARGET_DIR:-"$REPO_ROOT/target/note-transport"}

mkdir -p "$(dirname "$TRANSPORT_DIR")"
mkdir -p "$TRANSPORT_CARGO_TARGET_DIR"

if [ ! -d "$TRANSPORT_DIR/.git" ]; then
  echo "Cloning note transport repo (branch: $BRANCH) into $TRANSPORT_DIR";
  git clone --depth=1 -b "$BRANCH" "$REPO_URL" "$TRANSPORT_DIR"
else
  echo "Updating note transport repo in $TRANSPORT_DIR (branch: $BRANCH)";
  git -C "$TRANSPORT_DIR" fetch --prune origin "$BRANCH"
  git -C "$TRANSPORT_DIR" checkout "$BRANCH"
  git -C "$TRANSPORT_DIR" reset --hard "origin/$BRANCH"
fi

echo "Building note transport service..."
( cd "$TRANSPORT_DIR" && CARGO_TARGET_DIR="$TRANSPORT_CARGO_TARGET_DIR" cargo build --release --locked )

echo "Starting note transport service in foreground..."
cd "$TRANSPORT_DIR"
RUST_LOG=info CARGO_TARGET_DIR="$TRANSPORT_CARGO_TARGET_DIR" exec $RUN_CMD
