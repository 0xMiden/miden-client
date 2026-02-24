#!/bin/bash

# Starts the binary in the background and checks that it has not exited

if [ -z "$1" ]; then
    echo "Usage: $0 <binary-name>"
    exit 1
fi;

BINARY_NAME="$1"

# Resolve the workspace target directory (handles being called from subcrates)
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps 2>/dev/null | grep -o '"target_directory":"[^"]*"' | head -1 | cut -d'"' -f4)
TARGET_DIR="${TARGET_DIR:-target}"
BINARY_PATH="$TARGET_DIR/release/$BINARY_NAME"

if [ -x "$BINARY_PATH" ]; then
    echo "$BINARY_NAME binary found, skipping build"
else
    if ! cargo build --release --bin "$BINARY_NAME" --locked; then
        echo "Failed to build $BINARY_NAME"
        exit 1
    fi;
fi;

RUST_LOG=none "$BINARY_PATH" & echo $! > .$BINARY_NAME.pid;
sleep 4;
if ! ps -p $(cat .$BINARY_NAME.pid) > /dev/null; then
    echo "Failed to start $BINARY_NAME";
    rm -f .$BINARY_NAME.pid;
    exit 1;
fi;
rm -f .$BINARY_NAME.pid
