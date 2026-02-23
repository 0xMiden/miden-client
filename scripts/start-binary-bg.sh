#!/bin/bash

# Starts the binary in the background and checks that it has not exited

if [ -z "$1" ]; then
    echo "Usage: $0 <package-name>"
    exit 1
fi;

PACKAGE_NAME="$1"

# Resolve the workspace target directory (handles being called from subcrates)
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps 2>/dev/null | grep -o '"target_directory":"[^"]*"' | head -1 | cut -d'"' -f4)
TARGET_DIR="${TARGET_DIR:-target}"
BINARY_PATH="$TARGET_DIR/release/$PACKAGE_NAME"

if [ -x "$BINARY_PATH" ]; then
    echo "$PACKAGE_NAME binary found, skipping build"
else
    if ! cargo build --release --package "$PACKAGE_NAME" --locked; then
        echo "Failed to build $PACKAGE_NAME"
        exit 1
    fi;
fi;

RUST_LOG=none "$BINARY_PATH" & echo $! > .$PACKAGE_NAME.pid;
sleep 4;
if ! ps -p $(cat .$PACKAGE_NAME.pid) > /dev/null; then
    echo "Failed to start $PACKAGE_NAME";
    rm -f .$PACKAGE_NAME.pid;
    exit 1;
fi;
rm -f .$PACKAGE_NAME.pid
