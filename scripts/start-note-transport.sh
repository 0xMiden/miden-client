#!/bin/bash

# Starts the Note Transport service in the foreground.
# Installs it via cargo install if not already available.

set -euo pipefail

REPO_URL=${REPO_URL:-https://github.com/0xMiden/miden-note-transport}
# Pinned to the miden-note-transport v0.4.0 commit (matches miden-note-transport-proto-build in Cargo.toml).
NOTE_TRANSPORT_REV=${NOTE_TRANSPORT_REV:-22fd42a3cae9fe0451f9f7ee93d71eabaec7b6b8}
BINARY_NAME=miden-note-transport-node-bin

if ! command -v "$BINARY_NAME" &>/dev/null; then
  echo "Installing note transport service ($NOTE_TRANSPORT_REV)..."
  cargo install --git "$REPO_URL" --rev "$NOTE_TRANSPORT_REV" --locked
fi

echo "Starting note transport service in foreground..."
RUST_LOG=info exec "$BINARY_NAME"
