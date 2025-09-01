#!/bin/sh

# Script to publish all miden-node crates to crates.io.
# Usage: ./publish-crates.sh [args]
#
# E.G:   ./publish-crates.sh

set -e

# Check
credentials=~/.cargo/credentials.toml
if [ ! -f "$credentials" ]; then
    red="\033[0;31m"
    echo "${red}WARNING: $credentials not found. See https://doc.rust-lang.org/cargo/reference/publishing.html."
    echo "\033[0m"
fi

# Checkout
echo "Checking out main branch..."
git checkout main
git pull origin main

# Publish
echo "Publishing crates..."

crates=(
    miden-client
    miden-client-cli
    miden-client-integration-tests
)

for crate in ${crates[@]}; do
    echo "Publishing $crate..."
    cargo publish -p "$crate" 
done

# Publish miden-client-web
# This should use wasm32-unknown-unknown as target (specified on crates/web-client/config.toml,
# but publishing from the workspace root does not take it into account)
echo "Publishing miden-client-web..."
cargo publish -p miden-client-web --target wasm32-unknown-unknown
