# React Wrapper for Miden Web Client

## Goal
Create `@miden-sdk/react` - a React hooks library wrapping `@demox-labs/miden-sdk` to make Miden as easy to use as possible with sensible defaults.

## Package Location
`packages/react-sdk/` (new package in the miden-client monorepo)

**NPM Package:** `@miden-sdk/react`
**State Management:** Zustand

---

## Architecture

### 1. MidenProvider
Context provider that handles WASM initialization, client creation, and auto-sync.

```tsx
<MidenProvider config={{ rpcUrl?, autoSyncInterval? }}>
  <App />
</MidenProvider>
```

**Defaults:**
- `rpcUrl`: testnet (if not provided)
- `autoSyncInterval`: 15000ms (15 seconds)
- Shows loading UI during WASM init

### 2. Core Hooks

| Hook | Purpose | Returns |
|------|---------|---------|
| `useMiden()` | Access client & ready state | `{ client, isReady, error, sync }` |
| `useAccounts()` | List all accounts | `{ accounts, wallets, faucets, isLoading }` |
| `useAccount(id)` | Single account details | `{ account, assets, getBalance(faucetId) }` |
| `useNotes(options?)` | List notes | `{ notes, consumableNotes, isLoading }` |
| `useSyncState()` | Sync status | `{ syncHeight, isSyncing, sync() }` |

### 3. Mutation Hooks

| Hook | Purpose | Returns |
|------|---------|---------|
| `useCreateWallet()` | Create wallet | `{ createWallet(), isCreating }` |
| `useCreateFaucet()` | Create faucet | `{ createFaucet(opts), isCreating }` |
| `useSend()` | Send tokens | `{ send(opts), isLoading, stage }` |
| `useMint()` | Mint tokens | `{ mint(opts), isLoading, stage }` |
| `useConsume()` | Consume notes | `{ consume(opts), isLoading }` |
| `useSwap()` | Swap assets | `{ swap(opts), isLoading }` |

### 4. State Pattern

All hooks follow consistent patterns:

**Query hooks:**
```ts
{ data, isLoading, error, refetch }
```

**Mutation hooks:**
```ts
{ mutate(), isLoading, stage, error, reset }
// stage: 'idle' | 'executing' | 'proving' | 'submitting' | 'complete'
```

---

## Sensible Defaults

| Setting | Default | Rationale |
|---------|---------|-----------|
| Storage mode | `private` | Privacy-first |
| Wallet mutable | `true` | Flexibility |
| Auth scheme | `0` (Falcon) | Post-quantum secure |
| Note type | `private` | Privacy-first |
| Faucet decimals | `8` | Standard precision |
| Auto-sync | `15s` | Balance freshness vs battery |

---

## Example Usage

```tsx
// Minimal setup
import { MidenProvider, useCreateWallet, useSend } from '@miden-sdk/react';

function App() {
  return (
    <MidenProvider>
      <Wallet />
    </MidenProvider>
  );
}

function Wallet() {
  const { createWallet, isCreating } = useCreateWallet();
  const { send, isLoading, stage } = useSend();

  return (
    <div>
      <button onClick={() => createWallet()} disabled={isCreating}>
        Create Wallet
      </button>
      <button onClick={() => send({ from, to, faucetId, amount: 100n })}>
        {isLoading ? stage : 'Send'}
      </button>
    </div>
  );
}
```

---

## Package Structure

```
packages/react-sdk/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts              # Public exports
│   ├── context/
│   │   └── MidenProvider.tsx
│   ├── hooks/
│   │   ├── useAccounts.ts
│   │   ├── useAccount.ts
│   │   ├── useNotes.ts
│   │   ├── useSyncState.ts
│   │   ├── useCreateWallet.ts
│   │   ├── useCreateFaucet.ts
│   │   ├── useSend.ts
│   │   ├── useMint.ts
│   │   ├── useConsume.ts
│   │   └── useSwap.ts
│   ├── store/
│   │   └── MidenStore.ts     # Internal state (zustand)
│   └── types/
│       └── index.ts
└── README.md
```

---

## Dependencies

```json
{
  "peerDependencies": {
    "react": ">=18.0.0",
    "@demox-labs/miden-sdk": ">=0.12.0"
  },
  "dependencies": {
    "zustand": "^4.0.0"
  }
}
```

---

## Implementation Order

1. **MidenProvider** + `useMiden()` - Foundation
2. **useSyncState()** - Sync management
3. **useAccounts()** + **useAccount()** - Account reading
4. **useCreateWallet()** + **useCreateFaucet()** - Account creation
5. **useNotes()** - Note reading
6. **useSend()** - Token transfers
7. **useMint()** + **useConsume()** - Mint/consume flows
8. **useSwap()** - Asset swaps

---

## Verification

1. Build: `yarn build` in `packages/react-sdk/`
2. Test with example app:
   - Create wallet
   - Sync state
   - Display accounts
   - (If testnet faucet available) mint + send tokens
3. Unit tests for hooks with mock client
