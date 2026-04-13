---
title: Create a wallet
sidebar_position: 2
---

# Create a wallet

In this tutorial, you'll create a Miden wallet, request tokens from the public faucet, and display the balance — all through React hooks.

:::info Prerequisites
Complete the [provider setup](./setup-provider.md) first.
:::

## 1. Create an account

Use the `useCreateWallet` hook:

```tsx
import { useCreateWallet, useAccounts } from "@miden-sdk/react";

function CreateWallet() {
  const { createWallet, wallet, isCreating, error } = useCreateWallet();
  const { wallets } = useAccounts();

  if (wallets.length > 0) {
    return <div>Wallet: {wallets[0].id().toString()}</div>;
  }

  return (
    <div>
      <button onClick={() => createWallet()} disabled={isCreating}>
        {isCreating ? "Creating..." : "Create Wallet"}
      </button>
      {wallet && <div>Created: {wallet.id().toString()}</div>}
      {error && <div>Error: {error.message}</div>}
    </div>
  );
}
```

## 2. Request tokens from the faucet

1. Copy the account ID displayed after creation.
2. Navigate to the [Miden faucet website](https://faucet.testnet.miden.io/).
3. Paste the account ID and click **Send Public Note**.

:::tip
Use **Send Public Note** — public notes are discoverable via sync, so no file download is needed.
:::

## 3. Consume the tokens

After the faucet sends the note, the next auto-sync cycle will discover it. Use `useNotes` and `useConsume` to claim the tokens:

```tsx
import { useNotes, useConsume } from "@miden-sdk/react";

function ClaimNotes({ accountId }: { accountId: string }) {
  const { consumableNoteSummaries } = useNotes({ accountId });
  const { consume, isLoading, stage } = useConsume();

  if (consumableNoteSummaries.length === 0) {
    return <div>No notes to claim. Waiting for sync...</div>;
  }

  const handleClaim = () => {
    const noteIds = consumableNoteSummaries.map((s) => s.id);
    consume({ accountId, notes: noteIds });
  };

  return (
    <div>
      <div>{consumableNoteSummaries.length} note(s) available</div>
      <button onClick={handleClaim} disabled={isLoading}>
        {isLoading ? `Claiming (${stage})...` : "Claim All"}
      </button>
    </div>
  );
}
```

## 4. Display the balance

Use `useAccount` to read the wallet's assets:

```tsx
import { useAccount, formatAssetAmount } from "@miden-sdk/react";

function Balance({ accountId }: { accountId: string }) {
  const { account, assets, isLoading } = useAccount(accountId);

  if (isLoading) return <div>Loading...</div>;
  if (!account) return <div>Account not found</div>;

  return (
    <div>
      <div>Account: {account.bech32id()}</div>
      {assets.map((asset) => (
        <div key={asset.assetId}>
          {asset.symbol ?? asset.assetId}: {formatAssetAmount(asset.amount, asset.decimals)}
        </div>
      ))}
    </div>
  );
}
```

## Next steps

- [Send and receive tokens](./send-receive.md) — transfer tokens between accounts
