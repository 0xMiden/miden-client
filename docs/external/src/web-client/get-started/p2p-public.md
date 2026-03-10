---
title: Public peer-to-peer transfer
sidebar_position: 2
---

# Public peer-to-peer transfer

In this tutorial, you'll send tokens between two accounts using public notes. Public notes are discoverable by any client that syncs with the network, so the recipient doesn't need any out-of-band coordination.

:::info Prerequisites
- Complete the [Create account and use faucet](./create-account-use-faucet.md) tutorial first.
- You should have a funded wallet from that tutorial.
:::

## 1. Set up sender and recipient

Use the funded wallet from the previous tutorial as the sender, and create a new wallet as the recipient:

```typescript
import { MidenClient } from "@miden-sdk/ts-sdk";

const client = await MidenClient.createTestnet();

// Sender wallet (already funded from faucet tutorial)
const sender = await client.accounts.get("0xSENDER_ID...");

// Create a new recipient wallet with public storage
const recipient = await client.accounts.create({ storage: "public" });

console.log("Sender:", sender.id().toString());
console.log("Recipient:", recipient.id().toString());
```

## 2. Send tokens with a public note

Transfer tokens from the sender to the recipient using a public note:

```typescript
const faucetId = "0xFAUCET_ID...";

const txId = await client.transactions.send({
  account: sender,
  to: recipient,
  token: faucetId,
  amount: 50n,
  type: "public"
});

console.log("Send transaction:", txId.toString());
```

Wait for confirmation:

```typescript
await client.transactions.waitFor(txId.toHex());
```

## 3. Sync and consume on the recipient

Sync the client to discover the public note, then consume it on the recipient account:

```typescript
await client.sync();

const result = await client.transactions.consumeAll({ account: recipient });
console.log(`Consumed ${result.consumed} notes`);

if (result.txId) {
  await client.transactions.waitFor(result.txId.toHex());
}
```

## 4. Verify balances

```typescript
await client.sync();

const senderBalance = await client.accounts.getBalance(sender, faucetId);
const recipientBalance = await client.accounts.getBalance(recipient, faucetId);

console.log(`Sender balance: ${senderBalance}`);
console.log(`Recipient balance: ${recipientBalance}`);
```

## Complete example

```typescript
import { MidenClient } from "@miden-sdk/ts-sdk";

try {
  const client = await MidenClient.createTestnet();

  // Assume sender is already created and funded
  const sender = await client.accounts.get("0xSENDER_ID...");
  const recipient = await client.accounts.create({ storage: "public" });
  const faucetId = "0xFAUCET_ID...";

  // Send tokens via public note
  const txId = await client.transactions.send({
    account: sender,
    to: recipient,
    token: faucetId,
    amount: 50n,
    type: "public"
  });
  await client.transactions.waitFor(txId.toHex());

  // Recipient syncs and consumes
  await client.sync();
  const result = await client.transactions.consumeAll({ account: recipient });
  if (result.txId) {
    await client.transactions.waitFor(result.txId.toHex());
  }

  // Verify
  await client.sync();
  const balance = await client.accounts.getBalance(recipient, faucetId);
  console.log(`Recipient balance: ${balance}`);

  client.terminate();
} catch (error) {
  console.error("Error:", error.message);
}
```

## Next steps

- [Private peer-to-peer transfer](./p2p-private.md) — send tokens privately using note transport
