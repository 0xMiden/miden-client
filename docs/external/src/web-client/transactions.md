---
title: Transactions
sidebar_position: 9
---

# Retrieving Transaction History with the Miden SDK

This guide demonstrates how to retrieve and work with transaction history using the Miden SDK.

## Listing All Transactions

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // List all transactions
    const allTransactions = await client.transactions.list();

    for (const tx of allTransactions) {
        console.log("Transaction ID:", tx.id().toString());
        console.log("Account ID:", tx.accountId().toString());
        console.log("Block Number:", tx.blockNum().toString());

        // Check transaction status
        const status = tx.transactionStatus();
        if (status.isPending()) {
            console.log("Status: Pending");
        } else if (status.isCommitted()) {
            console.log("Status: Committed in block", status.getBlockNum());
            console.log("Committed at:", status.getCommitTimestamp());
        } else if (status.isDiscarded()) {
            console.log("Status: Discarded");
        }

        // Account state changes
        console.log("Initial State:", tx.initAccountState().toHex());
        console.log("Final State:", tx.finalAccountState().toHex());

        // Notes information
        console.log("Input Note Nullifiers:", tx.inputNoteNullifiers().map(n => n.toHex()));
        console.log("Output Notes:", tx.outputNotes().toString());
    }
} catch (error) {
    console.error("Failed to retrieve transactions:", error.message);
}
```

## Filtering Transactions

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Get uncommitted transactions
    const uncommitted = await client.transactions.list({ status: "uncommitted" });
    for (const tx of uncommitted) {
        console.log("Uncommitted:", tx.id().toString());
    }

    // Get specific transactions by ID
    const specific = await client.transactions.list({ ids: [txId1, txId2] });

    // Get expired transactions
    const expired = await client.transactions.list({ expiredBefore: 1000 });
} catch (error) {
    console.error("Failed to filter transactions:", error.message);
}
```

## Transaction Statuses

Transactions can have the following statuses:
- **Pending** — Transaction is waiting to be processed
- **Committed** — Transaction has been successfully included in a block
- **Discarded** — Transaction was discarded and will not be processed

Check status using methods on the `TransactionStatus` object:
- `isPending()` — Returns `true` if the transaction is pending
- `isCommitted()` — Returns `true` if the transaction is committed
- `isDiscarded()` — Returns `true` if the transaction is discarded
- `getBlockNum()` — Returns the block number if committed, otherwise `null`
- `getCommitTimestamp()` — Returns the commit timestamp if committed, otherwise `null`
