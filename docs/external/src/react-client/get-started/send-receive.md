---
title: Send and receive tokens
sidebar_position: 3
---

# Send and receive tokens

In this tutorial, you'll send tokens between two accounts using the `useSend` and `useConsume` hooks.

:::info Prerequisites
- Complete the [Create a wallet](./create-wallet.md) tutorial first.
- You should have a funded wallet.
:::

## 1. Create a second account

```tsx
import { useCreateWallet } from "@miden-sdk/react";

function CreateRecipient() {
  const { createWallet, wallet, isCreating } = useCreateWallet();

  return (
    <div>
      <button onClick={() => createWallet({ storageMode: "public" })} disabled={isCreating}>
        {isCreating ? "Creating..." : "Create Recipient"}
      </button>
      {wallet && <div>Recipient: {wallet.id().toString()}</div>}
    </div>
  );
}
```

## 2. Send tokens

Use the `useSend` hook. The `stage` property gives real-time feedback on the transaction lifecycle:

```tsx
import { useSend } from "@miden-sdk/react";

function SendForm({ from, faucetId }: { from: string; faucetId: string }) {
  const { send, isLoading, stage, error, result } = useSend();
  const [to, setTo] = useState("");
  const [amount, setAmount] = useState("");

  const handleSend = () => {
    send({
      from,
      to,
      assetId: faucetId,
      amount: BigInt(amount),
      noteType: "public",
    });
  };

  return (
    <div>
      <input placeholder="Recipient ID" value={to} onChange={(e) => setTo(e.target.value)} />
      <input placeholder="Amount" value={amount} onChange={(e) => setAmount(e.target.value)} />
      <button onClick={handleSend} disabled={isLoading}>
        {isLoading ? `Sending (${stage})...` : "Send"}
      </button>
      {result && <div>Transaction: {result.txId.toString()}</div>}
      {error && <div>Error: {error.message}</div>}
    </div>
  );
}
```

## 3. Receive and consume on the recipient

After the next sync cycle, the public note appears for the recipient. Consume it:

```tsx
import { useNotes, useConsume, formatNoteSummary } from "@miden-sdk/react";

function ReceiveNotes({ accountId }: { accountId: string }) {
  const { consumableNoteSummaries } = useNotes({ accountId });
  const { consume, isLoading, stage } = useConsume();

  return (
    <div>
      {consumableNoteSummaries.map((note) => (
        <div key={note.id}>
          {formatNoteSummary(note)}
          <button
            onClick={() => consume({ accountId, notes: [note.id] })}
            disabled={isLoading}
          >
            {isLoading ? `Consuming (${stage})...` : "Consume"}
          </button>
        </div>
      ))}
    </div>
  );
}
```

## Private transfers

To send a private note instead, set `noteType: "private"`:

```tsx
await send({
  from: senderAccountId,
  to: recipientAccountId,
  assetId: faucetId,
  amount: 50n,
  noteType: "private",
});
```

For cross-client private transfers, the note transport network delivers the note to the recipient. See the [TypeScript private transfer tutorial](../../web-client/get-started/p2p-private.md) for details on the underlying flow.

## Next steps

- [External signer](./external-signer.md) — integrate a third-party wallet provider
