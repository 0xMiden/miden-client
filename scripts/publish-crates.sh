#!/bin/sh

# Script to publish all miden-node crates to crates.io. 
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

echo "Publishing crates..."

crates=(
  miden-client
  miden-client-sqlite-store
  miden-client-cli
)

for crate in "${crates[@]}"; do
    echo "Publishing $crate..."
    cargo publish -p "$crate"
done

# Publish wasm crates

crates_wasm=(
  miden-client-web
  miden-idxdb-store
)

for crate in "${crates_wasm[@]}"; do
    echo "Publishing $crate (wasm)..."
    cargo publish -p "$crate" --target wasm32-unknown-unknown
done
