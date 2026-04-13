---
title: Features
sidebar_position: 2
---

# Features

The Miden Web SDK offers a range of functionality for interacting with the Miden rollup directly from the browser.

### Transaction execution

The SDK facilitates the execution of transactions on the Miden rollup, allowing users to transfer assets, mint new tokens, swap tokens, and perform custom operations using Miden Assembly scripts.

### Proof generation

The Miden rollup supports user-generated proofs which are key to ensuring the validity of transactions. The SDK contains the functionality for executing, proving, and submitting transactions — all within the browser using WebAssembly.

For better performance on complex transactions, proving can be offloaded to a remote prover.

### Network interactivity

The SDK enables users to interact with the Miden network. This includes syncing with the latest blockchain data and managing account information.

The SDK also supports connectivity with the Miden Note Transport network for the exchange of private notes (end-to-end encryption coming soon).

### Account generation

The SDK provides features for generating and tracking accounts within the Miden rollup ecosystem. Users can create wallets, faucets, and custom contract accounts, and track their state and transaction history.

### MASM compilation

The SDK includes a built-in compiler for Miden Assembly (MASM), enabling developers to compile account components and transaction scripts directly in the browser without external tooling.

### Browser-native architecture

The SDK is compiled from Rust to WebAssembly and uses a dedicated Web Worker to offload computationally intensive operations (proving, syncing, wallet creation) off the main thread, keeping the UI responsive.
