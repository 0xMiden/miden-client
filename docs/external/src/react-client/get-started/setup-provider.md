---
title: Set up the provider
sidebar_position: 1
---

# Set up the provider

Every React app using the Miden SDK needs a `MidenProvider` at the root. It initializes the WASM client, manages background sync, and provides context to all hooks.

## 1. Wrap your app

```tsx
import { MidenProvider } from "@miden-sdk/react";

function App() {
  return (
    <MidenProvider
      config={{ rpcUrl: "testnet" }}
      loadingComponent={<div>Loading Miden SDK...</div>}
      errorComponent={(error) => <div>Failed to initialize: {error.message}</div>}
    >
      <MyApp />
    </MidenProvider>
  );
}
```

The provider initializes the WASM binary, creates a Web Worker, and connects to the Miden testnet. Until initialization completes, `loadingComponent` is shown.

## 2. Check the connection

Use `useMiden()` inside any child component to access the client state:

```tsx
import { useMiden, useSyncState } from "@miden-sdk/react";

function Status() {
  const { isReady } = useMiden();
  const { syncHeight } = useSyncState();

  if (!isReady) return null;

  return <div>Connected — synced to block {syncHeight}</div>;
}
```

## 3. Configuration options

```tsx
<MidenProvider config={{
  rpcUrl: "testnet",           // "devnet" | "testnet" | "localhost" | custom URL
  autoSyncInterval: 15000,     // Sync every 15s (default). Set 0 to disable.
  prover: "testnet",           // "local" | "devnet" | "testnet" | custom URL
}} />
```

:::tip
Auto-sync runs in the background at the configured interval. You can also trigger a manual sync with the `sync()` function from `useSyncState()` or `useMiden()`.
:::

## Next steps

- [Create a wallet](./create-wallet.md) — create an account and request testnet tokens
