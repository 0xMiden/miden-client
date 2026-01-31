---
title: sync
sidebar_position: 6
---

# Synchronizing State with the Miden SDK

This guide demonstrates how to synchronize your local state with the Miden network using the SDK. Synchronization ensures that your local data (accounts, notes, transactions) is up-to-date with the network.

## Basic Synchronization

To synchronize your local state with the network:

```typescript
import { WebClient } from "@miden-sdk/miden-sdk";

try {
    // Initialize the web client
    const webClient = await WebClient.createClient({});

    const syncSummary = await webClient.syncState();
    
    // Access synchronization details
    console.log("Current block number:", syncSummary.blockNum());
    console.log("Committed notes:", syncSummary.committedNotes().map(id => id.toString()));
    console.log("Consumed notes:", syncSummary.executedTransaction().inputNotes().map(id => id.toString()));
    console.log("Updated accounts:", syncSummary.updatedAccounts().map(id => id.toString()));
    console.log("Committed transactions:", syncSummary.committedTransactions().map(id => id.toString()));
} catch (error) {
    console.error("Failed to sync state:", error.message);
}
```

## Understanding the Sync Summary

The `SyncSummary` object returned by `syncState()` contains the following information:

- `blockNum()`: The current block number of the network
- `committedNotes()`: Array of note IDs that have been committed to the network
- `consumedNotes()`: Array of note IDs that have been consumed
- `updatedAccounts()`: Array of account IDs that have been updated
- `committedTransactions()`: Array of transaction IDs that have been committed

## Relevant Documentation

For more detailed information about sync functionality, refer to the following API documentation:

- [WebClient](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/WebClient.md) - Main client class for sync operations
- [SyncSummary](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/SyncSummary.md) - Class representing sync state
- [NoteId](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/NoteId.md) - Class for working with note IDs
- [AccountId](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/AccountId.md) - Class for working with account IDs
- [TransactionId](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/TransactionId.md) - Class for working with transaction IDs

For a complete list of available classes and utilities, see the [SDK API Reference](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/README.md).
