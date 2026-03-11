# Basic Wallet

A Miden account component that provides basic asset management functionality.

## Functions

- **`receive_asset(asset)`** — Adds an asset to the account's vault.
- **`move_asset_to_note(asset, note_idx)`** — Removes an asset from the account and attaches it to an output note.

## Build

```sh
cargo miden build --manifest-path contracts/basic-wallet/Cargo.toml --release
```
