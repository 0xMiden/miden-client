---
title: Transactions
sidebar_position: 3
---

# Transactions

All transaction hooks follow the same pattern: they return a mutation function, a `result`, loading state, a `stage` for progress tracking, and an `error`. Transactions auto-sync before executing (unless `skipSync: true`) and refresh the store on completion.

### Transaction stages

Every mutation hook exposes a `stage` property:

`"idle"` → `"executing"` → `"proving"` → `"submitting"` → `"complete"`

## useSend()

Send tokens to another account.

```tsx
import { useSend } from "@miden-sdk/react-sdk";

const { send, result, isLoading, stage, error, reset } = useSend();

await send({
  from: senderAccountId,
  to: recipientAccountId,
  assetId: faucetId,
  amount: 100n,
  noteType: "private",         // "private" | "public" (default: "private")
  recallHeight: 100,           // Optional: block height for reclaim
  timelockHeight: 90,          // Optional: block height for timelock
  attachment: [1n, 2n, 3n],    // Optional: arbitrary data payload
  skipSync: false,             // Skip pre-sync (default: false)
  sendAll: false,              // Send entire balance (default: false)
});
```

`result` contains `{ txId, note }` on success.

## useMultiSend()

Send tokens to multiple recipients in a single transaction.

```tsx
import { useMultiSend } from "@miden-sdk/react-sdk";

const { mutate: multiSend, isLoading, stage, error } = useMultiSend();

await multiSend({
  from: senderAccountId,
  assetId: faucetId,
  recipients: [
    { to: "0xrecipient1...", amount: 50n },
    { to: "0xrecipient2...", amount: 30n, noteType: "public" },
  ],
});
```

## useInternalTransfer()

Transfer tokens between accounts managed by the same client.

```tsx
import { useInternalTransfer } from "@miden-sdk/react-sdk";

const { mutate: transfer, isLoading, stage, error } = useInternalTransfer();

const { createTransactionId, consumeTransactionId, noteId } = await transfer({
  from: senderAccountId,
  to: recipientAccountId,
  assetId: faucetId,
  amount: 100n,
});
```

Also supports chained transfers through multiple accounts:

```tsx
await transfer({
  from: senderAccountId,
  recipients: [account1, account2, account3],  // Pass-through chain
  assetId: faucetId,
  amount: 100n,
});
```

## useConsume()

Consume specific notes to receive their assets.

```tsx
import { useConsume } from "@miden-sdk/react-sdk";

const { consume, result, isLoading, stage, error, reset } = useConsume();

await consume({
  accountId: walletId,
  notes: [noteId1, noteId2],  // Hex strings, NoteId objects, or InputNoteRecords
});
```

## useMint()

Mint tokens from a faucet.

```tsx
import { useMint } from "@miden-sdk/react-sdk";

const { mint, result, isLoading, stage, error, reset } = useMint();

await mint({
  targetAccountId: recipientId,
  faucetId: faucetId,
  amount: 1000n,
  noteType: "public",          // default: "public"
});
```

## useSwap()

Swap tokens between two faucets.

```tsx
import { useSwap } from "@miden-sdk/react-sdk";

const { swap, result, isLoading, stage, error, reset } = useSwap();

await swap({
  accountId: walletId,
  offeredFaucetId: faucetA,
  offeredAmount: 100n,
  requestedFaucetId: faucetB,
  requestedAmount: 200n,
  noteType: "public",
});
```

## useTransaction()

Execute an arbitrary `TransactionRequest` — the escape hatch for custom transactions.

```tsx
import { useTransaction } from "@miden-sdk/react-sdk";

const { execute, result, isLoading, stage, error, reset } = useTransaction();

// With a static request
await execute({
  accountId: walletId,
  request: transactionRequest,
});

// With a factory function (receives the client for dynamic building)
await execute({
  accountId: walletId,
  request: async (client) => {
    // Build request dynamically
    return new TransactionRequestBuilder()
      .withCustomScript(script)
      .build();
  },
});
```

## useTransactionHistory()

Query transaction history with optional filtering.

```tsx
import { useTransactionHistory } from "@miden-sdk/react-sdk";

// All transactions
const { records, isLoading } = useTransactionHistory();

// Single transaction by ID
const { record, status } = useTransactionHistory({ id: txId });
// status: "pending" | "committed" | "discarded"

// Filtered
const { records } = useTransactionHistory({
  filter: TransactionFilter.Uncommitted,
  refreshOnSync: true,
});
```
