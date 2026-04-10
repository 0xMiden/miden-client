---
title: Working with the mock client
draft: true
sidebar_position: 12
---

# Working with the Mock Client

The mock client is useful for testing and development, as it simulates interactions with the Miden blockchain without requiring a live network connection.

The mock client provides the same resource-based API as the real client (`client.accounts.*`, `client.transactions.*`, etc.) plus mock-specific methods like `proveBlock()`. The simulated environment does not create blocks automatically, so `proveBlock()` must be called manually.

## Basic Usage

```typescript
import { MidenClient, AccountType } from "@miden-sdk/miden-sdk";

try {
    // Create a mock client
    const client = await MidenClient.createMock();

    // Create accounts (same API as the real client)
    const wallet = await client.accounts.create();
    const faucet = await client.accounts.create({
        type: AccountType.FungibleFaucet,
        symbol: "TEST",
        decimals: 8,
        maxSupply: 10_000_000n
    });

    // Advance the mock chain
    client.proveBlock();
    await client.sync();

    // Mint tokens
    await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 1000n
    });

    // Advance and sync again
    client.proveBlock();
    await client.sync();

    // Consume all available notes
    const result = await client.transactions.consumeAll({ account: wallet });
    console.log(`Consumed ${result.consumed} notes`);

    // Advance and sync to confirm
    client.proveBlock();
    await client.sync();

    // Check balance
    const balance = await client.accounts.getBalance(wallet, faucet);
    console.log(`Balance: ${balance}`);
} catch (error) {
    console.error("Error:", error.message);
}
```

## Dummy Proving

When using the mock client, transaction proving is automatically handled with dummy proofs. Instead of generating real zero-knowledge proofs (which is computationally expensive), the mock client uses `LocalTransactionProver.prove_dummy()` under the hood. This means:

- No prover configuration is needed — just call transaction methods as usual.
- Transaction proving is near-instant, making tests significantly faster.
- Dummy proofs are accepted by the mock chain but would not be valid on a real network.

This applies to all transaction flows: both the simplified API (`client.transactions.mint()`, `client.transactions.send()`, etc.) and the manual `proveTransaction()` method. If no custom prover is passed and the client is a mock client, dummy proving is used automatically.

## Mock-Only Methods

- `client.proveBlock()` — Advances the mock chain by one block
- `client.usesMockChain()` — Returns `true` for mock clients
- `client.serializeMockChain()` — Serializes mock chain state for snapshot/restore
- `client.serializeMockNoteTransportNode()` — Serializes mock note transport state

## Restoring from Snapshot

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

// Create and use a mock client
const client = await MidenClient.createMock();
// ... do some operations ...

// Snapshot the state
const chainState = client.serializeMockChain();
const transportState = client.serializeMockNoteTransportNode();

// Restore in a new client
const restored = await MidenClient.createMock({
    serializedMockChain: chainState,
    serializedNoteTransport: transportState
});
```

## Working with Note Transport

The mock client includes a built-in note transport layer for testing private note delivery:

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

const client = await MidenClient.createMock();

// Send a private note
await client.notes.sendPrivate({
    noteId: "0xnote...",
    to: "mtst1recipient..."
});

// Fetch private notes
await client.notes.fetchPrivate();

// List received notes
const notes = await client.notes.list();
console.log(`Received ${notes.length} notes`);
```
