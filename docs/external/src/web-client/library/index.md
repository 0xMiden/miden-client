---
title: Library
sidebar_position: 0
---

# Library

The `@miden-sdk/ts-sdk` package provides a resource-based TypeScript API for interacting with the Miden rollup from the browser.

## Client initialization

The SDK provides several factory methods to create a client:

```typescript
import { MidenClient } from "@miden-sdk/ts-sdk";

// Default client (connects to localhost)
const client = await MidenClient.create();

// Testnet client (pre-configured RPC, auto-sync enabled)
const client = await MidenClient.createTestnet();

// Mock client (no network, for testing)
const client = await MidenClient.createMock();

// With custom options
const client = await MidenClient.create({
  rpcUrl: "http://localhost:57291",
  autoSync: true,
  proverUrl: "https://prover.example.com"
});
```

## Resource-based API

The client organizes functionality into resource namespaces:

| Namespace | Purpose | Reference |
|-----------|---------|-----------|
| `client.accounts.*` | Create, retrieve, import, export accounts | [Accounts](./accounts.md), [New accounts](./new-accounts.md) |
| `client.transactions.*` | Mint, send, consume, swap, execute transactions | [New transactions](./new-transactions.md), [Transactions](./transactions.md) |
| `client.notes.*` | List, import, export, send/fetch private notes | [Notes](./notes.md), [Note transport](./note-transport.md) |
| `client.compile.*` | Compile MASM components and transaction scripts | [Compile](./compile.md) |
| `client.tags.*` | Manage note tags for sync filtering | [Tags](./tags.md) |
| `client.sync()` | Synchronize local state with the network | [Sync](./sync.md) |

Additional top-level methods:

| Method | Purpose |
|--------|---------|
| `client.importStore(snapshot)` | Import a store snapshot ([Import](./import.md)) |
| `client.exportStore()` | Export the store for backup ([Export](./export.md)) |
| `client.getSyncHeight()` | Get the current sync block height |
| `client.terminate()` | Release the Web Worker thread |

## Resource management

Each `MidenClient` holds a dedicated Web Worker thread. When you're done with a client instance, call `terminate()` to release the worker:

```typescript
const client = await MidenClient.create({ rpcUrl });

// ... use the client ...

// Clean up when done
client.terminate();
```

Or use explicit resource management (TC39 proposal):

```typescript
{
  using client = await MidenClient.create();
  // client.terminate() called automatically at end of scope
}
```

:::warning
After calling `terminate()`, all subsequent method calls will throw `Error("Client terminated")`.
:::
