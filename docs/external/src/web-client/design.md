---
title: Design
sidebar_position: 4
---

# Design

The Miden Web SDK shares the same core architecture as the [Rust client](../rust-client/design.md), compiled to WebAssembly with TypeScript bindings. It has the following architectural components:

- [Store](#store)
- [RPC client](#rpc-client)
- [Transaction executor](#transaction-executor)
- [Keystore](#keystore)
- [Note screener](#note-screener)
- [State sync](#state-sync)
- [Note transport](#note-transport)
- [Web Worker architecture](#web-worker-architecture)
- [WASM compilation pipeline](#wasm-compilation-pipeline)

## Store

The store manages persistence of client state using IndexedDB in the browser:

- Accounts: state history, vault assets, and account code
- Transactions and their scripts
- Notes (input and output)
- Note tags
- Block headers and chain information needed to execute transactions and consume notes

The store can track any number of accounts and notes.

## RPC client

The RPC client communicates with the Miden node through gRPC methods. The web-compatible implementation uses gRPC-web, making it suitable for browser environments.

## Transaction executor

The transaction executor uses the Miden VM (compiled to WASM) to execute transactions within the transaction kernel. When executing, the executor needs access to relevant blockchain history, which it retrieves from the store.

## Keystore

The keystore stores and manages private keys for tracked accounts. The web keystore implementation uses IndexedDB for secure browser-based key storage. Private keys are used by the executor to sign and authenticate transactions.

## Note screener

The note screener checks the consumability of notes by tracked accounts. It performs fast static checks (e.g., checking inputs for well-known notes) and dry runs of consumption transactions to determine which accounts can consume a given note.

## State sync

The state sync component handles synchronization of client state with the network. It repeatedly queries the node until the chain tip is reached, updating tracked elements (accounts, notes, transactions) with each response.

## Note transport

The SDK provides access to the note transport network for exchanging private notes. Notes are primarily exchanged using their tags as identifiers — by default the tag is derived from the recipient account ID, though it can also be randomized for increased privacy.

Methods include:

- `notes.sendPrivate()` — Send a note to the note transport network
- `notes.fetchPrivate()` — Fetch notes from the network by note tag, with pagination support

## Web Worker architecture

The SDK uses a dedicated [Web Worker](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API) to offload computationally intensive operations. This keeps the main thread responsive while the worker handles:

- Transaction proving
- State synchronization
- Account creation
- MASM compilation

Each `MidenClient` instance holds one Web Worker thread. Call `client.terminate()` to release it when done, or use [explicit resource management](https://github.com/tc39/proposal-explicit-resource-management):

```typescript
{
  using client = await MidenClient.create();
  // client.terminate() called automatically at end of scope
}
```

## WASM compilation pipeline

The SDK is built from the `web-client` Rust crate, which:

- Is implemented in Rust and compiled to WebAssembly
- Uses `wasm-bindgen` to expose JavaScript-compatible bindings
- Depends on the Rust client crate, which contains core logic for blockchain interaction

A custom `rollup.config.js` bundles the WASM module, JS bindings, and web worker into a distributable NPM package.
