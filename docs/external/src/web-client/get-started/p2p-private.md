---
title: Private peer-to-peer transfer
sidebar_position: 3
---

# Private peer-to-peer transfer

In this tutorial, you'll send tokens between two accounts using private notes. Unlike public notes, private notes are not visible on-chain — only the note's commitment is recorded. This provides privacy for both the sender and recipient.

:::info Prerequisites
- Complete the [Create account and use faucet](./create-account-use-faucet.md) tutorial first.
- You should have a funded wallet from that tutorial.
:::

## 1. Set up sender and recipient

Use the funded wallet from the previous tutorial as the sender, and create a new recipient wallet:

```typescript
import { MidenClient } from "@miden-sdk/ts-sdk";

const client = await MidenClient.createTestnet();

// Sender wallet (already funded)
const sender = await client.accounts.get("0xSENDER_ID...");

// Create a new recipient wallet (private storage by default)
const recipient = await client.accounts.create();

console.log("Sender:", sender.id().toString());
console.log("Recipient:", recipient.id().toString());
```

## 2. Send tokens with a private note

Transfer tokens from the sender to the recipient using a private note:

```typescript
const faucetId = "0xFAUCET_ID...";

const txId = await client.transactions.send({
  account: sender,
  to: recipient,
  token: faucetId,
  amount: 50n,
  type: "private"
});

console.log("Send transaction:", txId.toString());
await client.transactions.waitFor(txId.toHex());
```

## 3. Sync and consume

Since both accounts are managed by the same client in this example, the private note details are already available locally. Sync and consume:

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

## Using note transport for cross-client transfers

When the sender and recipient use different clients (e.g., different users), the note transport network is needed to deliver the private note to the recipient.

### Sender side

After creating the private transaction, send the note through the note transport network:

```typescript
// Get the note ID from the transaction's output notes
const sentNotes = await client.notes.listSent({ status: "committed" });
const noteId = sentNotes[0].id().toString();

// Send via note transport — requires the recipient's bech32 address
await client.notes.sendPrivate({
  noteId: noteId,
  to: "mtst1recipient..."
});
```

### Recipient side

The recipient fetches private notes and consumes them:

```typescript
// Fetch private notes from the note transport network
await client.notes.fetchPrivate();

// Sync to update local state
await client.sync();

// Consume the received note
const result = await client.transactions.consumeAll({ account: recipient });
console.log(`Consumed ${result.consumed} notes`);
```

:::note
The client fetches notes for tracked note tags. By default, note tags are derived from the recipient's account ID. For increased privacy, random tags can be used — track a specific tag with `client.tags.add(tag)`.
:::

## Complete example

```typescript
import { MidenClient } from "@miden-sdk/ts-sdk";

try {
  const client = await MidenClient.createTestnet();

  const sender = await client.accounts.get("0xSENDER_ID...");
  const recipient = await client.accounts.create();
  const faucetId = "0xFAUCET_ID...";

  // Send tokens via private note
  const txId = await client.transactions.send({
    account: sender,
    to: recipient,
    token: faucetId,
    amount: 50n,
    type: "private"
  });
  await client.transactions.waitFor(txId.toHex());

  // Sync and consume
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

- [Mock client](./mock-client.md) — test the full workflow locally without a network connection
