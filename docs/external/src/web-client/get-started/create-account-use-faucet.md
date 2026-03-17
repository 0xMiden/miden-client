---
title: Create account
sidebar_position: 1
---

# Create account and use faucet

In this tutorial, you'll create a new Miden wallet using the TypeScript SDK, request tokens from the public faucet, and consume them into your account.

## Prerequisites

- Completed [Installation](../install.md)
- Access to the [Miden faucet website](https://faucet.testnet.miden.io/)

## 1. Initialize the client

Create a client connected to the Miden testnet:

```typescript
import { MidenClient } from "@miden-sdk/ts-sdk";

// createTestnet() pre-configures the RPC endpoint and enables auto-sync
const client = await MidenClient.createTestnet();
```

## 2. Create a new wallet

Create a mutable wallet with private storage (the default):

```typescript
const wallet = await client.accounts.create();

console.log("Account ID:", wallet.id().toString());
console.log("Is private:", wallet.isPrivate());   // true
console.log("Is faucet:", wallet.isFaucet());       // false
```

Save the account ID — you'll need it when requesting tokens from the faucet.

## 3. Request tokens from the faucet

1. Navigate to the [Miden faucet website](https://faucet.testnet.miden.io/).
2. Paste your account ID into the **Request test tokens** field.
3. Choose an amount and click **Send Public Note**.

:::tip
Click **Send Public Note** rather than **Send Private Note**. Public notes are discoverable via sync, so you won't need to download and import a note file.
:::

## 4. Sync the client

After the faucet sends the note, sync your client to discover it:

```typescript
const summary = await client.sync();

console.log("Synced to block:", summary.blockNum());
console.log("Committed notes:", summary.committedNotes());
```

The sync retrieves the public note created by the faucet.

## 5. Consume the note

Consume all available notes for your wallet to receive the tokens:

```typescript
const result = await client.transactions.consumeAll({ account: wallet });

console.log(`Consumed ${result.consumed} notes`);
if (result.txId) {
  console.log("Transaction ID:", result.txId.toString());
}
```

Wait for the transaction to be confirmed:

```typescript
if (result.txId) {
  await client.transactions.waitFor(result.txId.toHex());
}
```

## 6. Verify the balance

Sync again to get the updated state, then check the balance:

```typescript
await client.sync();

// Use the faucet account ID from the faucet website
const balance = await client.accounts.getBalance(wallet, "0xFAUCET_ID...");
console.log(`Balance: ${balance}`);
```

## Complete example

```typescript
import { MidenClient } from "@miden-sdk/ts-sdk";

try {
  // Initialize
  const client = await MidenClient.createTestnet();

  // Create wallet
  const wallet = await client.accounts.create();
  console.log("Account ID:", wallet.id().toString());

  // --- Request tokens from the faucet website using the account ID above ---

  // Sync to discover the faucet note
  await client.sync();

  // Consume notes
  const result = await client.transactions.consumeAll({ account: wallet });
  console.log(`Consumed ${result.consumed} notes`);

  // Wait for confirmation
  if (result.txId) {
    await client.transactions.waitFor(result.txId.toHex());
  }

  // Verify balance
  await client.sync();
  const balance = await client.accounts.getBalance(wallet, "0xFAUCET_ID...");
  console.log(`Balance: ${balance}`);

  // Clean up
  client.terminate();
} catch (error) {
  console.error("Error:", error.message);
}
```

## Next steps

Now that you have tokens, try transferring them:

- [Public peer-to-peer transfer](./p2p-public.md) — send tokens using public notes
- [Private peer-to-peer transfer](./p2p-private.md) — send tokens using private notes
