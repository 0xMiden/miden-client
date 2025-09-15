# IndexedDB Store (WASM)

Browser‑compatible `Store` implementation for the Miden client. This crate targets WebAssembly and uses
IndexedDB (via `wasm-bindgen`) to persist client state in web environments.

- Persists accounts, notes, transactions, block headers, and MMR nodes in the browser
- Designed for use with the `web-client` crate and Playwright tests
- Ships as a `cdylib`; works with bundlers via `@wasm-tool/rollup-plugin-rust`

## Quick Start

Add to `Cargo.toml` and build for `wasm32-unknown-unknown`:

```toml
[dependencies]
miden-client       = { version = "0.12", default-features = false }
miden-idxdb-store  = { version = "0.12" }
```

## License
This project is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.
