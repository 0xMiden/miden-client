# @miden-sdk/react

React hooks library for the Miden Web Client. Provides a simple, ergonomic interface for building React applications on the Miden rollup.

## Features

- **Easy Setup** - Single provider component handles WASM initialization and client setup
- **Sensible Defaults** - Privacy-first defaults that work out of the box
- **Auto-Sync** - Automatic background synchronization with the network
- **TypeScript First** - Full type safety with comprehensive type exports
- **Consistent Patterns** - All hooks follow predictable patterns for loading, errors, and state

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

#### `useMiden()`

Access the Miden client instance and initialization state. This is your entry
point for low-level control (syncing, direct client access, and prover-aware
transactions) while still playing nicely with the provider lifecycle.

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
every time; it throws if the client isn't ready yet.

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
pull-to-refresh, or forcing a sync before running a transaction pipeline.

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
without extra filtering logic.

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
after syncs.

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
labels without extra lookups.

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

#### `useAssetMetadata(assetIds)`

Fetch asset symbols/decimals for a list of asset IDs. This is the lightweight
way to enrich balances and note lists with human-friendly token info.

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
        noteType: 'private',      // 'private' | 'public' | 'encrypted' (default: 'private')
        recallHeight: 1000,       // Sender can reclaim after this block
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
each note to recipients via `sendPrivateNote`. Recipient IDs accept bech32 or
hex (auto-detected).

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
        { to: '0xrec2...', amount: 250n },
      ],
      noteType: 'public',
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
between accounts you control (e.g., public → private), and mirrors the
unauthenticated note transfer tutorial flow.

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

Wait for a transaction to be committed. Handy for tutorial-style flows where
you need to block until a tx is visible on-chain before the next step.

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

#### `useWaitForNotes()`

Wait until an account has consumable notes. Great for mint → consume pipelines
and other flows where you want to proceed only when notes are ready.

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
        noteType: 'private',            // Optional: 'private' | 'public' | 'encrypted'
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
    });
  };

  return (
    <button onClick={handleRun} disabled={isLoading}>
      {isLoading ? stage : 'Run Transaction'}
    </button>
  );
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

## Default Values

The SDK uses privacy-first defaults:

| Setting | Default | Description |
|---------|---------|-------------|
| `storageMode` | `'private'` | Account data stored off-chain |
| `mutable` | `true` | Wallet code can be updated |
| `authScheme` | `0` (Falcon) | Post-quantum secure signatures |
| `noteType` | `'private'` | Note contents encrypted |
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

  // Hook results
  AccountResult,
  AccountsResult,
  NotesResult,
  TransactionResult,

  // State types
  TransactionStage,
  AssetBalance,
  SyncState,
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
- `@miden-sdk/miden-sdk` ^0.13.0-0

## Browser Support

Requires browsers with WebAssembly support:
- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## License

MIT
