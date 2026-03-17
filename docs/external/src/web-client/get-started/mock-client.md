---
title: Mock client
sidebar_position: 4
---

# Mock client

The mock client lets you test Miden SDK workflows entirely locally — no network connection or testnet access required. It simulates the Miden blockchain in-memory, making it ideal for development, testing, and experimentation.

## How it differs from the real client

The mock client provides the same resource-based API (`client.accounts.*`, `client.transactions.*`, etc.) with one key difference: the simulated environment does not create blocks automatically. You must call `client.proveBlock()` to advance the mock chain before syncing.

## 1. Create the mock client

```typescript
import { MidenClient, AccountType } from "@miden-sdk/ts-sdk";

const client = await MidenClient.createMock();
```

## 2. Create a wallet and faucet

Since there's no testnet faucet, you create your own faucet account:

```typescript
const wallet = await client.accounts.create();
const faucet = await client.accounts.create({
  type: AccountType.FungibleFaucet,
  symbol: "TEST",
  decimals: 8,
  maxSupply: 10_000_000n
});

// Advance the chain so the accounts are committed
client.proveBlock();
await client.sync();
```

## 3. Mint tokens

Mint tokens from your faucet to the wallet:

```typescript
await client.transactions.mint({
  account: faucet,
  to: wallet,
  amount: 1000n
});

// Advance and sync
client.proveBlock();
await client.sync();
```

## 4. Consume notes

Consume all available notes to receive the minted tokens:

```typescript
const result = await client.transactions.consumeAll({ account: wallet });
console.log(`Consumed ${result.consumed} notes`);

// Advance and sync to confirm
client.proveBlock();
await client.sync();
```

## 5. Verify the balance

```typescript
const balance = await client.accounts.getBalance(wallet, faucet);
console.log(`Balance: ${balance}`); // 1000
```

## Complete example

```typescript
import { MidenClient, AccountType } from "@miden-sdk/ts-sdk";

try {
  const client = await MidenClient.createMock();

  // Create accounts
  const wallet = await client.accounts.create();
  const faucet = await client.accounts.create({
    type: AccountType.FungibleFaucet,
    symbol: "TEST",
    decimals: 8,
    maxSupply: 10_000_000n
  });

  client.proveBlock();
  await client.sync();

  // Mint tokens
  await client.transactions.mint({
    account: faucet,
    to: wallet,
    amount: 1000n
  });

  client.proveBlock();
  await client.sync();

  // Consume notes
  const result = await client.transactions.consumeAll({ account: wallet });
  console.log(`Consumed ${result.consumed} notes`);

  client.proveBlock();
  await client.sync();

  // Verify balance
  const balance = await client.accounts.getBalance(wallet, faucet);
  console.log(`Balance: ${balance}`);

  client.terminate();
} catch (error) {
  console.error("Error:", error.message);
}
```

## Mock-only methods

| Method | Description |
|--------|-------------|
| `client.proveBlock()` | Advance the mock chain by one block |
| `client.usesMockChain()` | Returns `true` for mock clients |
| `client.serializeMockChain()` | Serialize mock chain state for snapshot/restore |
| `client.serializeMockNoteTransportNode()` | Serialize mock note transport state |

## Restoring from a snapshot

You can save and restore mock chain state:

```typescript
// Snapshot
const chainState = client.serializeMockChain();
const transportState = client.serializeMockNoteTransportNode();

// Restore in a new client
const restored = await MidenClient.createMock({
  serializedMockChain: chainState,
  serializedNoteTransport: transportState
});
```

## Testing note transport

The mock client includes a built-in note transport layer for testing private note delivery:

```typescript
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

## Next steps

- [Library reference](../library/index.md) — detailed API documentation by topic
- [Examples](../examples.md) — practical code examples for common use cases
