---
title: Examples
sidebar_position: 11
---

# Miden SDK Examples

This directory contains practical examples demonstrating how to use the `@miden-sdk/miden-sdk` in your web applications. These examples cover core functionality and common use cases for interacting with the Miden blockchain and virtual machine.

## Overview

The examples in this section showcase various capabilities of the Miden SDK, including:

- Account Operations

  - Creating new wallets
  - Creating new faucets
  - Importing existing accounts
  - Exporting account data
  - Retrieving account data

- Transaction Operations

  - Creating standard mint, consume, and send transaction requests
  - Creating custom transaction requests
  - Executing transactions
  - Submitting transactions to the network
  - Retrieving transaction history

- Note Operations
  - Retrieve input and output notes
  - Import and export notes
  - Working with consumable notes
  - Send and fetch private notes using the note transport network

For installation instructions, prerequisites, and setup details, please refer to the [SDK README](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/README.md).

Each example is self-contained and includes:

- Complete source code
- Step-by-step explanations
- Expected outputs
- Common pitfalls and troubleshooting tips

## Client Initialization

Most if not all examples require you to initialize the Miden Client. You can do this via:

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

// Initialize the client
const client = await MidenClient.create();

// Or with options
const client = await MidenClient.create({
  rpcUrl: "http://localhost:57291",
  autoSync: true
});

// For testing with a mock chain
const client = await MidenClient.createMock();
```

> **Note:** The `WebClient` class is deprecated. Use `MidenClient.create()` instead.
