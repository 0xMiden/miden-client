# Miden Swapp Contracts

Miden Assembly contracts compiled to `.masp` via `cargo-miden`.

## Contracts

| Contract | Kind | Description |
|---|---|---|
| `basic-wallet` | Account | Custom wallet component for receiving assets |
| `swapp-note` | Note Script | Swap note supporting full and partial fills |
| `consume-asset-script` | Transaction Script | Solver transaction script for cross-swap spread capture |

## Shared Configuration

- `rust-toolchain.toml` — Nightly toolchain with `wasm32-wasip2` target
- `.cargo/config.toml` — Default build target and `cfg(miden)` flag

## Building

Each contract is built independently with `cargo-miden`:

```sh
cd basic-wallet && cargo miden build --release
cd swapp-note && cargo miden build --release
cd consume-asset-script && cargo miden build --release
```

The compiled `.masp` files are included in the repository and referenced by the `miden-swapp` crate via `include_bytes!`.

## Dependencies

`swapp-note` and `consume-asset-script` both depend on `basic-wallet` for wallet interface definitions.
