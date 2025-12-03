#!/bin/bash

# Starts the binary in the background and checks that it has not exited

if [ -z "$1" ]; then
    echo "Usage: $0 <package-name>"
    exit 1
fi;

PACKAGE_NAME="$1"
BIN_NAME="${BIN_NAME:-$PACKAGE_NAME}"
BINARY_PATH="${BINARY_PATH:-target/release/$BIN_NAME}"

if [ ! -x "$BINARY_PATH" ]; then
    if ! cargo build --release --package "$PACKAGE_NAME" --locked; then
        echo "Failed to build $PACKAGE_NAME"
        exit 1
    fi;
fi;

if [ ! -x "$BINARY_PATH" ]; then
    echo "Binary for $PACKAGE_NAME not found at $BINARY_PATH"
    exit 1
fi;

RUST_LOG=none "$BINARY_PATH" & echo $! > .$PACKAGE_NAME.pid;
sleep 4;
if ! ps -p $(cat .$PACKAGE_NAME.pid) > /dev/null; then
    echo "Failed to start $PACKAGE_NAME";
    rm -f .$PACKAGE_NAME.pid;
    exit 1;
fi;
rm -f .$PACKAGE_NAME.pid
