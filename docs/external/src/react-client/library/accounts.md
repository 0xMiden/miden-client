---
title: Accounts
sidebar_position: 2
---

# Accounts

## useAccounts()

List all tracked accounts, automatically categorized into wallets and faucets.

```tsx
import { useAccounts } from "@miden-sdk/react";

function AccountList() {
  const { accounts, wallets, faucets, isLoading, error, refetch } = useAccounts();

  return (
    <div>
      <h3>Wallets ({wallets.length})</h3>
      {wallets.map((w) => (
        <div key={w.id().toString()}>{w.id().toString()}</div>
      ))}
      <h3>Faucets ({faucets.length})</h3>
      {faucets.map((f) => (
        <div key={f.id().toString()}>{f.id().toString()}</div>
      ))}
    </div>
  );
}
```

Auto-fetches on mount and refreshes after each sync cycle.

## useAccount(accountId)

Get full account details including vault assets enriched with metadata (symbol, decimals).

```tsx
import { useAccount, formatAssetAmount } from "@miden-sdk/react";

function AccountDetails({ accountId }: { accountId: string }) {
  const { account, assets, isLoading, getBalance } = useAccount(accountId);

  if (isLoading) return <div>Loading...</div>;
  if (!account) return <div>Not found</div>;

  return (
    <div>
      <div>Address: {account.bech32id()}</div>
      <div>Nonce: {account.nonce().toString()}</div>
      {assets.map((asset) => (
        <div key={asset.assetId}>
          {asset.symbol ?? asset.assetId}: {formatAssetAmount(asset.amount, asset.decimals)}
        </div>
      ))}
    </div>
  );
}
```

### Return value

| Property | Type | Description |
|----------|------|-------------|
| `account` | `Account \| null` | Full account object |
| `assets` | `AssetWithMetadata[]` | Vault assets with symbol and decimals |
| `isLoading` | `boolean` | Loading state |
| `error` | `Error \| null` | Error state |
| `refetch` | `() => void` | Manual refresh |
| `getBalance` | `(assetId: string) => bigint` | Get balance for a specific asset |

## useCreateWallet()

Create a new wallet account.

```tsx
import { useCreateWallet } from "@miden-sdk/react";

const { createWallet, wallet, isCreating, error, reset } = useCreateWallet();

const account = await createWallet({
  storageMode: "private",     // "private" | "public" (default: "private")
  mutable: true,              // Code updatable (default: true)
  authScheme: AuthScheme.AuthRpoFalcon512,  // default
  initSeed: new Uint8Array(32),  // Optional deterministic seed
});
```

## useCreateFaucet()

Create a new faucet account for minting tokens.

```tsx
import { useCreateFaucet } from "@miden-sdk/react";

const { createFaucet, faucet, isCreating, error, reset } = useCreateFaucet();

const account = await createFaucet({
  tokenSymbol: "TEST",
  decimals: 8,                // default: 8
  maxSupply: 10_000_000n,
  storageMode: "public",      // default for faucets
});
```

## useImportAccount()

Import an account by file, ID, or seed.

```tsx
import { useImportAccount } from "@miden-sdk/react";

const { importAccount, account, isImporting, error, reset } = useImportAccount();

// Import by file
await importAccount({ type: "file", file: accountFile });

// Import by ID (public accounts only)
await importAccount({ type: "id", accountId: "0x1234..." });

// Import by seed (public accounts only)
await importAccount({ type: "seed", seed: initSeed, mutable: true });
```

## useAssetMetadata(assetIds)

Fetch token metadata (symbol, decimals) for a list of faucet IDs. Results are cached globally.

```tsx
import { useAssetMetadata } from "@miden-sdk/react";

function TokenInfo({ assetIds }: { assetIds: string[] }) {
  const { assetMetadata } = useAssetMetadata(assetIds);

  return (
    <div>
      {assetIds.map((id) => {
        const meta = assetMetadata.get(id);
        return <div key={id}>{meta?.symbol ?? id} ({meta?.decimals ?? "?"} decimals)</div>;
      })}
    </div>
  );
}
```
