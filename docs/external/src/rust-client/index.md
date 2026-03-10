---
title: Rust
sidebar_position: 1
---

# Rust client library

The `miden-client` crate is the core Rust library for interacting with the Miden rollup. It provides a programmatic API for building applications that execute transactions, generate zero-knowledge proofs, manage accounts, and sync state with the network.

## Key capabilities

- **Transaction execution** — Build and execute transactions using `TransactionRequestBuilder`
- **Proof generation** — Generate client-side ZK proofs locally or delegate to a remote prover
- **Account management** — Create and track accounts with `AccountBuilder`, supporting both private and public storage modes
- **State sync** — Keep local state in sync with the Miden network via gRPC

The library uses trait-based dependency injection for storage, RPC, proving, and key management, making each component pluggable and testable.

## Getting started

1. [Install](./install-and-run.md) the library as a dependency
2. Follow the [getting started tutorials](./get-started/index.md) to build your first Miden application
3. Explore the [library reference](./library.md) for detailed API patterns
4. Review the [design](./design.md) for architectural details
5. See the full [API documentation on docs.rs](https://docs.rs/miden-client/latest/miden_client/)
