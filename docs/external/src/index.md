---
title: Client
sidebar_position: 1
---

# Miden client

The Miden client currently has three main components:

1. [Miden client library](#miden-client-library).
2. [Miden client CLI](#miden-client-cli).
3. [Miden web client](#miden-web-client).

### Miden client library

The Rust libraries are split into:

- `miden-client-core`, a no_std-compatible crate with the core client logic and APIs for executing transactions, generating proofs, and managing activity on the Miden network.
- `miden-client`, a std-only facade that re-exports the core APIs and adds a background service for continuous sync and event hooks.

### Miden client CLI

The Miden client also includes a command-line interface (CLI) that serves as a wrapper around the library, exposing its basic functionality in a user-friendly manner.

The CLI provides commands for interacting with the Miden rollup, such as submitting transactions, syncing with the network, and managing account data.

More information about the CLI can be found in the [CLI section](./rust-client/cli), which now
documents the std-enabled `miden-client` service layer in addition to the core APIs.

### Miden web client

The Miden web client is a web-based interface that allows users to interact with the Miden rollup through a browser. It wraps most of the functionality of the Rust library and provides a user-friendly interface for managing accounts, submitting transactions, and monitoring activity on the network.
