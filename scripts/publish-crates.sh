#!/bin/sh

# Script to publish all miden-client crates to crates.io.
# This should only be invoked manually in case the automated publishing CI workflows fail.
# Usage: ./publish-crates.sh [args] 
# 
# E.G: ./publish-crates.sh

set -e

# Check credentials
credentials=~/.cargo/credentials.toml
if [ ! -f "$credentials" ]; then
    red="\033[0;31m"
    echo "${red}WARNING: $credentials not found. See https://doc.rust-lang.org/cargo/reference/publishing.html."
    echo "\033[0m"
fi

# Checkout main
echo "Checking out main branch..."
git checkout main
git pull origin main

# Publish
echo "Publishing all crates except miden-client-web..."
cargo publish --workspace --exclude miden-client-web --exclude miden-idxdb-store

# Publish miden-client-web and miden-idxdb-store
# These should use wasm32-unknown-unknown as target (specified in .cargo/config.toml files for
# these crates, but publishing from the workspace root does not take it into account). So we
# publish them from the web-client and idxdb-store directories.
echo "Publishing miden-client-web..."
cd crates/web-client
cargo publish

echo "Publishing miden-idxdb-store..."
cd crates/idxdb-store
cargo publish
