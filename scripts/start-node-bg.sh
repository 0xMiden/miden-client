#!/bin/bash

# Starts the node in the background and checks that it has not exited

if ! cargo build --release --package node-builder; then
    echo "Failed to build node server";
    exit 1;
fi;

RUST_LOG=none cargo run --release --package node-builder & echo $! > .node.pid;
sleep 4;
if ! ps -p $(cat .node.pid) > /dev/null; then
    echo "Failed to start node server";
    rm -f .node.pid;
    exit 1;
fi;
rm -f .node.pid
