---
title: Installation
sidebar_position: 1
---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) version 1.88 or later.

## Add the dependency

Add `miden-client` to your project's `Cargo.toml`:

```toml
[dependencies]
miden-client = { version = "0.14" }
```

### Feature flags

The crate exposes several feature flags for optional functionality:

- **`tonic`** — Enables gRPC-based RPC client (required for network communication)
- **`testing`** — Enables test utilities and mock implementations

Enable features as needed:

```toml
[dependencies]
miden-client = { version = "0.14", features = ["tonic", "testing"] }
```

### SQLite store

For persistent storage, add the SQLite store crate:

```toml
[dependencies]
miden-client = { version = "0.14" }
miden-client-sqlite-store = { version = "0.14" }
```

## Verify

Confirm the dependency resolves correctly:

```sh
cargo check
```

## Next steps

Head to the [getting started tutorials](./get-started/index.md) to create your first account and execute a transaction using the Rust API.
