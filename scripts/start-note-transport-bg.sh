#!/bin/bash

# Starts the external Note Transport service in the background.
# - Default: clones/updates the note transport repo and runs it via cargo

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

TRANSPORT_DIR=${TRANSPORT_DIR:-.tmp/miden-note-transport}
REPO_URL=${REPO_URL:-https://github.com/0xMiden/miden-note-transport}
BRANCH=${TRANSPORT_BRANCH:-mmagician-update-base-v0.13}
RUN_CMD=${TRANSPORT_RUN_CMD:-cargo run --release --locked}

# Shared target directory (important for CI speed: ends up under repo `target/` which is cached)
TRANSPORT_CARGO_TARGET_DIR=${TRANSPORT_CARGO_TARGET_DIR:-"$REPO_ROOT/target/note-transport"}

PID_FILE=.note-transport.pid

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

echo "Starting note transport service in background..."
( cd "$TRANSPORT_DIR" && RUST_LOG=info CARGO_TARGET_DIR="$TRANSPORT_CARGO_TARGET_DIR" $RUN_CMD ) & echo $! > "$PID_FILE"

sleep 4

if [ ! -s "$PID_FILE" ]; then
  echo "Failed to start note transport service: PID file missing or empty"
  rm -f "$PID_FILE"
  exit 1
fi

PID=$(cat "$PID_FILE")
if ! [[ "$PID" =~ ^[0-9]+$ ]]; then
  echo "Failed to start note transport service: PID file invalid"
  rm -f "$PID_FILE"
  exit 1
fi

if ! ps -p "$PID" > /dev/null 2>&1; then
  echo "Failed to start note transport service"
  rm -f "$PID_FILE"
  exit 1
fi

echo "Note transport service started (pid $PID)"

