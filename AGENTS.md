This repo defines a web client for the Miden blockchain.
There exists a Rust part and a Javascript part, which is the wrapper for the Rust part and instantiates the Rust bits in a webassembly.
crates/web-client/src has the Rust code, crates/web-client/js is the JavaScript part.

Formatting / linting:
- CI runs `make format-check`, which requires nightly rustfmt and runs `cargo +nightly fmt --all --check && yarn prettier . --check && yarn eslint .`.
- Always use the make target above (or `make format`) instead of vanilla `cargo fmt` to avoid style regressions.
