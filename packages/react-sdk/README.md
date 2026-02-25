# @miden-sdk/react

React hooks library for the Miden Web Client. Provides a simple, ergonomic interface for building React applications on the Miden rollup.

## Features

- **Easy Setup** - Single provider component handles WASM initialization and client setup
- **Sensible Defaults** - Privacy-first defaults that work out of the box
- **Auto-Sync** - Automatic background synchronization with the network
- **TypeScript First** - Full type safety with comprehensive type exports
- **Consistent Patterns** - All hooks follow predictable patterns for loading, errors, and state
- **Note Attachments** - Send and read arbitrary data payloads on notes via `useSend()` and `readNoteAttachment()`
- **Temporal Note Tracking** - `useNoteStream()` tracks when notes first appear, with built-in filtering, handled-note exclusion, and phase snapshots
- **Session Wallets** - `useSessionAccount()` manages the create-fund-consume lifecycle for temporary wallets
- **Concurrency Safety** - Transaction hooks prevent double-sends with built-in concurrency guards
- **Auto Pre-Sync** - Transaction hooks sync before executing by default (opt out with `skipSync`)
- **WASM Error Wrapping** - Cryptic WASM errors are intercepted and replaced with actionable messages

## Installation

```bash
npm install @miden-sdk/react @miden-sdk/miden-sdk
# or
yarn add @miden-sdk/react @miden-sdk/miden-sdk
# or
pnpm add @miden-sdk/react @miden-sdk/miden-sdk
```

## Testing

From `packages/react-sdk`:

```bash
# Unit tests
yarn test:unit

# Integration tests (Playwright) in test/
# Build the web-client dist first:
cd ../../crates/web-client && yarn build
cd ../../packages/react-sdk
yarn playwright install --with-deps
yarn test:integration
```

## Quick Start

Wrap your app with `MidenProvider` and start using hooks:

```tsx
import { MidenProvider, useMiden, useCreateWallet, useAccounts } from '@miden-sdk/react';

function App() {
  return (
    <MidenProvider>
      <Wallet />
    </MidenProvider>
  );
}

function Wallet() {
  const { isReady } = useMiden();
  const { wallets, isLoading } = useAccounts();
  const { createWallet, isCreating } = useCreateWallet();

  if (!isReady) return <div>Initializing Miden...</div>;
  if (isLoading) return <div>Loading accounts...</div>;

  return (
    <div>
      <h2>My Wallets ({wallets.length})</h2>
      <ul>
        {wallets.map(wallet => (
          <li key={wallet.id().toString()}>
            {wallet.id().toString()}
          </li>
        ))}
      </ul>

      <button onClick={() => createWallet()} disabled={isCreating}>
        {isCreating ? 'Creating...' : 'Create Wallet'}
      </button>
    </div>
  );
}
```

## Provider Configuration

The `MidenProvider` handles WASM initialization and client setup:

```tsx
import { MidenProvider } from '@miden-sdk/react';

function App() {
  return (
    <MidenProvider
      config={{
        // RPC endpoint (defaults to testnet). You can also use 'devnet' or 'testnet'.
        rpcUrl: 'devnet',

        // Auto-sync interval in milliseconds (default: 15000)
        // Set to 0 to disable auto-sync
        autoSyncInterval: 15000,

        // Optional: prover selection ('local' | 'devnet' | 'testnet' | URL)
        // prover: 'local',
      }}
      // Optional: Custom loading component
      loadingComponent={<div>Loading Miden...</div>}

      // Optional: Custom error component
      errorComponent={(error) => (
        <div>Failed to initialize: {error.message}</div>
      )}
    >
      <YourApp />
    </MidenProvider>
  );
}
```

## Hooks Reference

### Core Hooks

**ID formats:** All hooks that accept account IDs or asset IDs accept both
bech32 and hex strings (auto-detected).

#### `useMiden()`

Access the Miden client instance and initialization state. This is your entry
point for low-level control (syncing, direct client access, and prover-aware
transactions) while still playing nicely with the provider lifecycle. It
centralizes readiness and error handling so you avoid sprinkling guards across
components. You also get a single place to hook sync and exclusive access
without wiring your own locks.

```tsx
import { useMiden } from '@miden-sdk/react';

function MyComponent() {
  const {
    client,      // WebClient instance (null if not ready)
    isReady,     // true when client is initialized
    error,       // Initialization error if any
    sync,        // Function to trigger manual sync
  } = useMiden();

  if (error) {
    return <div>Error: {error.message}</div>;
  }

  if (!isReady) {
    return <div>Initializing...</div>;
  }

  return <div>Connected! Block height: {/* ... */}</div>;
}
```

#### `useMidenClient()`

Get the ready `WebClient` instance directly. It’s a convenience for advanced
flows where you want to call SDK methods yourself without re-checking readiness
every time; it throws if the client isn't ready yet. This keeps your component
logic clean by avoiding boilerplate null checks. You still benefit from the
provider lifecycle while opting into raw SDK control.

```tsx
import { useMidenClient } from '@miden-sdk/react';

function MyComponent() {
  const client = useMidenClient();
  // Safe to use client here (initialized)
  return <div>Client ready</div>;
}
```

#### `useSyncState()`

Monitor network sync status and trigger manual syncs. Useful for UI indicators,
pull-to-refresh, or forcing a sync before running a transaction pipeline. It
wraps the shared sync lock so multiple components don't stampede the node. You
get consistent timestamps and error state without wiring your own timers.

```tsx
import { useSyncState } from '@miden-sdk/react';

function SyncStatus() {
  const {
    syncHeight,    // Current synced block height
    isSyncing,     // true during sync operation
    lastSyncTime,  // Timestamp of last successful sync
    error,         // Sync error if any
    sync,          // Function to trigger manual sync
  } = useSyncState();

  return (
    <div>
      <p>Block Height: {syncHeight}</p>
      <p>Last Sync: {lastSyncTime ? new Date(lastSyncTime).toLocaleString() : 'Never'}</p>
      <button onClick={sync} disabled={isSyncing}>
        {isSyncing ? 'Syncing...' : 'Sync Now'}
      </button>
    </div>
  );
}
```

### Account Hooks

#### `useAccounts()`

List all accounts tracked by the local client, automatically categorized into
wallets and faucets. Great for dashboards, account pickers, and quick summaries
without extra filtering logic. It hides the bit twiddling needed to recognize
faucet IDs. Refetch runs through the same sync-safe path so you avoid stale
caches or double fetches.

```tsx
import { useAccounts } from '@miden-sdk/react';

function AccountList() {
  const {
    accounts,   // All accounts
    wallets,    // Regular wallet accounts
    faucets,    // Faucet accounts
    isLoading,  // Loading state
    error,      // Error if fetch failed
    refetch,    // Function to refresh the list
  } = useAccounts();

  if (isLoading) return <div>Loading...</div>;

  return (
    <div>
      <h2>Wallets ({wallets.length})</h2>
      {wallets.map(w => (
        <div key={w.id().toString()}>
          {w.id().toString()}
        </div>
      ))}

      <h2>Faucets ({faucets.length})</h2>
      {faucets.map(f => (
        <div key={f.id().toString()}>
          {f.id().toString()}
        </div>
      ))}

      <button onClick={refetch}>Refresh</button>
    </div>
  );
}
```

#### `useAccount(accountId)`

Get detailed information for a single account, including assets. The hook
hydrates balances with token metadata (symbol/decimals) and keeps data fresh
after syncs. It spares you from manual vault parsing and metadata joins.
Balances update automatically with sync so UI stays in step.

```tsx
import { useAccount } from '@miden-sdk/react';

function AccountDetails({ accountId }: { accountId: string }) {
  const {
    account,     // Full account object
    assets,      // Array of { assetId, amount, symbol?, decimals? } balances
    isLoading,
    error,
    refetch,
    getBalance,  // Helper to get balance for specific asset
  } = useAccount(accountId);

  if (isLoading) return <div>Loading...</div>;
  if (!account) return <div>Account not found</div>;

  // Get balance for a specific token
  const usdcBalance = getBalance('0xasset123...');

  return (
    <div>
      <h2>Account: {account.id().toString()}</h2>
      <p>Nonce: {account.nonce().toString()}</p>

      <h3>Assets</h3>
      {assets.map(asset => (
        <div key={asset.assetId}>
          {asset.symbol ?? asset.assetId}: {asset.amount.toString()}
        </div>
      ))}

      <p>USDC Balance: {usdcBalance.toString()}</p>
    </div>
  );
}
```

#### `useCreateWallet()`

Create new wallet accounts. Supports storage mode, mutability, and auth scheme
so you can quickly spin up accounts for demos or customize for production needs.
It wraps ID parsing and defaults so you can start with a one-liner. The hook
also tracks creation state so you can wire UI without extra reducers.

```tsx
import { useCreateWallet } from '@miden-sdk/react';

function CreateWalletButton() {
  const {
    createWallet,  // Function to create wallet
    wallet,        // Created wallet (after success)
    isCreating,    // Loading state
    error,         // Error if creation failed
    reset,         // Reset state for new creation
  } = useCreateWallet();

  const handleCreate = async () => {
    try {
      // With defaults (private storage, mutable, Falcon auth)
      const newWallet = await createWallet();
      console.log('Created wallet:', newWallet.id().toString());

      // Or with custom options
      const customWallet = await createWallet({
        storageMode: 'private',  // 'private' | 'public' | 'network'
        mutable: true,           // Allow code updates
        authScheme: 0,           // 0 = Falcon (default), 1 = ECDSA
      });
    } catch (err) {
      console.error('Failed to create wallet:', err);
    }
  };

  return (
    <div>
      {error && (
        <div>
          Error: {error.message}
          <button onClick={reset}>Try Again</button>
        </div>
      )}

      <button onClick={handleCreate} disabled={isCreating}>
        {isCreating ? 'Creating...' : 'Create Wallet'}
      </button>

      {wallet && <div>Created: {wallet.id().toString()}</div>}
    </div>
  );
}
```

#### `useCreateFaucet()`

Create new faucets for minting tokens. Ideal for dev/test flows where you need
a controlled token source and quick bootstrap of balances.
It handles storage/auth defaults and returns a ready faucet object. That
removes the usual setup friction when you just want tokens to exist.

```tsx
import { useCreateFaucet } from '@miden-sdk/react';

function CreateFaucetForm() {
  const { createFaucet, faucet, isCreating, error, reset } = useCreateFaucet();

  const handleCreate = async () => {
    try {
      const newFaucet = await createFaucet({
        tokenSymbol: 'USDC',              // 1-4 character symbol
        decimals: 6,                       // Token decimals (default: 8)
        maxSupply: 1000000000n * 10n**6n, // Max supply in smallest units
        storageMode: 'private',            // Optional (default: 'private')
        authScheme: 0,                     // Optional (default: 0 = Falcon)
      });
      console.log('Created faucet:', newFaucet.id().toString());
    } catch (err) {
      console.error('Failed:', err);
    }
  };

  return (
    <div>
      {error && <div>Error: {error.message}</div>}
      <button onClick={handleCreate} disabled={isCreating}>
        {isCreating ? 'Creating...' : 'Create USDC Faucet'}
      </button>
    </div>
  );
}
```

#### `useImportAccount()`

Import an existing account into the client. This lets you start tracking an
on-chain account by ID, or restore a private account from a file/seed.
The hook normalizes ID formats and hides SDK branching. It also exposes the
imported account so you can update UI immediately.

```tsx
import { useImportAccount } from '@miden-sdk/react';

function ImportAccountButton({ accountId }: { accountId: string }) {
  const { importAccount, account, isImporting, error, reset } = useImportAccount();

  const handleImport = async () => {
    await importAccount({ type: 'id', accountId });
  };

  return (
    <button onClick={handleImport} disabled={isImporting}>
      {isImporting ? 'Importing...' : 'Import Account'}
    </button>
  );
}
```

### Note Hooks

#### `useNotes(options?)`

List and filter notes (incoming transactions). Includes consumable notes and
optional summaries that bundle asset metadata so you can render balances and
labels without extra lookups. It syncs notes after successful syncs so you
don't have to wire listeners. Summaries reduce boilerplate around asset
metadata and formatting.

```tsx
import { useNotes } from '@miden-sdk/react';

function NotesList() {
  const {
    notes,            // All notes matching filter
    consumableNotes,  // Notes ready to be consumed
    noteSummaries,    // Summary objects with asset metadata
    isLoading,
    error,
    refetch,
  } = useNotes();

  // With filtering options
  const { notes: committedNotes } = useNotes({
    status: 'committed',  // 'all' | 'consumed' | 'committed' | 'expected' | 'processing'
    accountId: '0x...',   // Filter by account
    sender: '0x...',      // Filter by sender (any format, normalized internally)
    excludeIds: ['0xnote1...'], // Exclude specific note IDs
  });

  return (
    <div>
      <h2>Consumable Notes ({consumableNotes.length})</h2>
      {consumableNotes.map(note => (
        <div key={note.id().toString()}>
          {note.id().toString()}
        </div>
      ))}

      <h2>Note Summaries</h2>
      {noteSummaries.map(summary => (
        <div key={summary.id}>
          {summary.id} — {summary.assets.map(a => `${a.amount} ${a.symbol ?? a.assetId}`).join(', ')}
        </div>
      ))}
    </div>
  );
}
```

#### `useNoteStream(options?)`

Temporal note tracking with a unified model. Replaces the common pattern of
`handledNoteIds` refs, deferred baselines, and dual-track note decoding.
Returns `StreamedNote` objects that merge summary data with the underlying
record and pre-decode attachments.

Features:
- **Unified `StreamedNote` type** with sender (bech32), amount, assets, attachment, and `firstSeenAt` timestamp
- **Built-in filtering** by sender, status, `since` timestamp, `excludeIds`, and custom `amountFilter`
- **`markHandled` / `markAllHandled`** to exclude processed notes without removing them from the store
- **`snapshot()`** to capture current note IDs and timestamp for passing to the next phase

```tsx
import { useNoteStream } from '@miden-sdk/react';

function IncomingNotes({ opponentId }: { opponentId: string }) {
  const { notes, latest, markHandled, snapshot } = useNoteStream({
    sender: opponentId,       // Only notes from this sender
    status: 'committed',      // 'all' | 'consumed' | 'committed' | 'expected' | 'processing'
    // since: phaseStartTime, // Only notes after this timestamp
    // excludeIds: staleIds,  // Exclude specific note IDs
    // amountFilter: (a) => a >= 100n,
  });

  useEffect(() => {
    if (latest) {
      console.log('New note!', latest.attachment);
      markHandled(latest.id);
    }
  }, [latest, markHandled]);

  // Capture state for next phase
  const handlePhaseEnd = () => {
    const snap = snapshot(); // { ids: Set<string>, timestamp: number }
    // Pass snap.ids as excludeIds or snap.timestamp as since to next phase
  };

  return <div>{notes.length} unhandled notes</div>;
}
```

The `StreamedNote` type provides:
```typescript
interface StreamedNote {
  id: string;              // Note ID (hex)
  sender: string;          // Sender account ID (bech32)
  amount: bigint;          // First fungible asset amount (0n if none)
  assets: NoteAsset[];     // All assets on the note
  record: InputNoteRecord; // Underlying record for escape-hatch access
  firstSeenAt: number;     // Timestamp (ms) when first observed
  attachment: bigint[] | null; // Pre-decoded attachment values
}
```

#### `useAssetMetadata(assetIds)`

Fetch asset symbols/decimals for a list of asset IDs. This is the lightweight
way to enrich balances and note lists with human-friendly token info.
It batches lookups and caches results for reuse across components. That avoids
repeated RPC calls and inconsistent labels.

```tsx
import { useAssetMetadata } from '@miden-sdk/react';

function AssetLabels({ assetIds }: { assetIds: string[] }) {
  const { assetMetadata } = useAssetMetadata(assetIds);
  return (
    <ul>
      {assetIds.map((id) => {
        const meta = assetMetadata.get(id);
        return (
          <li key={id}>
            {id} — {meta?.symbol ?? 'UNKNOWN'} ({meta?.decimals ?? 0})
          </li>
        );
      })}
    </ul>
  );
}
```

### Transaction Hooks

All transaction hooks follow a consistent pattern with `stage` tracking:

| Stage | Description |
|-------|-------------|
| `'idle'` | Not started |
| `'executing'` | Building/executing request |
| `'proving'` | Generating ZK proof |
| `'submitting'` | Submitting to network |
| `'complete'` | Transaction confirmed |

#### `useSend()`

Send tokens from one account to another. Handles the full lifecycle (execute,
prove, submit, apply) and delivers private notes automatically when needed.
It collapses the multi-step transaction pipeline into a single call with stage
tracking. You get private note delivery without having to remember the extra
send step.

Built-in features:
- **Auto pre-sync** before executing (disable with `skipSync: true`)
- **Concurrency guard** prevents double-sends while a transaction is in-flight
- **Attachment support** for sending arbitrary data with notes
- **`sendAll`** to send the full balance of an asset

```tsx
import { useSend } from '@miden-sdk/react';

function SendForm() {
  const {
    send,       // Function to execute send
    result,     // { transactionId } after success
    isLoading,  // true during transaction
    stage,      // Current stage
    error,
    reset,
  } = useSend();

  const handleSend = async () => {
    try {
      const { transactionId } = await send({
        from: '0xsender...',      // Sender account ID
        to: '0xrecipient...',     // Recipient account ID
        assetId: '0xtoken...',    // Asset ID (token id)
        amount: 100n,             // Amount in smallest units

        // Optional parameters
        noteType: 'private',      // 'private' | 'public' (default: 'private')
        recallHeight: 1000,       // Sender can reclaim after this block
        attachment: [1n, 2n, 3n], // Arbitrary data payload
        skipSync: false,          // Skip auto-sync before send (default: false)
        sendAll: false,           // Send full balance (ignores amount)
      });

      console.log('Sent! TX:', transactionId);
    } catch (err) {
      console.error('Send failed:', err);
    }
  };

  return (
    <div>
      {error && <div>Error: {error.message}</div>}

      <button onClick={handleSend} disabled={isLoading}>
        {isLoading ? `Sending (${stage})...` : 'Send Tokens'}
      </button>

      {result && <div>Success! TX: {result.transactionId}</div>}
    </div>
  );
}
```

#### `useMultiSend()`

Create multiple P2ID output notes in a single transaction. This is ideal for
batched payouts or airdrops; with `noteType: 'private'`, the hook also delivers
each note to recipients via `sendPrivateNote`.
It builds the request and executes the full pipeline in one go. That means
fewer chances to handle batching incorrectly or forget private note delivery.

Built-in features:
- **Auto pre-sync** before executing (disable with `skipSync: true`)
- **Concurrency guard** prevents double-sends while a transaction is in-flight
- **Per-recipient note type and attachment overrides**

```tsx
import { useMultiSend } from '@miden-sdk/react';

function MultiSendButton() {
  const { sendMany, isLoading, stage } = useMultiSend();

  const handleSend = async () => {
    await sendMany({
      from: '0xsender...',
      assetId: '0xtoken...',
      recipients: [
        { to: '0xrec1...', amount: 100n },
        { to: '0xrec2...', amount: 250n, attachment: [42n] },
      ],
      noteType: 'public',  // Default for all recipients
      skipSync: false,      // Optional: skip auto-sync (default: false)
    });
  };

  return (
    <button onClick={handleSend} disabled={isLoading}>
      {isLoading ? `Sending (${stage})...` : 'Multi-Send'}
    </button>
  );
}
```

#### `useInternalTransfer()`

Create a P2ID note and immediately consume it. This is useful for transfers
between accounts you control (e.g., public → private), since the receiver must
be available in the local client to consume the note.
It abstracts the two-transaction flow and keeps both steps tied to a single UI
action. You don't have to juggle temporary note IDs or manual syncs.

```tsx
import { useInternalTransfer } from '@miden-sdk/react';

function InternalTransferButton() {
  const { transfer, isLoading, stage } = useInternalTransfer();

  const handleTransfer = async () => {
    await transfer({
      from: '0xsender...',
      to: '0xrecipient...',
      assetId: '0xtoken...',
      amount: 50n,
      noteType: 'public',
    });
  };

  return (
    <button onClick={handleTransfer} disabled={isLoading}>
      {isLoading ? `Transferring (${stage})...` : 'Transfer'}
    </button>
  );
}
```

#### `useWaitForCommit()`

Wait for a transaction to be committed. Useful for gating UI transitions,
follow-up actions, or polling-driven workflows that depend on finality.
It handles sync + polling loops and the discarded/timeout edge cases. So you
can keep UI logic simple and avoid bespoke timers.

```tsx
import { useWaitForCommit } from '@miden-sdk/react';

function WaitForTx({ txId }: { txId: string }) {
  const { waitForCommit } = useWaitForCommit();

  const handleWait = async () => {
    await waitForCommit(txId, { timeoutMs: 10_000, intervalMs: 1_000 });
  };

  return <button onClick={handleWait}>Wait for Commit</button>;
}
```

#### `useTransactionHistory()`

Query transaction history and get live state for a single transaction. You can
pass a single ID, multiple IDs, or a custom `TransactionFilter`. Results refresh
after each successful sync by default.
It hides the filter plumbing and provides a single-record convenience view.
The hook keeps lists fresh on sync so you don't wire manual refreshes.

```tsx
import { useTransactionHistory } from '@miden-sdk/react';

function TxHistory() {
  const { records, isLoading, refetch } = useTransactionHistory();

  if (isLoading) return <div>Loading...</div>;

  return (
    <div>
      <button onClick={refetch}>Refresh</button>
      <ul>
        {records.map((record) => (
          <li key={record.id().toHex()}>{record.id().toHex()}</li>
        ))}
      </ul>
    </div>
  );
}
```

```tsx
import { useTransactionHistory } from '@miden-sdk/react';

function TxStatus({ txId }: { txId: string }) {
  const { record, status } = useTransactionHistory({ id: txId });

  if (!record) return <div>Not found</div>;

  return (
    <div>
      {record.id().toHex()} → {status}
    </div>
  );
}
```

#### `useWaitForNotes()`

Wait until an account has consumable notes. Great for mint → consume pipelines
and other flows where you want to proceed only when notes are ready.
It wraps the poll/sync loop and returns notes once the threshold is met. That
keeps workflows linear without ad-hoc sleeps.

```tsx
import { useWaitForNotes } from '@miden-sdk/react';

function WaitForNotes({ accountId }: { accountId: string }) {
  const { waitForConsumableNotes } = useWaitForNotes();

  const handleWait = async () => {
    const notes = await waitForConsumableNotes({
      accountId,
      minCount: 1,
      timeoutMs: 10_000,
      intervalMs: 1_000,
    });
    console.log('Notes ready:', notes.length);
  };

  return <button onClick={handleWait}>Wait for Notes</button>;
}
```

#### `useMint()`

Mint new tokens from a faucet you control. The hook handles the full tx pipeline
and is perfect for quickly funding accounts in dev/test environments.
Defaults and ID parsing keep the call small while still letting you tune note
type. It also tracks stages so you can surface progress without extra state.

```tsx
import { useMint } from '@miden-sdk/react';

function MintForm() {
  const { mint, result, isLoading, stage, error, reset } = useMint();

  const handleMint = async () => {
    try {
      const { transactionId } = await mint({
        faucetId: '0xmyfaucet...',      // Your faucet ID
        targetAccountId: '0xwallet...', // Recipient wallet
        amount: 1000n * 10n**8n,        // Amount to mint
        noteType: 'private',            // Optional: 'private' | 'public'
      });

      console.log('Minted! TX:', transactionId);
    } catch (err) {
      console.error('Mint failed:', err);
    }
  };

  return (
    <button onClick={handleMint} disabled={isLoading}>
      {isLoading ? `Minting (${stage})...` : 'Mint 1000 Tokens'}
    </button>
  );
}
```

#### `useConsume()`

Consume notes to claim tokens sent to your account. Supports multiple note IDs
and handles proof generation and submission automatically.
It wraps the multi-step consume pipeline so you don't have to string calls
together. That prevents common snags like forgetting to sync or submit.

```tsx
import { useConsume } from '@miden-sdk/react';

function ConsumeNotes() {
  const { consume, result, isLoading, stage, error, reset } = useConsume();

  const handleConsume = async (noteIds: string[]) => {
    try {
      const { transactionId } = await consume({
        accountId: '0xmywallet...',  // Your wallet ID
        noteIds: noteIds,             // Array of note IDs to consume
      });

      console.log('Consumed! TX:', transactionId);
    } catch (err) {
      console.error('Consume failed:', err);
    }
  };

  return (
    <button
      onClick={() => handleConsume(['0xnote1...', '0xnote2...'])}
      disabled={isLoading}
    >
      {isLoading ? `Consuming (${stage})...` : 'Claim Tokens'}
    </button>
  );
}
```

#### `useSwap()`

Create atomic swap offers. Use it to build escrow-style swaps with configurable
note types for both the swap note and the payback note.
The hook hides request construction and lets you focus on trade parameters.
Stage tracking helps you provide clear UX during proof/submission.

```tsx
import { useSwap } from '@miden-sdk/react';

function SwapForm() {
  const { swap, result, isLoading, stage, error, reset } = useSwap();

  const handleSwap = async () => {
    try {
      const { transactionId } = await swap({
        accountId: '0xmywallet...',

        // What you're offering
        offeredFaucetId: '0xtokenA...',
        offeredAmount: 100n,

        // What you want in return
        requestedFaucetId: '0xtokenB...',
        requestedAmount: 50n,

        // Optional
        noteType: 'private',        // Note type for swap note
        paybackNoteType: 'private', // Note type for payback note
      });

      console.log('Swap created! TX:', transactionId);
    } catch (err) {
      console.error('Swap failed:', err);
    }
  };

  return (
    <button onClick={handleSwap} disabled={isLoading}>
      {isLoading ? `Creating Swap (${stage})...` : 'Create Swap Offer'}
    </button>
  );
}
```

#### `useTransaction()`

Execute a custom `TransactionRequest` or build one with the client. This is the
escape hatch for advanced flows not covered by higher-level hooks.
It standardizes the execute/prove/submit flow while still letting you craft the
request. You get progress and error handling without wrapping every call
yourself.

Built-in features:
- **Auto pre-sync** before executing (disable with `skipSync: true`)
- **Concurrency guard** prevents double-executions while a transaction is in-flight

```tsx
import { useTransaction } from '@miden-sdk/react';
import { AccountId, NoteType } from '@miden-sdk/miden-sdk';

function CustomTransactionButton({ accountId }: { accountId: string }) {
  const { execute, isLoading, stage } = useTransaction();

  const handleRun = async () => {
    await execute({
      accountId,
      request: (client) =>
        client.newSwapTransactionRequest(
          AccountId.fromHex(accountId),
          AccountId.fromHex('0xassetA'),
          10n,
          AccountId.fromHex('0xassetB'),
          5n,
          NoteType.Private,
          NoteType.Private
        ),
      skipSync: false, // Optional: skip auto-sync (default: false)
    });
  };

  return (
    <button onClick={handleRun} disabled={isLoading}>
      {isLoading ? stage : 'Run Transaction'}
    </button>
  );
}
```

#### `useSessionAccount(options)`

Manage a session wallet lifecycle: create, fund, and consume in a single flow.
Replaces the common 300+ line pattern of creating a temporary wallet, waiting
for funding, and consuming the funding note. Persists the session wallet ID
to localStorage for page reloads.

```tsx
import { useSessionAccount, useSend } from '@miden-sdk/react';

function SessionWallet({ mainWalletId, assetId }: { mainWalletId: string; assetId: string }) {
  const { send } = useSend();
  const { initialize, sessionAccountId, isReady, step, error, reset } =
    useSessionAccount({
      fund: async (sessionId) => {
        // Send tokens from main wallet to session wallet
        await send({ from: mainWalletId, to: sessionId, assetId, amount: 100n });
      },
      assetId,
      // Optional:
      // walletOptions: { storageMode: 'public', mutable: true, authScheme: 0 },
      // pollIntervalMs: 3000,
      // storagePrefix: 'miden-session',
    });

  if (error) return <div>Error: {error.message} <button onClick={reset}>Retry</button></div>;
  if (isReady) return <div>Session ready: {sessionAccountId}</div>;

  return (
    <button onClick={initialize} disabled={step !== 'idle'}>
      {step === 'idle' ? 'Start Session' : `${step}...`}
    </button>
  );
}
```

Steps: `idle` -> `creating` -> `funding` -> `consuming` -> `ready`

## Utilities

The SDK exports standalone utility functions for common tasks.

### Note Attachments

Read and write arbitrary data payloads on notes:

```typescript
import { readNoteAttachment, createNoteAttachment } from '@miden-sdk/react';

// Read attachment from a note record
const data = readNoteAttachment(noteRecord);
if (data) {
  console.log(data.values); // bigint[]
  console.log(data.kind);   // 'word' (<=4 values) or 'array' (>4 values)
}

// Create attachment for sending
const attachment = createNoteAttachment([1n, 2n, 3n]);
// <= 4 values -> Word (auto-padded to 4)
// > 4 values -> Array (auto-padded to multiple of 4)
```

### Account ID Normalization

Compare and normalize account IDs across hex and bech32 formats:

```typescript
import { normalizeAccountId, accountIdsEqual } from '@miden-sdk/react';

const bech32 = normalizeAccountId('0x1234...');  // Returns bech32 format
accountIdsEqual('0x1234...', 'miden1abc...');     // true (format-agnostic)
```

### Byte Conversion

Utilities for bigint/byte conversions useful in cryptographic note data:

```typescript
import { bytesToBigInt, bigIntToBytes, concatBytes } from '@miden-sdk/react';

const n = bytesToBigInt(new Uint8Array([0x01, 0x00])); // 256n
const bytes = bigIntToBytes(256n, 2);                   // Uint8Array([0x01, 0x00])
const combined = concatBytes(bytes1, bytes2);            // Concatenated Uint8Array
```

### Storage Helpers

IndexedDB migration and namespaced localStorage persistence:

```typescript
import { migrateStorage, clearMidenStorage, createMidenStorage } from '@miden-sdk/react';

// Auto-clear stale IndexedDB when SDK version changes
await migrateStorage({
  version: '0.13.1',
  // versionKey: 'miden:storageVersion',  // default
  // reloadOnClear: true,                 // default
  // onBeforeClear: () => { /* save data */ },
});

// Namespaced localStorage for app state
const store = createMidenStorage('myapp');
store.set('lastOpponent', '0x1234...');
const opponent = store.get<string>('lastOpponent');
store.remove('lastOpponent');
store.clear(); // Clears only keys under 'myapp:' prefix
```

### Wallet Detection

Wait for a browser extension wallet to be detected before connecting:

```typescript
import { waitForWalletDetection } from '@miden-sdk/react';

const adapter = wallets[0].adapter;

// Default 5s timeout
await waitForWalletDetection(adapter);

// Custom timeout
await waitForWalletDetection(adapter, 10000);

// Returns immediately if already installed, otherwise
// listens for readyStateChange events with a timeout.
```

This replaces the common 30-line polling+timeout pattern needed when the wallet extension loads slowly. The adapter is duck-typed — any object with `readyState`, `on("readyStateChange", ...)`, and `off("readyStateChange", ...)` will work.

### Error Handling Utilities

WASM errors are cryptic. The SDK wraps common patterns with actionable messages:

```typescript
import { MidenError, wrapWasmError } from '@miden-sdk/react';

try {
  await client.someMethod(accountId);
} catch (e) {
  const wrapped = wrapWasmError(e);
  // MidenError with code: 'WASM_CLASS_MISMATCH' | 'WASM_POINTER_CONSUMED' |
  //   'WASM_NOT_INITIALIZED' | 'WASM_SYNC_REQUIRED' | 'SEND_BUSY' | 'UNKNOWN'
  console.log(wrapped.message); // Human-readable with fix suggestions
}
```

## Common Patterns

### Error Handling

All hooks that can fail provide an `error` state and `reset` function:

```tsx
function MyComponent() {
  const { createWallet, error, reset } = useCreateWallet();

  if (error) {
    return (
      <div>
        <p>Error: {error.message}</p>
        <button onClick={reset}>Try Again</button>
      </div>
    );
  }

  // ...
}
```

### Loading States

Query hooks provide `isLoading`, mutation hooks provide both `isLoading` and `stage`:

```tsx
function TransactionButton() {
  const { send, isLoading, stage } = useSend();

  // Show detailed progress
  const buttonText = isLoading
    ? `${stage === 'proving' ? 'Generating proof' : 'Submitting'}...`
    : 'Send';

  return <button disabled={isLoading}>{buttonText}</button>;
}
```

### Refreshing Data

All query hooks provide a `refetch` function:

```tsx
function AccountBalance({ accountId }) {
  const { assets, refetch } = useAccount(accountId);

  // Refresh after a transaction
  const handleSendComplete = async () => {
    await refetch();
  };

  return (
    <div>
      {/* ... */}
      <button onClick={refetch}>Refresh Balance</button>
    </div>
  );
}
```

### Waiting for Client Ready

Always check `isReady` before using hooks that require the client:

```tsx
function MyFeature() {
  const { isReady } = useMiden();
  const { createWallet } = useCreateWallet();

  if (!isReady) {
    return <div>Please wait...</div>;
  }

  return <button onClick={() => createWallet()}>Create Wallet</button>;
}
```

## External Signer Integration

For wallets using external key management, wrap your app with a signer provider **above** `MidenProvider`. The signer provider populates a `SignerContext` with a `signCb` and an `accountConfig`; `MidenProvider` picks these up automatically to create the client and initialize the account.

### Para (EVM Wallets)

```tsx
import { ParaSignerProvider } from '@miden-sdk/para';

function App() {
  return (
    <ParaSignerProvider apiKey="your-api-key" environment="PRODUCTION">
      <MidenProvider config={{ rpcUrl: 'testnet' }}>
        <YourApp />
      </MidenProvider>
    </ParaSignerProvider>
  );
}

// Access Para-specific data
const { para, wallet, isConnected } = useParaSigner();
```

### Turnkey

```tsx
import { TurnkeySignerProvider } from '@miden-sdk/miden-turnkey-react';

function App() {
  return (
    // Config is optional — defaults to https://api.turnkey.com
    // and reads VITE_TURNKEY_ORG_ID from environment
    <TurnkeySignerProvider>
      <MidenProvider config={{ rpcUrl: 'testnet' }}>
        <YourApp />
      </MidenProvider>
    </TurnkeySignerProvider>
  );
}

// Or with explicit config:
<TurnkeySignerProvider config={{
  apiBaseUrl: 'https://api.turnkey.com',
  defaultOrganizationId: 'your-org-id',
}}>
  ...
</TurnkeySignerProvider>
```

Connect via passkey authentication:

```tsx
import { useSigner } from '@miden-sdk/react';
import { useTurnkeySigner } from '@miden-sdk/miden-turnkey-react';

const { isConnected, connect, disconnect } = useSigner();
await connect(); // triggers passkey flow

const { client, account, setAccount } = useTurnkeySigner();
```

### MidenFi Wallet Adapter

```tsx
import { MidenFiSignerProvider } from '@miden-sdk/wallet-adapter-react';

function App() {
  return (
    <MidenFiSignerProvider network="Testnet">
      <MidenProvider config={{ rpcUrl: 'testnet' }}>
        <YourApp />
      </MidenProvider>
    </MidenFiSignerProvider>
  );
}
```

### Unified Signer Hook

`useSigner()` works with any signer provider:

```tsx
import { useSigner } from '@miden-sdk/react';

function ConnectButton() {
  const signer = useSigner();
  if (!signer) return null; // local keystore mode

  const { isConnected, connect, disconnect, name } = signer;
  return isConnected
    ? <button onClick={disconnect}>Disconnect {name}</button>
    : <button onClick={connect}>Connect with {name}</button>;
}
```

### Custom Account Components

Signer providers can include custom `AccountComponent` instances in the account via the `customComponents` field on `SignerAccountConfig`. This is useful for attaching application-specific logic compiled from `.masp` packages (e.g. a DEX component or custom smart contract) alongside the default auth and basic wallet components.

Pre-built signer providers (Para, Turnkey, MidenFi) accept `customComponents` as a prop and forward it into `accountConfig`:

```tsx
import { ParaSignerProvider } from '@miden-sdk/para';
import type { AccountComponent } from '@miden-sdk/miden-sdk';

const dexComponent: AccountComponent = await loadCompiledComponent();

<ParaSignerProvider
  apiKey="your-api-key"
  environment="PRODUCTION"
  customComponents={[dexComponent]}
>
  <MidenProvider config={{ rpcUrl: 'testnet' }}>
    <YourApp />
  </MidenProvider>
</ParaSignerProvider>
```

When building a custom signer provider, pass `customComponents` through the `SignerContext` directly:

```tsx
import { SignerContext } from '@miden-sdk/react';
import type { AccountComponent } from '@miden-sdk/miden-sdk';

function MySignerProvider({ children, customComponents }: {
  children: React.ReactNode;
  customComponents?: AccountComponent[];
}) {
  return (
    <SignerContext.Provider value={{
      name: 'MySigner',
      storeName: `mysigner_${userId}`,
      isConnected: true,
      accountConfig: {
        publicKeyCommitment: userPublicKeyCommitment,
        accountType: 'RegularAccountUpdatableCode',
        storageMode: myStorageMode,
        customComponents,
      },
      signCb: async (pubKey, signingInputs) => signature,
      connect: async () => { /* ... */ },
      disconnect: async () => { /* ... */ },
    }}>
      {children}
    </SignerContext.Provider>
  );
}
```

Components are appended to the `AccountBuilder` after the default basic wallet component and before `build()` is called, so the account always includes wallet functionality plus any extras you provide. The field is optional — omitting it or passing an empty array preserves the default behavior.

## Default Values

The SDK uses privacy-first defaults:

| Setting | Default | Description |
|---------|---------|-------------|
| `storageMode` | `'private'` | Account data stored off-chain |
| `mutable` | `true` | Wallet code can be updated |
| `authScheme` | `0` (Falcon) | Post-quantum secure signatures |
| `noteType` | `'private'` | Note contents are private |
| `skipSync` | `false` | Auto-sync before transactions |
| `decimals` | `8` | Token decimal places |
| `autoSyncInterval` | `15000` | Sync every 15 seconds |

## TypeScript

Full TypeScript support with exported types:

```tsx
import type {
  // Configuration
  MidenConfig,

  // Hook options
  CreateWalletOptions,
  CreateFaucetOptions,
  ImportAccountOptions,
  SendOptions,
  MultiSendRecipient,
  MultiSendOptions,
  InternalTransferOptions,
  InternalTransferChainOptions,
  InternalTransferResult,
  WaitForCommitOptions,
  WaitForNotesOptions,
  MintOptions,
  ConsumeOptions,
  SwapOptions,
  ExecuteTransactionOptions,
  NotesFilter,

  // Note stream types
  StreamedNote,
  UseNoteStreamOptions,
  UseNoteStreamReturn,

  // Session account types
  UseSessionAccountOptions,
  UseSessionAccountReturn,
  SessionAccountStep,

  // Hook results
  AccountResult,
  AccountsResult,
  NotesResult,
  TransactionResult,

  // State types
  TransactionStage,
  AssetBalance,
  SyncState,

  // Utility types
  NoteAttachmentData,
  MidenErrorCode,
  MigrateStorageOptions,
} from '@miden-sdk/react';
```

## Examples

One runnable Vite example lives in `examples/`:

- `examples/wallet` - Minimal wallet: create account, view balances, claim notes, send tokens.

```bash
cd examples/wallet
yarn install
yarn dev
```

## Requirements

- React 18.0 or higher
- `@miden-sdk/miden-sdk` ^0.13.1

## Browser Support

Requires browsers with WebAssembly support:
- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## License

MIT
