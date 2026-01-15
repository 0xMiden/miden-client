# Simplified Web-Client API Design

## Overview

Simplify four key APIs in the web-client:
1. **Client creation** - Options object pattern
2. **Note creation** - Object syntax with string inputs
3. **Account creation** - Object syntax with sensible defaults
4. **Transaction building** - Convenience methods for common operations

All simplified APIs will **coexist** with existing APIs (no breaking changes).

---

## 1. Client Creation

### Current API
```typescript
// Must pass undefined for unused params
const client = await WebClient.createClient(
  "http://localhost:57291",  // rpcUrl
  undefined,                  // noteTransportUrl
  undefined                   // seed
);

// External keystore - 6 positional params
const client = await WebClient.createClientWithExternalKeystore(
  rpcUrl, noteTransportUrl, seed,
  getKeyCb, insertKeyCb, signCb
);
```

### Simplified API
```typescript
// Minimal - uses testnet defaults
const client = await WebClient.create();

// With RPC URL
const client = await WebClient.create({
  rpcUrl: "http://localhost:57291"
});

// With seed (string auto-hashed to 32 bytes)
const client = await WebClient.create({
  rpcUrl: "http://localhost:57291",
  seed: "my-deterministic-seed"
});

// With external keystore (grouped in single object)
const client = await WebClient.create({
  rpcUrl: "http://localhost:57291",
  keystore: {
    getKey: async (pubKey) => { /* ... */ },
    insertKey: async (pubKey, secretKey) => { /* ... */ },
    sign: async (pubKey, inputs) => { /* ... */ }
  }
});

// With note transport
const client = await WebClient.create({
  rpcUrl: "http://localhost:57291",
  noteTransportUrl: "http://transport:8080"
});
```

---

## 2. Note Creation

### Current API
```typescript
// 8+ lines for a simple P2ID note
let senderAccountId = AccountId.fromHex(_senderId);
let targetAccountId = AccountId.fromHex(_targetId);
let faucetAccountId = AccountId.fromHex(_faucetId);
let fungibleAsset = new FungibleAsset(faucetAccountId, BigInt(10));
let noteAssets = new NoteAssets([fungibleAsset]);
let p2IdNote = Note.createP2IDNote(
  senderAccountId, targetAccountId, noteAssets,
  NoteType.Public, new Felt(0n)
);
let outputNote = OutputNote.full(p2IdNote);
```

### Simplified API
```typescript
// Basic P2ID - accepts hex strings, plain numbers
const note = Note.p2id({
  from: "0xabc123...",
  to: "0xdef456...",
  asset: { faucet: "0x789...", amount: 10 }
});
// Defaults: type="public", aux=0

// With options
const note = Note.p2id({
  from: "0xabc123...",
  to: "0xdef456...",
  asset: { faucet: "0x789...", amount: 10 },
  type: "private",  // "public" | "private" | "encrypted"
  aux: 42
});

// P2IDE with timelock/reclaim
const note = Note.p2ide({
  from: "0xabc123...",
  to: "0xdef456...",
  asset: { faucet: "0x789...", amount: 10 },
  reclaimAfter: 1000,   // block height
  timelockUntil: 500    // block height
});

// Multiple assets
const note = Note.p2id({
  from: "0xabc...",
  to: "0xdef...",
  assets: [
    { faucet: "0x111...", amount: 10 },
    { faucet: "0x222...", amount: 20 }
  ]
});

// Returns OutputNote directly (no need to wrap)
```

---

## 3. Account Creation

### Current API
```typescript
// Wallet - must understand storage modes, auth schemes
const wallet = await client.newWallet(
  AccountStorageMode.private(),  // Must construct enum
  true,                          // mutable - boolean
  0,                             // authScheme - magic number
  walletSeed                     // optional Uint8Array
);

// Faucet - many required params
const faucet = await client.newFaucet(
  AccountStorageMode.public(),
  false,           // nonFungible - must be false
  "DAG",           // tokenSymbol
  8,               // decimals
  BigInt(10000000), // maxSupply - requires BigInt
  0                // authScheme
);
```

### Simplified API
```typescript
// Wallet - minimal
const wallet = await client.createWallet();
// Defaults: storage="private", mutable=true, auth="falcon"

// Wallet - with options
const wallet = await client.createWallet({
  storage: "public",      // "private" | "public"
  mutable: false,
  auth: "ecdsa",          // "falcon" | "ecdsa"
  seed: "deterministic"   // optional string seed
});

// Faucet - minimal (only required fields)
const faucet = await client.createFaucet({
  symbol: "DAG",
  decimals: 8,
  maxSupply: 10_000_000   // plain number, no BigInt needed
});

// Faucet - with options
const faucet = await client.createFaucet({
  symbol: "DAG",
  decimals: 8,
  maxSupply: 10_000_000,
  storage: "public",      // "private" | "public" (default: "public")
  auth: "falcon"          // "falcon" | "ecdsa" (default: "falcon")
});
```

---

## 4. Transaction Building

### Current API
```typescript
// Mint - requires constructing multiple objects
const tx = client.newMintTransactionRequest(
  targetAccountId,   // Must be AccountId object
  faucetId,          // Must be AccountId object
  NoteType.Public,   // Enum
  BigInt(100)        // Amount as BigInt
);

// Send - many positional params
const tx = client.newSendTransactionRequest(
  senderAccountId,
  targetAccountId,
  faucetId,
  NoteType.Public,
  BigInt(100),
  null,   // recallHeight - must pass null
  null    // timelockHeight - must pass null
);

// Consume - note IDs as strings but in array
const tx = client.newConsumeTransactionRequest([noteId1, noteId2]);

// Custom - complex builder
const tx = new TransactionRequestBuilder()
  .withOwnOutputNotes(new OutputNoteArray([outputNote]))
  .withUnauthenticatedInputNotes(new NoteAndArgsArray([...]))
  .withCustomScript(script)
  .build();
```

### Simplified API
```typescript
// Mint - accepts strings and numbers
const tx = client.mint({
  target: "0xdef456...",
  faucet: "0x789...",
  amount: 100,
  type: "public"   // optional, defaults to "public"
});

// Send - object with sensible defaults
const tx = client.send({
  from: "0xabc123...",
  to: "0xdef456...",
  faucet: "0x789...",
  amount: 100
});
// Defaults: type="public", no timelock/reclaim

// Send with timelock/reclaim
const tx = client.send({
  from: "0xabc123...",
  to: "0xdef456...",
  faucet: "0x789...",
  amount: 100,
  type: "private",
  reclaimAfter: 1000,
  timelockUntil: 500
});

// Consume - simple array of note IDs
const tx = client.consume(["0xnote1...", "0xnote2..."]);
// or single note
const tx = client.consume("0xnote1...");

// Swap
const tx = client.swap({
  from: "0xabc123...",
  offer: { faucet: "0x111...", amount: 10 },
  request: { faucet: "0x222...", amount: 5 },
  type: "public"
});

// Execute (submit) - simplified
const txId = await client.submitTransaction("0xaccountId...", tx);
```

---

## Implementation Plan

### Files to Modify

| File | Changes |
|------|---------|
| `crates/web-client/js/index.js` | Add wrapper functions for all simplified APIs |
| `crates/web-client/dist/index.d.ts` | TypeScript type definitions for new APIs |

### Implementation Strategy

All simplified APIs are **JavaScript wrappers** that call existing Rust/WASM bindings:
- No changes to Rust code required
- Wrappers handle all conversions (hex→AccountId, number→BigInt, string→enum)
- Existing APIs remain unchanged for power users

### Type Definitions

```typescript
// Types for simplified APIs
interface ClientOptions {
  rpcUrl?: string;
  noteTransportUrl?: string;
  seed?: string;
  keystore?: {
    getKey: (pubKey: Uint8Array) => Promise<Uint8Array | null>;
    insertKey: (pubKey: Uint8Array, secretKey: Uint8Array) => Promise<void>;
    sign: (pubKey: Uint8Array, inputs: Uint8Array) => Promise<Uint8Array>;
  };
}

interface Asset {
  faucet: string;
  amount: number;
}

interface NoteOptions {
  from: string;
  to: string;
  asset?: Asset;
  assets?: Asset[];
  type?: "public" | "private" | "encrypted";
  aux?: number;
}

interface P2IDEOptions extends NoteOptions {
  reclaimAfter?: number;
  timelockUntil?: number;
}

interface WalletOptions {
  storage?: "private" | "public";
  mutable?: boolean;
  auth?: "falcon" | "ecdsa";
  seed?: string;
}

interface FaucetOptions {
  symbol: string;
  decimals: number;
  maxSupply: number;
  storage?: "private" | "public";
  auth?: "falcon" | "ecdsa";
}

interface SendOptions {
  from: string;
  to: string;
  faucet: string;
  amount: number;
  type?: "public" | "private" | "encrypted";
  reclaimAfter?: number;
  timelockUntil?: number;
}

interface MintOptions {
  target: string;
  faucet: string;
  amount: number;
  type?: "public" | "private" | "encrypted";
}

interface SwapOptions {
  from: string;
  offer: Asset;
  request: Asset;
  type?: "public" | "private" | "encrypted";
  paybackType?: "public" | "private" | "encrypted";
}
```

---

## Verification

1. **Unit tests**: Add tests in `crates/web-client/test/` for each simplified API
2. **Build**: Run `yarn build` in `crates/web-client`
3. **Existing tests**: Ensure all existing tests pass (no breaking changes)
4. **Integration**: Run `make integration-test-web-client`

---

## 5. Address/AccountId Handling

### Current API
```typescript
// Two-step parsing everywhere
const bobAccountId = Address.fromBech32('mtst1apve54rq8...').accountId();

// Must convert to string for some APIs
note.inputNoteRecord().id().toString()
```

### Simplified API
All APIs accept bech32 address strings directly:
```typescript
// Just use strings everywhere
await client.send({
  from: "mtst1abc123...",     // bech32 string
  to: "mtst1def456...",       // bech32 string
  faucet: "mtst1faucet...",   // bech32 string
  amount: 100
});

// Also accept hex format
await client.send({
  from: "0xabc123...",
  to: "0xdef456...",
  ...
});
```

---

## 6. Remote Prover Configuration

### Current API
```typescript
// Separate prover setup
const prover = TransactionProver.newRemoteProver(
  'https://tx-prover.testnet.miden.io'
);

// Must pass to proveTransaction
const proven = await client.proveTransaction(txResult, prover);
```

### Simplified API
```typescript
// Built into client creation
const client = await WebClient.create({
  rpcUrl: "https://rpc.testnet.miden.io",
  proverUrl: "https://tx-prover.testnet.miden.io"  // auto-used
});

// All transactions automatically use remote prover
await client.submitTransaction(accountId, tx);

// Or use testnet preset
const client = await WebClient.createTestnet();  // all URLs preconfigured
```

---

## 7. Consume All Notes

### Current API
```typescript
// 4 lines, manual mapping
const consumableNotes = await client.getConsumableNotes(alice.id());
const noteIds = consumableNotes.map((note) =>
  note.inputNoteRecord().id().toString()
);
const consumeTxRequest = client.newConsumeTransactionRequest(noteIds);
await client.submitNewTransaction(alice.id(), consumeTxRequest);
```

### Simplified API
```typescript
// One-liner
await client.consumeAllNotes("mtst1alice...");

// With options
await client.consumeAllNotes("mtst1alice...", {
  maxNotes: 10,           // limit batch size
  waitForConfirmation: true
});
```

---

## 8. Wait for Transaction Confirmation

### Current API
```typescript
// Manual delay + sync
await client.submitNewTransaction(accountId, txRequest);
await new Promise((resolve) => setTimeout(resolve, 10000));
await client.syncState();
```

### Simplified API
```typescript
// Option 1: Wait flag on submit
const txId = await client.submitTransaction(accountId, tx, {
  waitForConfirmation: true,
  timeout: 30000  // optional timeout in ms
});

// Option 2: Explicit wait method
const txId = await client.submitTransaction(accountId, tx);
await client.waitForTransaction(txId);

// Option 3: Wait with callback for progress
await client.waitForTransaction(txId, {
  onProgress: (status) => console.log(status),
  timeout: 60000
});
```

---

## 9. High-Level Workflow Methods

### Mint and Consume (common pattern)
```typescript
// Current - ~10 lines
const mintTx = client.newMintTransactionRequest(...);
await client.submitNewTransaction(faucet.id(), mintTx);
await new Promise(r => setTimeout(r, 10000));
await client.syncState();
const notes = await client.getConsumableNotes(alice.id());
const noteIds = notes.map(n => n.inputNoteRecord().id().toString());
const consumeTx = client.newConsumeTransactionRequest(noteIds);
await client.submitNewTransaction(alice.id(), consumeTx);

// Simplified - 1 line
await client.mintAndConsume({
  faucet: "mtst1faucet...",
  target: "mtst1alice...",
  amount: 1000
});
```

### Import Account (with auto-sync)
```typescript
// Current - 7 lines
const id = Address.fromBech32('mtst1...').accountId();
let account = await client.getAccount(id);
if (!account) {
  await client.importAccountById(id);
  await client.syncState();
  account = await client.getAccount(id);
}

// Simplified - 1 line
const account = await client.importAccount("mtst1...");
```

### Send Tokens (full transfer flow)
```typescript
// Full transfer flow
await client.transfer({
  from: "mtst1alice...",
  to: "mtst1bob...",
  faucet: "mtst1faucet...",
  amount: 100,
  waitForConfirmation: true
});
```

---

## 10. Testnet Preset

### Simplified API
```typescript
// One-liner for testnet with all defaults
const client = await WebClient.createTestnet();

// Equivalent to:
const client = await WebClient.create({
  rpcUrl: "https://rpc.testnet.miden.io",
  proverUrl: "https://tx-prover.testnet.miden.io"
});
```

---

## Summary

| API | Current | Simplified |
|-----|---------|------------|
| Client | 3 methods, positional params | `WebClient.create(options?)` |
| Testnet | Manual URL config | `WebClient.createTestnet()` |
| Note | 8+ lines, manual wrapping | `Note.p2id({ from, to, asset })` |
| Wallet | Enums, magic numbers | `client.createWallet(options?)` |
| Faucet | Many required params | `client.createFaucet({ symbol, decimals, maxSupply })` |
| Mint | AccountId objects, BigInt | `client.mint({ target, faucet, amount })` |
| Send | 7 positional params | `client.send({ from, to, faucet, amount })` |
| Consume | Array wrapper | `client.consume(noteIds)` |
| Consume All | 4 lines, manual mapping | `client.consumeAllNotes(accountId)` |
| Swap | Complex builder | `client.swap({ from, offer, request })` |
| Addresses | Two-step parsing | Direct bech32/hex strings |
| Prover | Separate setup | Built into client options |
| Wait | Manual setTimeout | `{ waitForConfirmation: true }` |
| Mint+Consume | ~10 lines | `client.mintAndConsume({...})` |
| Import | 7 lines with null check | `client.importAccount(address)` |
| Transfer | Multiple steps | `client.transfer({...})` |
