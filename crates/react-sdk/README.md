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
        // RPC endpoint (defaults to testnet)
        rpcUrl: 'https://rpc.testnet.miden.io',

        // Auto-sync interval in milliseconds (default: 15000)
        // Set to 0 to disable auto-sync
        autoSyncInterval: 15000,
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

Access the Miden client instance and initialization state.

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

#### `useSyncState()`

Monitor network sync status and trigger manual syncs.

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

List all accounts, automatically categorized into wallets and faucets.

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

Get detailed information for a single account, including assets.

```tsx
import { useAccount } from '@miden-sdk/react';

function AccountDetails({ accountId }: { accountId: string }) {
  const {
    account,     // Full account object
    assets,      // Array of { faucetId, amount } balances
    isLoading,
    error,
    refetch,
    getBalance,  // Helper to get balance for specific faucet
  } = useAccount(accountId);

  if (isLoading) return <div>Loading...</div>;
  if (!account) return <div>Account not found</div>;

  // Get balance for a specific token
  const usdcBalance = getBalance('0xfaucet123...');

  return (
    <div>
      <h2>Account: {account.id().toString()}</h2>
      <p>Nonce: {account.nonce().toString()}</p>

      <h3>Assets</h3>
      {assets.map(asset => (
        <div key={asset.faucetId}>
          {asset.faucetId}: {asset.amount.toString()}
        </div>
      ))}

      <p>USDC Balance: {usdcBalance.toString()}</p>
    </div>
  );
}
```

#### `useCreateWallet()`

Create new wallet accounts.

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

Create new faucets for minting tokens.

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

### Note Hooks

#### `useNotes(options?)`

List and filter notes (incoming transactions).

```tsx
import { useNotes } from '@miden-sdk/react';

function NotesList() {
  const {
    notes,            // All notes matching filter
    consumableNotes,  // Notes ready to be consumed
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
    </div>
  );
}
```

### Transaction Hooks

All transaction hooks follow a consistent pattern with `stage` tracking:

| Stage | Description |
|-------|-------------|
| `'idle'` | Not started |
| `'proving'` | Generating ZK proof |
| `'submitting'` | Submitting to network |
| `'complete'` | Transaction confirmed |

#### `useSend()`

Send tokens from one account to another.

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
        faucetId: '0xtoken...',   // Token faucet ID
        amount: 100n,             // Amount in smallest units

        // Optional parameters
        noteType: 'private',      // 'private' | 'public' (default: 'private')
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

#### `useMint()`

Mint new tokens from a faucet you control.

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
        noteType: 'private',            // Optional (default: 'private')
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

Consume notes to claim tokens sent to your account.

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

Create atomic swap offers.

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
        noteType: 'private',
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
  SendOptions,
  MintOptions,
  ConsumeOptions,
  SwapOptions,
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

## Example App

See the [example](./example) directory for a complete working example with:

- Wallet creation
- Faucet creation
- Token minting
- Token transfers
- Sync status display

```bash
cd example
yarn install
yarn dev
```

## Requirements

- React 18.0 or higher
- `@miden-sdk/miden-sdk` 0.13.0 or higher

## Browser Support

Requires browsers with WebAssembly support:
- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## License

MIT
