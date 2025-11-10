#!/bin/bash
set -e

# This script prepares the idxdb-store crate for publishing by generating
# the JS files from TypeScript sources before running cargo publish.
# 
# The JS files are generated in src/js/ and are included in the package
# via the Cargo.toml include field, but are NOT committed to git (.gitignore).

echo "Generating JS files from TypeScript sources..."
cd src
yarn install
yarn build
cd ..

echo "JS files generated successfully. You can now run:"
echo "  cargo publish --dry-run"
echo "or:"
echo "  cargo publish"
