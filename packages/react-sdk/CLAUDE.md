# Miden React SDK - Usage Guide

## Installation

```bash
npm install @miden-sdk/react @miden-sdk/miden-sdk
# or
yarn add @miden-sdk/react @miden-sdk/miden-sdk
```

## Getting Started

```tsx
import { MidenProvider } from "@miden-sdk/react";

function App() {
  return (
    <MidenProvider config={{ rpcUrl: "testnet" }}>
      <YourApp />
    </MidenProvider>
  );
}
```

## Configuration

```tsx
<MidenProvider
  config={{
    rpcUrl: "testnet",           // "devnet" | "testnet" | "localhost" | custom URL
    prover: "testnet",           // "local" | "devnet" | "testnet" | custom URL
    autoSyncInterval: 15000,     // ms, set to 0 to disable
    noteTransportUrl: "...",     // optional: for private note delivery
    passkeyEncryption: true,     // optional: encrypt keys with WebAuthn PRF (biometric)
    storeName: "my-wallet",      // recommended when using passkeyEncryption
  }}
  loadingComponent={<Loading />}  // shown during WASM init
  errorComponent={<Error />}      // shown on init failure
>
```

| Network | Use When |
|---------|----------|
| `devnet` | Development, testing with fake tokens |
| `testnet` | Pre-production testing |
| `localhost` | Local node at `http://localhost:57291` |

## Passkey Encryption

Encrypt keys at rest using WebAuthn PRF (Touch ID / Face ID / Windows Hello). Requires Chrome 116+, Safari 18+, or Edge 116+.

```tsx
import { MidenProvider, isPasskeyPrfSupported } from "@miden-sdk/react";

// Check support
const supported = await isPasskeyPrfSupported();

// Enable in provider
<MidenProvider config={{
  rpcUrl: "testnet",
  passkeyEncryption: true,    // or { credentialId: "...", rpName: "..." }
  storeName: "my-wallet",     // recommended for migration support
}}>
```

- Biometric prompt fires once per `MidenProvider` mount
- Keys encrypted with AES-256-GCM, wrapping key never exposed to JS
- Ignored when a `SignerContext` is active (external signers handle their own keys)
- Credential loss = key loss (recommend iCloud Keychain / Google Password Manager)

## Reading Data (Query Hooks)

All query hooks return `{ data, isLoading, error, refetch }`.

### List Accounts
```tsx
const { data: accounts, isLoading } = useAccounts();

// accounts.wallets - regular accounts
// accounts.faucets - token faucets
// accounts.all - everything
```

### Get Account Details
```tsx
const { data: account } = useAccount(accountId);

// account.id, account.nonce, account.bech32id()
// account.balance(faucetId) - get token balance
```

### Get Notes
```tsx
const { data: notes } = useNotes();

// notes.input - incoming notes
// notes.consumable - ready to claim
```

### Check Sync Status
```tsx
const { syncHeight, isSyncing, sync } = useSyncState();

// Manual sync
await sync();
```

### Get Token Metadata
```tsx
const { data: metadata } = useAssetMetadata(faucetId);
// metadata.symbol, metadata.decimals
```

## Writing Data (Mutation Hooks)

All mutation hooks return `{ mutate, data, isLoading, stage, error, reset }`.

**Transaction stages:** `idle` → `executing` → `proving` → `submitting` → `complete`

### Create Wallet
```tsx
const { mutate: createWallet, isLoading } = useCreateWallet();

const account = await createWallet({
  storageMode: "private",  // "private" | "public" | "network"
});
```

### Send Tokens
```tsx
const { mutate: send, stage } = useSend();

await send({
  from: senderAccountId,
  to: recipientAccountId,
  faucetId: tokenFaucetId,
  amount: 1000n,
  noteType: "private",  // "private" | "public"
});
```

### Send to Multiple Recipients
```tsx
const { mutate: multiSend } = useMultiSend();

await multiSend({
  from: senderAccountId,
  outputs: [
    { to: recipient1, faucetId, amount: 500n },
    { to: recipient2, faucetId, amount: 300n },
  ],
});
```

### Claim Notes
```tsx
const { mutate: consume } = useConsume();

await consume({
  accountId: myAccountId,
  noteIds: [noteId1, noteId2],  // optional: consume specific notes
});
```

### Mint Tokens (Faucet Owner)
```tsx
const { mutate: mint } = useMint();

await mint({
  faucetId: myFaucetId,
  to: recipientAccountId,
  amount: 10000n,
});
```

### Create Faucet
```tsx
const { mutate: createFaucet } = useCreateFaucet();

const faucet = await createFaucet({
  symbol: "TOKEN",
  decimals: 8,
  maxSupply: 1000000n,
  storageMode: "public",
});
```

## Common Patterns

### Show Transaction Progress
```tsx
function SendButton() {
  const { mutate: send, stage, isLoading, error } = useSend();

  const handleSend = async () => {
    try {
      await send({ from, to, faucetId, amount });
    } catch (err) {
      console.error("Transaction failed:", err);
    }
  };

  return (
    <div>
      <button onClick={handleSend} disabled={isLoading}>
        {isLoading ? `${stage}...` : "Send"}
      </button>
      {error && <p>Error: {error.message}</p>}
    </div>
  );
}
```

### Format Token Amounts
```tsx
import { formatAssetAmount, parseAssetAmount } from "@miden-sdk/react";

// Display: 1000000n with 8 decimals → "0.01"
const display = formatAssetAmount(balance, 8);

// User input: "0.01" with 8 decimals → 1000000n
const amount = parseAssetAmount("0.01", 8);
```

### Display Note Summary
```tsx
import { getNoteSummary, formatNoteSummary } from "@miden-sdk/react";

const summary = getNoteSummary(note);
const text = formatNoteSummary(summary);  // "1.5 TOKEN"
```

### Wait for Transaction Confirmation
```tsx
const { mutate: waitForCommit } = useWaitForCommit();

// After sending
const result = await send({ ... });
await waitForCommit({ transactionId: result.transactionId });
```

### Access Client Directly
```tsx
const client = useMidenClient();

// For advanced operations not covered by hooks
const blockHeader = await client.getBlockHeaderByNumber(100);
```

### Prevent Race Conditions
```tsx
const { runExclusive } = useMiden();

// Ensures sequential execution
await runExclusive(async (client) => {
  // Multiple operations that must not interleave
});
```

## External Signer Integration

For wallets using external key management, use the pre-built signer providers:

### Para (EVM Wallets)
```tsx
import { ParaSignerProvider } from "@miden-sdk/para";

<ParaSignerProvider apiKey="your-api-key" environment="PRODUCTION">
  <MidenProvider config={{ rpcUrl: "testnet" }}>
    <App />
  </MidenProvider>
</ParaSignerProvider>

// Access Para-specific data
const { para, wallet, isConnected } = useParaSigner();
```

### Turnkey
```tsx
import { TurnkeySignerProvider } from "@miden-sdk/miden-turnkey-react";

// Config is optional — defaults to https://api.turnkey.com
// and reads VITE_TURNKEY_ORG_ID from environment
<TurnkeySignerProvider>
  <MidenProvider config={{ rpcUrl: "testnet" }}>
    <App />
  </MidenProvider>
</TurnkeySignerProvider>

// Or with explicit config:
<TurnkeySignerProvider config={{
  apiBaseUrl: "https://api.turnkey.com",
  defaultOrganizationId: "your-org-id",
}}>
  ...
</TurnkeySignerProvider>
```

Connect via passkey authentication:
```tsx
import { useSigner } from "@miden-sdk/react";
import { useTurnkeySigner } from "@miden-sdk/miden-turnkey-react";

// useSigner() handles connect/disconnect
const { isConnected, connect, disconnect } = useSigner();
await connect();  // triggers passkey flow, auto-selects account

// useTurnkeySigner() exposes Turnkey-specific extras
const { client, account, setAccount } = useTurnkeySigner();
```

### MidenFi Wallet Adapter
```tsx
import { MidenFiSignerProvider } from "@miden-sdk/wallet-adapter-react";

<MidenFiSignerProvider network="Testnet">
  <MidenProvider config={{ rpcUrl: "testnet" }}>
    <App />
  </MidenProvider>
</MidenFiSignerProvider>
```

### Using the Unified Signer Interface
```tsx
import { useSigner } from "@miden-sdk/react";

// Works with any signer provider above
const { isConnected, connect, disconnect, name } = useSigner();

if (!isConnected) {
  return <button onClick={connect}>Connect {name}</button>;
}
```

### Building a Custom Signer Provider
```tsx
import { SignerContext } from "@miden-sdk/react";

<SignerContext.Provider value={{
  name: "MyWallet",
  storeName: `mywallet_${userAddress}`,  // unique per user for DB isolation
  isConnected: true,
  accountConfig: {
    publicKey: userPublicKeyCommitment,  // Uint8Array
    storageMode: "private",
  },
  signCb: async (pubKey, signingInputs) => {
    // Route to your signing service
    return signature;  // Uint8Array
  },
  connect: async () => { /* trigger wallet connection */ },
  disconnect: async () => { /* clear session */ },
}}>
  <MidenProvider config={{ rpcUrl: "testnet" }}>
    <App />
  </MidenProvider>
</SignerContext.Provider>
```

## Account ID Formats

Both formats work interchangeably in all hooks:

```tsx
// Hex format
useAccount("0x1234567890abcdef");

// Bech32 format
useAccount("miden1qy35...");

// Convert to bech32 for display
account.bech32id();  // "miden1qy35..."
```

## Hook Reference

| Hook | Returns | Purpose |
|------|---------|---------|
| `useAccounts()` | `{ wallets, faucets, all }` | List all accounts |
| `useAccount(id)` | `Account` | Account details + balances |
| `useNotes(filter?)` | `{ input, consumable }` | Available notes |
| `useSyncState()` | `{ syncHeight, sync() }` | Sync status |
| `useAssetMetadata(id)` | `{ symbol, decimals }` | Token info |
| `useCreateWallet()` | `Account` | Create wallet |
| `useCreateFaucet()` | `Account` | Create faucet |
| `useImportAccount()` | `Account` | Import account |
| `useSend()` | `TransactionResult` | Send tokens |
| `useMultiSend()` | `TransactionResult` | Multi-recipient send |
| `useMint()` | `TransactionResult` | Mint tokens |
| `useConsume()` | `TransactionResult` | Claim notes |
| `useSwap()` | `TransactionResult` | Atomic swap |
| `useTransaction()` | `TransactionResult` | Custom transaction |

## Type Imports

```tsx
import type {
  // Config
  MidenConfig,

  // Hook results
  QueryResult,
  MutationResult,
  AccountsResult,

  // SDK types (re-exported)
  Account,
  AccountId,
  Note,
  TransactionRecord,
  FungibleAsset,
} from "@miden-sdk/react";
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Client not ready" | Wrap component in `MidenProvider`, check `useMiden().isReady` |
| Transaction stuck | Check `stage` value, network connectivity, prover availability |
| Notes not appearing | Call `sync()` manually, check `autoSyncInterval` config |
| Bech32 address wrong | Verify `rpcUrl` matches intended network |
| WASM init fails | Check browser compatibility, ensure WASM served with correct MIME type |
