---
title: New-transactions
sidebar_position: 8
---

# Creating Transactions with the Miden SDK

This guide demonstrates how to create and submit different types of transactions using the Miden SDK. We'll cover minting, sending, consuming, and swapping.

## Basic Transaction Flow

The simplified API handles the full transaction lifecycle automatically (execute, prove, submit). Each transaction method returns a transaction ID.

```typescript
import { MidenClient, AccountType } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const faucet = await client.accounts.create({
        type: AccountType.FungibleFaucet, symbol: "TEST", decimals: 8, maxSupply: 10_000_000n
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
    await client.transactions.waitFor(mintTxId.toHex());
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
        type: "private",           // "public" or "private" (default: "public")
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
        type: "private"            // Optional (default: "public")
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
        type: "public"
    });
    console.log("Swap transaction:", txId.toString());
} catch (error) {
    console.error("Swap failed:", error.message);
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
    await client.transactions.waitFor(txId.toHex());

    // Wait with custom options
    await client.transactions.waitFor(txId.toHex(), {
        timeout: 120_000,   // 2 minutes
        interval: 3_000,    // Check every 3 seconds
        onProgress: (status) => {
            console.log(`Status: ${status}`); // "pending", "submitted", or "committed"
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

## Custom Script Transactions

Use `transactions.execute()` to run a custom MASM transaction script against an account. Compile the script first with [`client.compile.txScript()`](./compile.md).

```typescript
import { MidenClient, AccountType, AuthSecretKey, StorageSlot } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Compile the script (and any libraries it depends on)
    const script = await client.compile.txScript({
        code: `
            use external_contract::counter_contract
            begin
                call.counter_contract::increment_count
            end
        `,
        libraries: [
            { namespace: "external_contract::counter_contract", code: counterContractCode }
        ]
    });

    // Execute the script against the contract account
    const txId = await client.transactions.execute({
        account: contractAccount.id(),
        script
    });

    console.log("Transaction ID:", txId.toHex());
} catch (error) {
    console.error("Custom transaction failed:", error.message);
}
```

### Foreign Procedure Invocation (FPI)

Pass `foreignAccounts` to allow the transaction to read state from other contracts:

```typescript
try {
    const client = await MidenClient.create();

    // Get the procedure hash of the foreign contract's function
    const counterComponent = await client.compile.component({
        code: counterContractCode,
        slots: [StorageSlot.emptyValue("miden::tutorials::counter")]
    });
    const getCountHash = counterComponent.getProcedureHash("get_count");

    // Compile the FPI script
    const script = await client.compile.txScript({
        code: `
            use external_contract::count_reader_contract
            use miden::core::sys
            begin
                push.${getCountHash}
                push.${counterAccount.id().suffix()}
                push.${counterAccount.id().prefix()}
                call.count_reader_contract::copy_count
                exec.sys::truncate_stack
            end
        `,
        libraries: [
            // "dynamic" linking (default) — foreign contract code lives on-chain
            { namespace: "external_contract::count_reader_contract", code: countReaderCode }
        ]
    });

    const txId = await client.transactions.execute({
        account: countReaderAccount.id(),
        script,
        foreignAccounts: [
            // Bare AccountRef — client fetches storage requirements automatically
            counterAccount.id(),
            // Or with explicit storage requirements:
            // { id: counterAccount.id(), storage: requirements }
        ]
    });

    console.log("FPI transaction:", txId.toHex());
} catch (error) {
    console.error("FPI transaction failed:", error.message);
}
```

### Advanced: Manual Transaction Request

For full control over note inputs/outputs, build a `TransactionRequest` manually and call `submit()`:

```typescript
import { MidenClient, TransactionRequestBuilder } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const request = new TransactionRequestBuilder()
        .withCustomScript(transactionScript)
        .withOwnOutputNotes(outputNotes)
        .withExpectedOutputNotes(expectedNotes)
        .build();

    const txId = await client.transactions.submit(wallet, request);
    console.log("Custom transaction:", txId.toString());
} catch (error) {
    console.error("Custom transaction failed:", error.message);
}
```

:::note
Custom transactions require understanding of the Miden VM and its instruction set. See the integration tests in [`new_transactions.test.ts`](https://github.com/0xMiden/miden-client/blob/main/crates/web-client/test/new_transactions.test.ts) for examples.
:::
