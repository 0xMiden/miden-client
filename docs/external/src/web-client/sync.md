---
title: sync
sidebar_position: 6
---

# Synchronizing State with the Miden SDK

This guide demonstrates how to synchronize your local state with the Miden network.

## Basic Synchronization

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const syncSummary = await client.sync();

    console.log("Current block number:", syncSummary.blockNum());
    console.log("Committed notes:", syncSummary.committedNotes());
    console.log("Updated accounts:", syncSummary.updatedAccounts());
    console.log("Committed transactions:", syncSummary.committedTransactions());
} catch (error) {
    console.error("Failed to sync state:", error.message);
}
```

## Sync with Timeout

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Sync with a 30-second timeout
    const syncSummary = await client.sync({ timeout: 30_000 });

    console.log("Synced to block:", syncSummary.blockNum());
} catch (error) {
    console.error("Sync timed out or failed:", error.message);
}
```

## Auto-Sync on Creation

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

// Client syncs automatically before returning
const client = await MidenClient.create({ autoSync: true });

// Or use createTestnet() which defaults to autoSync: true
const testnetClient = await MidenClient.createTestnet();
```

## Get Current Sync Height

```typescript
const height = await client.getSyncHeight();
console.log("Current sync height:", height);
```

## Understanding the Sync Summary

The `SyncSummary` object returned by `sync()` contains:

- `blockNum()`: The current block number of the network
- `committedNotes()`: Array of note IDs that have been committed
- `updatedAccounts()`: Array of account IDs that have been updated
- `committedTransactions()`: Array of transaction IDs that have been committed
