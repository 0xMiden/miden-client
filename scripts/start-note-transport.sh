#!/bin/bash

# Starts the external Note Transport service in the foreground.
# - Clones the note transport repo if missing
# - Builds it
# - Runs it in the foreground

set -euo pipefail

TRANSPORT_DIR=${TRANSPORT_DIR:-.tmp/miden-note-transport}
REPO_URL=${REPO_URL:-https://github.com/0xMiden/miden-note-transport}
RUN_CMD=${TRANSPORT_RUN_CMD:-cargo run --release --locked}

mkdir -p "$(dirname "$TRANSPORT_DIR")"

if [ ! -d "$TRANSPORT_DIR/.git" ]; then
  echo "Cloning note transport repo into $TRANSPORT_DIR";
  git clone --depth=1 "$REPO_URL" "$TRANSPORT_DIR"
else
  echo "Updating note transport repo in $TRANSPORT_DIR";
  git -C "$TRANSPORT_DIR" fetch --prune
  # Reset to the default remote HEAD to avoid local drift on CI
  DEFAULT_REF=$(git -C "$TRANSPORT_DIR" symbolic-ref --quiet refs/remotes/origin/HEAD || true)
  if [ -n "${DEFAULT_REF:-}" ]; then
    git -C "$TRANSPORT_DIR" reset --hard "$DEFAULT_REF"
  else
    git -C "$TRANSPORT_DIR" reset --hard origin/HEAD || true
  fi
fi

echo "Building note transport service..."
( cd "$TRANSPORT_DIR" && cargo build --release --locked )

echo "Starting note transport service in foreground..."
cd "$TRANSPORT_DIR"
RUST_LOG=info exec $RUN_CMD
