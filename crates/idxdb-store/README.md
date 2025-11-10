# IndexedDB Store (WASM)

Browser‑compatible `Store` implementation for the Miden client. This crate targets WebAssembly and uses
IndexedDB (via `wasm-bindgen`) to persist client state in web environments.

- Persists accounts, notes, transactions, block headers, and MMR nodes in the browser
- Ships as a `cdylib`; works with bundlers via `@wasm-tool/rollup-plugin-rust`

## Quick Start

Add to `Cargo.toml` and build for `wasm32-unknown-unknown`:

```toml
[dependencies]
miden-client       = { version = "0.12", default-features = false }
miden-idxdb-store  = { version = "0.12" }
```

## Development

### TypeScript to JavaScript Compilation

This crate includes TypeScript sources in `src/ts/` that are compiled to JavaScript in `src/js/`. The JavaScript files are:
- **NOT** committed to git (they're in `.gitignore`)
- **Automatically generated** by `build.rs` during normal builds
- **Required for publishing** to crates.io

The `build.rs` script will:
1. Install dependencies via `yarn`
2. Compile TypeScript to JavaScript in `src/js/`
3. Skip compilation if JS files already exist (for `cargo publish` verification)

### Publishing to crates.io

Before publishing, you must generate the JS files:

```bash
# Generate JS files from TypeScript sources
./prepare-publish.sh

# Verify the package
cargo publish --dry-run

# Publish to crates.io
cargo publish
```

Alternatively, you can manually generate the JS files:

```bash
cd src
yarn install
yarn build
cd ..
cargo publish
```

**Note**: The JS files must be present before running `cargo publish` because Cargo doesn't allow build scripts to modify files outside of `OUT_DIR` during package verification.

## License
This project is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.
