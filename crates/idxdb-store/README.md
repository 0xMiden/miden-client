# IndexedDB Store (WASM)

Browser‑compatible `Store` implementation for the Miden client. This crate targets WebAssembly and uses
IndexedDB (via `wasm-bindgen`) to persist client state in web environments.

- Persists accounts, notes, transactions, block headers, and MMR nodes in the browser
- Ships as a `cdylib`; works with bundlers via `@wasm-tool/rollup-plugin-rust`

## Quick Start

Add to `Cargo.toml` and build for `wasm32-unknown-unknown`:

```toml
[dependencies]
miden-client       = { version = "0.13", default-features = false }
miden-idxdb-store  = { version = "0.13" }
```

## License
This project is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.
