---
title: New-transactions
sidebar_position: 8
---

# Creating Transactions with the Miden SDK

This guide demonstrates how to create and submit different types of transactions using the Miden SDK. We'll cover minting, sending, consuming, and swapping.

## Basic Transaction Flow

The simplified API handles the full transaction lifecycle automatically (execute, prove, submit). Each transaction method returns a transaction ID.

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const faucet = await client.accounts.create({
        type: "faucet", symbol: "TEST", decimals: 8, maxSupply: 10_000_000n
    });
    const wallet = await client.accounts.create();

    // Mint tokens — all steps handled automatically
    const mintTxId = await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 1000n
    });
    console.log("Mint transaction:", mintTxId.toString());

    // Wait for confirmation
    await client.transactions.waitFor(mintTxId);
} catch (error) {
    console.error("Transaction failed:", error.message);
}
```

## Sending Tokens

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const txId = await client.transactions.send({
        account: senderWallet,
        to: recipientWallet,
        token: faucet,
        amount: 100n,
        noteType: "private",      // "public" or "private" (default: "public")
        reclaimAfter: 100,        // Optional: block height for reclaim
        timelockUntil: 90         // Optional: block height for timelock
    });
    console.log("Send transaction:", txId.toString());
} catch (error) {
    console.error("Send failed:", error.message);
}
```

## Minting Tokens

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const txId = await client.transactions.mint({
        account: faucet,          // The faucet account
        to: wallet,               // Recipient account
        amount: 1000n,            // Amount to mint
        noteType: "private"       // Optional (default: "public")
    });
    console.log("Mint transaction:", txId.toString());
} catch (error) {
    console.error("Mint failed:", error.message);
}
```

## Consuming Notes

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Consume specific notes
    const txId = await client.transactions.consume({
        account: wallet,
        notes: [noteId1, noteId2]  // Note IDs, InputNoteRecords, or Note objects
    });

    // Consume all available notes for an account
    const result = await client.transactions.consumeAll({ account: wallet });
    console.log(`Consumed ${result.consumed} notes, ${result.remaining} remaining`);
    if (result.txId) {
        console.log("Transaction:", result.txId.toString());
    }

    // Limit the number of notes consumed
    const limited = await client.transactions.consumeAll({
        account: wallet,
        maxNotes: 5
    });
} catch (error) {
    console.error("Consume failed:", error.message);
}
```

## Swap Transactions

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const txId = await client.transactions.swap({
        account: wallet,
        offer: { token: faucetA, amount: 100n },
        request: { token: faucetB, amount: 200n },
        noteType: "public"
    });
    console.log("Swap transaction:", txId.toString());
} catch (error) {
    console.error("Swap failed:", error.message);
}
```

## Mint and Consume (Combined)

A convenience method that mints, waits for confirmation, then consumes in one call:

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const txId = await client.transactions.mintAndConsume({
        account: faucet,
        to: wallet,
        amount: 1000n
    });
    console.log("Mint-and-consume transaction:", txId.toString());
} catch (error) {
    // Error includes a `step` property: "mint", "sync", or "consume"
    console.error(`Failed at step "${error.step}":`, error.message);
}
```

## Using a Remote Prover

For better performance, offload proving to a remote prover:

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    // Set a default prover URL for all transactions
    const client = await MidenClient.create({
        proverUrl: "https://prover.example.com"
    });

    // All transactions automatically use the remote prover
    const txId = await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 1000n
    });

    // Or override per-transaction
    const txId2 = await client.transactions.send({
        account: wallet,
        to: recipient,
        token: faucet,
        amount: 100n,
        prover: customProver  // TransactionProver instance
    });
} catch (error) {
    console.error("Transaction failed:", error.message);
}
```

:::note
Using a remote prover can significantly improve performance for complex transactions by offloading the computationally intensive proving work to a dedicated server.
:::

## Waiting for Confirmation

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const txId = await client.transactions.mint({
        account: faucet, to: wallet, amount: 1000n
    });

    // Wait with default settings (60s timeout, 5s interval)
    await client.transactions.waitFor(txId);

    // Wait with custom options
    await client.transactions.waitFor(txId, {
        timeout: 120_000,   // 2 minutes
        interval: 3_000,    // Check every 3 seconds
        onProgress: (status) => {
            console.log(`Block: ${status.blockNum}, committed: ${status.committed}`);
        }
    });
} catch (error) {
    console.error("Wait failed:", error.message);
}
```

## Transaction Preview

Preview a transaction without submitting it:

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const summary = await client.transactions.preview({
        operation: "send",
        account: wallet,
        to: recipient,
        token: faucet,
        amount: 100n
    });
    console.log("Preview result:", summary);
} catch (error) {
    console.error("Preview failed:", error.message);
}
```

## Custom Transactions

For advanced use cases, build a `TransactionRequest` manually and submit it:

```typescript
import {
    MidenClient,
    TransactionRequestBuilder,
    TransactionScript,
    TransactionProver
} from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Build a custom transaction request
    const request = new TransactionRequestBuilder()
        .withCustomScript(transactionScript)
        .withOwnOutputNotes(outputNotes)
        .withExpectedOutputNotes(expectedNotes)
        .build();

    // Submit the custom request
    const txId = await client.transactions.submit(wallet, request);
    console.log("Custom transaction:", txId.toString());
} catch (error) {
    console.error("Custom transaction failed:", error.message);
}
```

:::note
Custom transactions require understanding of the Miden VM and its instruction set. See the integration tests in [`new_transactions.test.ts`](https://github.com/0xMiden/miden-client/blob/main/crates/web-client/test/new_transactions.test.ts) for examples.
:::
