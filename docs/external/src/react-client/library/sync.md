---
title: Sync
sidebar_position: 5
---

# Sync

## useSyncState()

Access sync progress and trigger manual synchronization.

```tsx
import { useSyncState } from "@miden-sdk/react";

function SyncStatus() {
  const { syncHeight, isSyncing, lastSyncTime, error, sync } = useSyncState();

  return (
    <div>
      <div>Block: {syncHeight}</div>
      <div>Last sync: {lastSyncTime ? new Date(lastSyncTime).toLocaleTimeString() : "never"}</div>
      <button onClick={sync} disabled={isSyncing}>
        {isSyncing ? "Syncing..." : "Sync Now"}
      </button>
      {error && <div>Sync error: {error.message}</div>}
    </div>
  );
}
```

:::tip
Auto-sync runs in the background at the interval configured in `MidenProvider` (default: 15 seconds). Use `sync()` for manual triggers.
:::

## useWaitForCommit()

Wait for a transaction to be committed on chain. Polls until the transaction status changes or times out.

```tsx
import { useWaitForCommit } from "@miden-sdk/react";

const { mutate: waitForCommit, isLoading, error } = useWaitForCommit();

await waitForCommit({
  transactionId: txId,
  timeoutMs: 10000,    // default: 10000
  intervalMs: 1000,    // default: 1000
});
```

## useWaitForNotes()

Wait for a minimum number of consumable notes to arrive for an account. Useful after requesting faucet tokens or receiving a transfer.

```tsx
import { useWaitForNotes } from "@miden-sdk/react";

const { mutate: waitForNotes, isLoading, error } = useWaitForNotes();

await waitForNotes({
  accountId: walletId,
  minCount: 1,         // default: 1
  timeoutMs: 10000,    // default: 10000
  intervalMs: 1000,    // default: 1000
});
```

## useSessionAccount()

Manage a temporary session wallet lifecycle: create → fund → consume → ready. Persists state in localStorage for recovery across page reloads.

```tsx
import { useSessionAccount } from "@miden-sdk/react";

const {
  initialize,        // Start the create → fund → consume flow
  sessionAccountId,  // bech32 ID when ready, null otherwise
  isReady,           // true when funded and ready to use
  step,              // "idle" | "creating" | "funding" | "consuming" | "ready"
  error,
  reset,
} = useSessionAccount({
  fund: async (sessionAccountId) => {
    // Your funding logic — e.g., send from main wallet
    await send({ from: mainWalletId, to: sessionAccountId, assetId, amount: 100n });
  },
  assetId: faucetId,
  pollIntervalMs: 3000,      // default: 3000
  maxWaitMs: 60000,           // default: 60000
  storagePrefix: "miden-session",  // localStorage key prefix
});

// Start the session
await initialize();
```
