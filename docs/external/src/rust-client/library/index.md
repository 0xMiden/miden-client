---
title: Library
sidebar_position: 0
---

# Library

The `miden-client` crate provides a Rust API for interacting with the Miden rollup. All operations go through the `Client` struct, created via `ClientBuilder`.

## Client initialization

The recommended way to create a client is using `ClientBuilder`. For standard networks, use the pre-configured constructors:

```rust
use std::sync::Arc;
use miden_client::builder::ClientBuilder;
use miden_client_sqlite_store::SqliteStore;

// Create store
let sqlite_store = SqliteStore::new("path/to/store".try_into()?).await?;
let store = Arc::new(sqlite_store);

// Build client for testnet (pre-configured RPC, prover, and note transport)
let mut client = ClientBuilder::for_testnet()
    .store(store)
    .filesystem_keystore("path/to/keys")?
    .build()
    .await?;
```

Other network constructors:
- `ClientBuilder::for_testnet()` — Pre-configured for Miden testnet
- `ClientBuilder::for_devnet()` — Pre-configured for Miden devnet
- `ClientBuilder::for_localhost()` — Pre-configured for local development

For custom configurations, use `ClientBuilder::new()` and configure each component:

```rust
use std::sync::Arc;
use miden_client::builder::ClientBuilder;
use miden_client::rpc::{Endpoint, GrpcClient};
use miden_client_sqlite_store::SqliteStore;

let sqlite_store = SqliteStore::new("path/to/store".try_into()?).await?;
let store = Arc::new(sqlite_store);

let endpoint = Endpoint::new("https".into(), "localhost".into(), Some(57291));

let mut client = ClientBuilder::new()
    .grpc_client(&endpoint, None)
    .store(store)
    .filesystem_keystore("path/to/keys")?
    // Optional: custom prover via .prover(Arc::new(prover))
    // Optional: note transport via .note_transport(Arc::new(nt_client))
    // Optional: debug mode via .in_debug_mode(DebugMode::Enabled)
    .build()
    .await?;
```

## API overview

| Area | Purpose | Reference |
|------|---------|-----------|
| Accounts | Create, retrieve accounts | [Accounts](./accounts.md), [New accounts](./new-accounts.md) |
| Compile | Compile MASM scripts and components | [Compile](./compile.md) |
| Import / Export | Import and export accounts and notes | [Import](./import.md), [Export](./export.md) |
| Transactions | Build, execute, prove, submit transactions | [New transactions](./new-transactions.md), [Transactions](./transactions.md) |
| Tags | Manage note tags for sync filtering | [Tags](./tags.md) |
| Notes | List, filter, consume notes | [Notes](./notes.md) |
| Note Transport | Send and receive private notes | [Note Transport](./note-transport.md) |
| Sync | Synchronize local state with the network | [Sync](./sync.md) |

## Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
miden-client = { version = "0.14", features = ["tonic"] }
miden-client-sqlite-store = { version = "0.14" }
tokio = { version = "1", features = ["full"] }
```
