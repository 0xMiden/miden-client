---
title: Simplified Web-Client API Design
---

# Simplified Web-Client API Design

## Overview

Redesign the web-client SDK around a **resource-based architecture** inspired by Stripe, replacing
the current flat 30+ method God Object with namespaced resources. All simplified APIs **replace**
existing APIs — the old method names (`createClient`, `newWallet`, `newFaucet`,
`newMintTransactionRequest`, `newSendTransactionRequest`, `newConsumeTransactionRequest`,
`newSwapTransactionRequest`, `submitNewTransaction`, etc.) will be removed from the JS wrapper.
Callers must migrate to the new signatures.

### Design Principles

1. **Resource-based namespacing** — `client.accounts.*`, `client.transactions.*`, `client.notes.*`,
   `client.tags.*`, `client.settings.*`. Type `client.` and see 5 resources, not 30 methods.
2. **One call for the common case** — `client.transactions.send()` builds, executes, proves, and
   submits. No two-step ceremony for 90% of use cases.
3. **`account` field everywhere** — Every transaction method takes `account` as the executing
   account. No redundant positional `accountId` parameter. No ambiguity between `from`/`faucet`
   as implicit executor.
4. **Consistent naming** — `to` for recipient (never `target`), `list` for collections, `get` for
   single items. Learn one resource, predict the rest.
5. **TypeScript-first** — Discriminated unions for multi-variant options. `bigint` for all amounts.
   IntelliSense guides you through the API without reading docs.
6. **Resource methods async** — Every resource method returns a `Promise`. Standalone utilities
   (`buildSendRequest`, `createP2IDNote`, `buildSwapTag`) and lifecycle methods (`terminate`)
   are synchronous when the underlying operation involves no I/O.
7. **Standalone utilities for pure computation** — `createP2IDNote()`, `createP2IDENote()`,
   `AuthScheme` are tree-shakeable imports that don't need client state.

---

## Quick Start

```typescript
import { MidenClient, AuthScheme } from '@miden-sdk/miden-sdk';

// 1. Create client (defaults to testnet)
const client = await MidenClient.create();

// 2. Create a wallet and a token (faucet account)
const wallet = await client.accounts.create();
const dagToken = await client.accounts.create({
  type: "faucet", symbol: "DAG", decimals: 8, maxSupply: 10_000_000n
});

// 3. Mint tokens to the wallet (mint + sync + consume in one call)
// All ID fields accept Account objects, AccountId objects, or hex/bech32 strings.
await client.transactions.mintAndConsume({ faucet: dagToken, to: wallet, amount: 1000n });

// 4. Send tokens to another address
await client.transactions.send({
  account: wallet,
  to: "0xBOB",          // strings work too
  token: dagToken,
  amount: 100n
});

// 5. Check balance
const balance = await client.accounts.getBalance(wallet, dagToken);
console.log(`Balance: ${balance}`); // 900n
```

---

## 1. Client Creation

### Current API
```typescript
// Must pass undefined for unused params — 4 positional params
const client = new WebClient();
await client.createClient(
  "http://localhost:57291",  // rpcUrl
  undefined,                  // noteTransportUrl
  undefined,                  // seed (Uint8Array)
  undefined                   // storeName
);

// External keystore — 7 positional params
await client.createClientWithExternalKeystore(
  rpcUrl, noteTransportUrl, seed,
  storeName,
  getKeyCb, insertKeyCb, signCb
);
```

### Simplified API
```typescript
// Minimal — defaults to testnet RPC (existing behavior), adds options pattern
const client = await MidenClient.create();

// With RPC URL
const client = await MidenClient.create({
  rpcUrl: "http://localhost:57291"
});

// Testnet with prover preconfigured and autoSync (syncs state before returning)
const client = await MidenClient.createTestnet();

// Testnet without autoSync (useful if testnet node may be unreachable)
const client = await MidenClient.createTestnet({ autoSync: false });

// With seed (string auto-hashed to 32 bytes via SHA-256)
const client = await MidenClient.create({
  rpcUrl: "http://localhost:57291",
  seed: "my-deterministic-seed"
});

// With external keystore (grouped in single object)
const client = await MidenClient.create({
  rpcUrl: "http://localhost:57291",
  keystore: {
    getKey: async (pubKey) => { /* ... */ },
    insertKey: async (pubKey, secretKey) => { /* ... */ },
    sign: async (pubKey, inputs) => { /* ... */ }
  }
});

// Auto-sync on creation (fetches latest chain state before returning)
const client = await MidenClient.create({
  rpcUrl: "http://localhost:57291",
  autoSync: true
});

// With note transport and debug logging
const client = await MidenClient.create({
  rpcUrl: "http://localhost:57291",
  noteTransportUrl: "http://transport:8080",
  debug: true  // structured logging via console.debug
});

// Mock client for testing (uses same resource interface)
const client = await MidenClient.createMock();
```

> **Note:** `createClient()` already defaults to testnet when no URL is given.
> `createTestnet()` adds value by preconfiguring the prover URL **and** defaulting
> `autoSync: true` so the client is ready to use immediately. Pass `{ autoSync: false }`
> to skip the initial sync (useful if the testnet node may be unreachable at boot time).

---

## 2. Accounts Resource

### Current API
```typescript
// Create wallet — must construct enum objects
const wallet = await client.newWallet(
  AccountStorageMode.private(),
  true,
  AuthScheme.AuthRpoFalcon512,
  walletSeed
);

// Create faucet — many required params, nonFungible must be false
const faucet = await client.newFaucet(
  AccountStorageMode.public(), false, "DAG", 8,
  BigInt(10000000), AuthScheme.AuthRpoFalcon512
);

// Query — 4-5 separate calls, all require AccountId objects
const account = await client.getAccount(accountId);
const vault = await client.getAccountVault(accountId);
const storage = await client.getAccountStorage(accountId);
const code = await client.getAccountCode(accountId);
const balance = await reader.getBalance(faucetId);

// Import — 4 inconsistent methods
await client.importAccountById(accountId);
await client.importAccountFile(accountFile);
await client.importPublicAccountFromSeed(seed, mutable, AuthScheme.AuthRpoFalcon512);
```

### Simplified API
```typescript
// ── Create ──
// Wallet (default: private storage, mutable, Falcon auth)
const wallet = await client.accounts.create();

// Wallet with options
const wallet = await client.accounts.create({
  storage: "public",
  mutable: false,
  auth: AuthScheme.ECDSA,
  seed: "deterministic"
});

// Faucet — only required fields
const faucet = await client.accounts.create({
  type: "faucet",
  symbol: "DAG",
  decimals: 8,
  maxSupply: 10_000_000n
});

// Faucet with options
const faucet = await client.accounts.create({
  type: "faucet",
  symbol: "DAG",
  decimals: 8,
  maxSupply: 10_000_000n,
  storage: "public",
  auth: AuthScheme.Falcon
});

// ── Retrieve ──
const account = await client.accounts.get("0xabc...");
const accounts = await client.accounts.list();
const details = await client.accounts.getDetails("0xabc...");
// Returns { account, vault, storage, code, keys }

// Quick balance check (wraps existing accountReader)
const balance = await client.accounts.getBalance("0xabc...", "0xDAG...");

// ── Import / Export ──
const account = await client.accounts.import("mtst1abc...");     // by bech32
const account = await client.accounts.import({ file: accountFile });
const account = await client.accounts.import({
  seed: walletSeed, mutable: true, auth: AuthScheme.Falcon
});
const file = await client.accounts.export("0xabc...");

// ── Address management ──
await client.accounts.addAddress("0xabc...", "mtst1address...");
await client.accounts.removeAddress("0xabc...", "mtst1address...");
```

> **Note:** `accountReader()` already provides lazy per-field access (nonce, balance,
> storage slots). `getBalance()` is a thin convenience wrapper.
> `getDetails()` is a batch call for when you need multiple fields at once.
>
> **Null vs throw convention:** `get()` returns `null` when the entity is not found.
> All other methods (`getDetails()`, `getBalance()`, `export()`) throw if the account
> doesn't exist. This convention applies uniformly across all resources:
> - `accounts.get()`, `notes.get()` → `T | null`
> - Everything else → throws `"Account not found: 0x..."` / `"Note not found: 0x..."`

---

## 3. Transactions Resource

### Current API
```typescript
// Build: separate methods with positional params, enum objects, BigInt
const tx = client.newMintTransactionRequest(targetAccountId, faucetId, NoteType.Public, BigInt(100));
const tx = client.newSendTransactionRequest(
  senderAccountId, targetAccountId, faucetId, NoteType.Public, BigInt(100), undefined, undefined
);
const tx = client.newConsumeTransactionRequest(notes); // Vec<Note>

// Submit: requires AccountId object, redundant with info in the request
await client.submitNewTransaction(accountId, txRequest);

// Wait: manual setTimeout + syncState
await new Promise(r => setTimeout(r, 10000));
await client.syncState();

// Preview: executeForSummary with AccountId object
const summary = await client.executeForSummary(accountId, txRequest);
```

### Simplified API

Every method takes an `account` field — the account executing the transaction. All amounts
accept `number | bigint` as input; all return `bigint`. Every method is async and returns
a `TransactionId` (the full build → execute → prove → submit pipeline runs internally).

```typescript
// ── Send tokens ──
const txId = await client.transactions.send({
  account: "0xALICE",
  to: "0xBOB",
  token: "0xDAG",
  amount: 100n
});
// Defaults: type="public", no timelock/reclaim

// With all options
const txId = await client.transactions.send({
  account: "0xALICE",
  to: "0xBOB",
  token: "0xDAG",
  amount: 100n,
  type: "private",
  reclaimAfter: 1000,
  timelockUntil: 500,
  waitForConfirmation: true,
  timeout: 30_000
});

// ── Mint tokens ──
const txId = await client.transactions.mint({
  account: "0xDAG_FAUCET",   // the faucet IS the executing account
  to: "0xBOB",
  amount: 100n
});

// ── Consume notes ──
const txId = await client.transactions.consume({
  account: "0xALICE",
  notes: ["0xnote1", "0xnote2"]           // string IDs
});
const txId = await client.transactions.consume({
  account: "0xALICE",
  notes: [note1, note2]                    // Note objects
});
const txId = await client.transactions.consume({
  account: "0xALICE",
  notes: ["0xnote1", note2]               // mixed
});

// ── Consume all ──
const result = await client.transactions.consumeAll({ account: "0xALICE" });
// result: { txId: TransactionId, consumed: number, remaining: number }

const result = await client.transactions.consumeAll({
  account: "0xALICE",
  maxNotes: 10,     // consume at most 10; remaining tells you how many are left
  waitForConfirmation: true
});
if (result.remaining > 0) {
  // more notes available — call again or handle as needed
}

// ── Swap ──
const txId = await client.transactions.swap({
  account: "0xALICE",
  offer: { token: "0xDAG", amount: 10n },
  request: { token: "0xETH", amount: 5n }
});

// ── Mint and consume (composed) ──
await client.transactions.mintAndConsume({
  faucet: "0xDAG_FAUCET",    // executes the mint
  to: "0xALICE",             // receives AND consumes
  amount: 1000n
});

// ── Preview (dry run — wraps existing executeForSummary) ──
// Uses explicit PreviewSendOptions / PreviewMintOptions etc. (not Omit<> types)
// so IntelliSense shows clean, readable property shapes.
const summary = await client.transactions.preview({
  operation: "send",
  account: "0xALICE",
  to: "0xBOB",
  token: "0xDAG",
  amount: 100n
});

// ── Query ──
const txs = await client.transactions.list();
const txs = await client.transactions.list({ status: "uncommitted" });
const txs = await client.transactions.list({ ids: ["0xtx1", "0xtx2"] });
const txs = await client.transactions.list({ expiredBefore: 1000 });

// ── Wait ──
await client.transactions.waitFor(txId);
await client.transactions.waitFor(txId, {
  timeout: 60_000,
  onProgress: (status) => console.log(status)
  // status: "pending" | "submitted" | "committed"
  // Throws immediately if transaction is rejected (no waiting until timeout)
});

// ── Advanced: submit pre-built request ──
// For custom TransactionRequestBuilder usage
const txId = await client.transactions.submit(request);
// account is embedded in the request from building

// ── Advanced: inspect before submit ──
// Standalone request builders for two-step "build → inspect → submit" flow
import { buildSendRequest, buildMintRequest } from '@miden-sdk/miden-sdk';

const request = buildSendRequest({
  account: "0xALICE", to: "0xBOB", token: "0xDAG", amount: 100n
});
console.log(request);  // inspect, log, serialize
const txId = await client.transactions.submit(request);
```

> **Note:** `submitNewTransaction` already exists in WASM. The simplified wrappers add
> string→AccountId conversion, default prover handling, and optional wait.
> `preview()` wraps the existing `executeForSummary()` WASM method.
>
> **Return type consistency:** `send()`, `mint()`, `consume()`, `swap()`, `submit()`, and
> `mintAndConsume()` all return `Promise<TransactionId>`. `consumeAll()` is the exception —
> it returns `ConsumeAllResult` (which includes `txId` plus `consumed`/`remaining` counts)
> because callers need the pagination info to decide whether to call again.

---

## 4. Notes Resource

### Current API
```typescript
// Query — must construct NoteFilter enum
const filter = new NoteFilter(NoteFilterTypes.Committed, undefined);
const notes = await client.getInputNotes(filter);

// Requires NoteId objects for list filter
const filter = new NoteFilter(NoteFilterTypes.List, [noteId1, noteId2]);

// Private transport — two methods with subtle difference
await client.fetchPrivateNotes();      // incremental
await client.fetchAllPrivateNotes();   // full

// Send requires Note + Address objects
await client.sendPrivateNote(note, address);
```

### Simplified API

> **Terminology:** In Miden's UTXO model, *received notes* are notes you can spend
> (received from others). *Sent notes* are notes you've created (sent to others).
> `list()` defaults to received/spendable notes since that's the most common query.

```typescript
// ── Query received notes (notes you can spend) ──
const notes = await client.notes.list();
const notes = await client.notes.list({ status: "committed" });
const notes = await client.notes.list({ status: "consumed" });
const notes = await client.notes.list({ ids: ["0xnote1", "0xnote2"] });

// ── Single note (returns null if not found) ──
const note = await client.notes.get("0xnote1...");

// ── Sent notes (notes you've created/sent to others) ──
const notes = await client.notes.listSent();
const notes = await client.notes.listSent({ status: "committed" });

// ── Available notes for an account (ready to consume right now) ──
const notes = await client.notes.listAvailable({ account: "0xaccount..." });

// ── Import / Export ──
const noteId = await client.notes.import(noteFile);
const file = await client.notes.export("0xnote...");
const file = await client.notes.export("0xnote...", { format: "full" });

// ── Private note transport ──
await client.notes.fetch();                    // incremental (default)
await client.notes.fetch({ mode: "all" });     // fetch all from transport
await client.notes.sendPrivate({ noteId: "0xnote...", to: "mtst1recipient..." });
```

---

## 5. Tags Resource

### Current API
```typescript
await client.addTag("12345");
await client.removeTag("12345");
const tags = await client.listTags();  // returns JsValue
```

### Simplified API
```typescript
await client.tags.add(12345);
await client.tags.remove(12345);
const tags: number[] = await client.tags.list();
// Input and output are both `number` — no type mismatch.
// Use NoteTag helpers from WASM to compute tag values from faucet IDs etc.
```

---

## 6. Settings Resource

### Current API
```typescript
const value = await client.getSetting("myKey");
await client.setSetting("myKey", someValue);
await client.removeSetting("myKey");
const keys: string[] = await client.listSettingKeys();
```

### Simplified API
```typescript
// Key-value store persisted in IndexedDB — useful for app-level preferences
const value = await client.settings.get("myKey");             // unknown | null
const theme = await client.settings.get<{ dark: boolean }>("theme");  // typed
await client.settings.set("myKey", { theme: "dark" });
await client.settings.remove("myKey");
const keys: string[] = await client.settings.listKeys();
```

> **Note:** Settings are arbitrary key-value pairs stored in the client's backing store.
> Values are serialized/deserialized automatically (stored as `Uint8Array` internally).
> This is a thin wrapper over existing `getSetting`/`setSetting`/`removeSetting`/`listSettingKeys`.

---

## 7. Note Creation (Standalone Utilities)

### Current API
```typescript
// 8+ lines for a simple P2ID note
let senderAccountId = AccountId.fromHex(_senderId);
let targetAccountId = AccountId.fromHex(_targetId);
let faucetAccountId = AccountId.fromHex(_faucetId);
let fungibleAsset = new FungibleAsset(faucetAccountId, BigInt(10));
let noteAssets = new NoteAssets([fungibleAsset]);
let attachment = new NoteAttachment([]);
let p2IdNote = Note.createP2IDNote(
  senderAccountId, targetAccountId, noteAssets, NoteType.Public, attachment
);
let outputNote = OutputNote.full(p2IdNote);
```

### Simplified API

Note creation is **standalone functions** (tree-shakeable, no client state needed).
Use these when building custom transactions via `TransactionRequestBuilder`.
For standard send/mint/consume/swap, use `client.transactions.*` instead.

```typescript
import { createP2IDNote, createP2IDENote } from '@miden-sdk/miden-sdk';

// P2ID — accepts hex/bech32 strings, plain numbers
const note = createP2IDNote({
  from: "0xabc123...",
  to: "0xdef456...",
  assets: { token: "0x789...", amount: 10n }
});
// Defaults: type="public", attachment=empty

// With options
const note = createP2IDNote({
  from: "0xabc123...",
  to: "0xdef456...",
  assets: { token: "0x789...", amount: 10n },
  type: "private",
  attachment: [/* Felt values */]
});

// P2IDE with timelock/reclaim
const note = createP2IDENote({
  from: "0xabc123...",
  to: "0xdef456...",
  assets: { token: "0x789...", amount: 10n },
  reclaimAfter: 1000,
  timelockUntil: 500
});

// Multiple assets
const note = createP2IDNote({
  from: "0xabc...",
  to: "0xdef...",
  assets: [
    { token: "0x111...", amount: 10n },
    { token: "0x222...", amount: 20n }
  ]
});

// Flow to custom transaction (simplified — no WASM type ceremony):
const note = createP2IDNote({ from: "0x...", to: "0x...", assets: { token: "0x...", amount: 100n } });
const request = new TransactionRequestBuilder()
  .withOutputNotes([note])       // accepts OutputNote[] — handles wrapping internally
  .build();
const txId = await client.transactions.submit(request);

// Or use the standalone request builder for common operations:
import { buildSendRequest } from '@miden-sdk/miden-sdk';
const request = buildSendRequest({ account: "0x...", to: "0x...", token: "0x...", amount: 100n });
const txId = await client.transactions.submit(request);
```

---

## 8. Address / ID Handling

All APIs accept hex or bech32 strings directly. No `AccountId.fromHex()` or
`Address.fromBech32().accountId()` ceremony.

```typescript
// Hex format
await client.transactions.send({
  account: "0xabc123...",
  to: "0xdef456...",
  token: "0x789...",
  amount: 100n
});

// Bech32 format
await client.transactions.send({
  account: "mtst1abc123...",
  to: "mtst1def456...",
  token: "mtst1dag...",
  amount: 100n
});
```

---

## 9. Lifecycle & Store

```typescript
// Sync state with the network
const summary = await client.sync();
const summary = await client.sync({ timeout: 30_000 });

// Get current sync height
const height = await client.getSyncHeight();

// Export/import full store (backup/restore)
const snapshot: StoreSnapshot = await client.exportStore();
console.log(snapshot.version);  // format version for compatibility checks
await client.importStore(snapshot);

// Terminate worker — after this, all method calls throw "Client terminated"
client.terminate();
```

> **Post-termination behavior:** After `terminate()`, any subsequent method call on any
> resource throws `Error("Client terminated")`. This is enforced by a `#terminated` flag
> checked at the start of every resource method. The client cannot be re-initialized —
> create a new `MidenClient` instance instead.

---

## Type Definitions

```typescript
// ════════════════════════════════════════════════════════════════
// Constants
// ════════════════════════════════════════════════════════════════

/** Authentication scheme constants. Use with IntelliSense for discoverability. */
const AuthScheme = {
  Falcon: "falcon",
  ECDSA: "ecdsa",
} as const;
type AuthSchemeType = typeof AuthScheme[keyof typeof AuthScheme];

// ════════════════════════════════════════════════════════════════
// Client options
// ════════════════════════════════════════════════════════════════

interface ClientOptions {
  rpcUrl?: string;               // default: testnet RPC
  noteTransportUrl?: string;
  proverUrl?: string;            // auto-creates remote prover
  seed?: string;                 // hashed to 32 bytes via SHA-256
  storeName?: string;            // store isolation key
  autoSync?: boolean;            // sync state on creation (default: false)
  debug?: boolean;               // structured logging via console.debug
  keystore?: {
    getKey: (pubKey: Uint8Array) => Promise<Uint8Array | null>;
    insertKey: (pubKey: Uint8Array, secretKey: Uint8Array) => Promise<void>;
    sign: (pubKey: Uint8Array, inputs: Uint8Array) => Promise<Uint8Array>;
  };
}

// ════════════════════════════════════════════════════════════════
// Shared types
// ════════════════════════════════════════════════════════════════

/** An account reference: hex string, bech32 string, Account object, or AccountId
 *  object. All ID fields throughout the SDK accept any of these forms. The SDK
 *  resolves to AccountId internally via toString(). This eliminates .id().toString()
 *  ceremony — you can pass Account objects returned by create() directly. */
type AccountRef = string | Account | AccountId;

/** Represents an amount of a specific token. Every token on Miden is issued by a
 *  faucet account, so the faucet's account ID serves as the token identifier.
 *  The field is named `token` (not `faucet`) because developers think "which token"
 *  not "which faucet." */
interface Asset {
  token: AccountRef;             // token identifier (faucet account ID)
  amount: number | bigint;       // auto-converted to bigint internally
}

type NoteVisibility = "public" | "private";

/** A note reference: hex note ID string, NoteId object, or Note object.
 *  All are accepted wherever notes are referenced. String IDs are resolved
 *  via getInputNote() internally when a full Note object is needed. */
type NoteInput = string | NoteId | Note;

// ════════════════════════════════════════════════════════════════
// Account types
// ════════════════════════════════════════════════════════════════

/** Create a wallet (default) or faucet. Discriminated by `type` field. */
type CreateAccountOptions =
  | WalletOptions
  | FaucetOptions;

interface WalletOptions {
  type?: "wallet";                 // default, can be omitted
  storage?: "private" | "public";  // default: "private"
  mutable?: boolean;               // default: true
  auth?: AuthSchemeType;           // default: "falcon"
  seed?: string;                   // optional deterministic seed
}

interface FaucetOptions {
  type: "faucet";                  // required discriminator
  symbol: string;                  // required — token symbol
  decimals: number;                // required — decimal places
  maxSupply: number | bigint;      // required — max token supply
  storage?: "private" | "public";  // default: "public"
  auth?: AuthSchemeType;           // default: "falcon"
}

interface AccountDetails {
  account: Account;
  vault: AssetVault;
  storage: AccountStorage;
  code: AccountCode | null;
  keys: Word[];
}

/** Discriminated union for account import. */
type ImportAccountInput =
  | string                                                     // bech32 or hex ID
  | { file: AccountFile }                                      // from exported file
  | { seed: Uint8Array; mutable?: boolean; auth?: AuthSchemeType }  // from seed

// ════════════════════════════════════════════════════════════════
// Transaction types
// ════════════════════════════════════════════════════════════════

interface TransactionOptions {
  waitForConfirmation?: boolean;
  timeout?: number;              // ms, default: 60_000
  prover?: TransactionProver;    // override default prover
}

interface SendOptions extends TransactionOptions {
  account: AccountRef;           // sender (executing account)
  to: AccountRef;                // recipient
  token: AccountRef;             // token to send (identified by its faucet account)
  amount: number | bigint;
  type?: NoteVisibility;         // default: "public"
  reclaimAfter?: number;         // block height
  timelockUntil?: number;        // block height
}

interface MintOptions extends TransactionOptions {
  account: AccountRef;           // faucet (executing account)
  to: AccountRef;                // recipient
  amount: number | bigint;
  type?: NoteVisibility;         // default: "public"
}

interface ConsumeOptions extends TransactionOptions {
  account: AccountRef;           // consumer (executing account)
  notes: NoteInput | NoteInput[];
}

interface ConsumeAllOptions extends TransactionOptions {
  account: AccountRef;           // consumer (executing account)
  maxNotes?: number;
}

interface SwapOptions extends TransactionOptions {
  account: AccountRef;           // swapper (executing account)
  offer: Asset;
  request: Asset;
  type?: NoteVisibility;         // default: "public"
  paybackType?: NoteVisibility;  // default: "public"
}

/** Exception to the `account` field pattern: this composed operation executes
 *  under TWO accounts (faucet mints, `to` consumes). Named fields clarify roles. */
interface MintAndConsumeOptions extends TransactionOptions {
  /** The faucet account that executes the mint (plays the role of `account` in other methods). */
  faucet: AccountRef;
  /** The account that receives the minted note AND consumes it. */
  to: AccountRef;
  amount: number | bigint;
  type?: NoteVisibility;
}

/** Explicit preview option interfaces — IntelliSense shows clean, readable shapes
 *  instead of opaque `Omit<>` intersections. Each mirrors its transaction counterpart
 *  minus the TransactionOptions fields, plus the `operation` discriminator. */

interface PreviewSendOptions {
  operation: "send";
  account: AccountRef;
  to: AccountRef;
  token: AccountRef;
  amount: number | bigint;
  type?: NoteVisibility;
  reclaimAfter?: number;
  timelockUntil?: number;
}

interface PreviewMintOptions {
  operation: "mint";
  account: AccountRef;
  to: AccountRef;
  amount: number | bigint;
  type?: NoteVisibility;
}

interface PreviewConsumeOptions {
  operation: "consume";
  account: AccountRef;
  notes: NoteInput | NoteInput[];
}

interface PreviewSwapOptions {
  operation: "swap";
  account: AccountRef;
  offer: Asset;
  request: Asset;
  type?: NoteVisibility;
  paybackType?: NoteVisibility;
}

type PreviewOptions =
  | PreviewSendOptions
  | PreviewMintOptions
  | PreviewConsumeOptions
  | PreviewSwapOptions;

/** Status values reported during waitFor polling. */
type WaitStatus = "pending" | "submitted" | "committed";

interface WaitOptions {
  timeout?: number;              // ms, default: 60_000
  interval?: number;             // polling interval ms, default: 5_000
  onProgress?: (status: WaitStatus) => void;
}

/** Result of consumeAll — includes count of remaining notes for pagination. */
interface ConsumeAllResult {
  txId: TransactionId;
  consumed: number;              // notes consumed in this call
  remaining: number;             // notes still available (0 = all consumed)
}

/** Discriminated union for transaction queries. Prevents ambiguous status + ids.
 *  Mirrors the underlying WASM TransactionFilter enum which supports exactly one
 *  filter variant at a time (All, Ids, Uncommitted, ExpiredBefore). Omit the
 *  query parameter entirely to get all transactions. */
type TransactionQuery =
  | { status: "uncommitted" }
  | { ids: string[] }
  | { expiredBefore: number };

// ════════════════════════════════════════════════════════════════
// Note types
// ════════════════════════════════════════════════════════════════

/** Discriminated union for note queries. Prevents ambiguous status + ids. */
type NoteQuery =
  | { status: "consumed" | "committed" | "expected" | "processing" | "unverified" }
  | { ids: string[] };

/** Options for standalone note creation utilities (createP2IDNote, createP2IDENote).
 *  Uses `from`/`to` (not `account`) because these are pure functions that don't
 *  execute transactions — they just describe who the note is from and to.
 *  Compare with resource methods which use `account` for the executing account. */
interface NoteOptions {
  from: AccountRef;              // sender
  to: AccountRef;                // recipient
  assets: Asset | Asset[];       // single or multiple assets
  type?: NoteVisibility;         // default: "public"
  attachment?: Felt[];           // default: empty
}

interface P2IDEOptions extends NoteOptions {
  reclaimAfter?: number;         // block height
  timelockUntil?: number;        // block height
}

/** Options for accounts.export(). Currently empty; exists for forward-compatible
 *  extensibility (e.g., future options to include/exclude keys). */
interface ExportAccountOptions {}

interface ExportNoteOptions {
  format?: "id" | "full" | "details";  // default: "full"
}

interface FetchPrivateNotesOptions {
  mode?: "incremental" | "all";  // default: "incremental"
}

interface MockOptions {
  seed?: string;                             // deterministic seed
  serializedMockChain?: Uint8Array;          // pre-built mock chain state
  serializedNoteTransport?: Uint8Array;      // pre-built mock note transport
}

interface SendPrivateOptions {
  noteId: string;                // note ID (hex)
  to: AccountRef;                // recipient address
}

/** Versioned store snapshot for backup/restore. The version field allows
 *  importStore to validate compatibility before restoring. */
interface StoreSnapshot {
  version: number;               // format version (for migration compatibility)
  data: unknown;                 // opaque store contents
}

// ════════════════════════════════════════════════════════════════
// Resource interfaces
// ════════════════════════════════════════════════════════════════

interface AccountsResource {
  create(options?: CreateAccountOptions): Promise<Account>;
  get(accountId: AccountRef): Promise<Account | null>;
  list(): Promise<AccountHeader[]>;
  getDetails(accountId: AccountRef): Promise<AccountDetails>;
  getBalance(accountId: AccountRef, tokenId: AccountRef): Promise<bigint>;

  import(input: ImportAccountInput): Promise<Account>;
  export(accountId: AccountRef, options?: ExportAccountOptions): Promise<AccountFile>;

  addAddress(accountId: AccountRef, address: string): Promise<void>;
  removeAddress(accountId: AccountRef, address: string): Promise<void>;
}

interface TransactionsResource {
  // One-call operations (build + execute + prove + submit)
  send(options: SendOptions): Promise<TransactionId>;
  mint(options: MintOptions): Promise<TransactionId>;
  consume(options: ConsumeOptions): Promise<TransactionId>;
  swap(options: SwapOptions): Promise<TransactionId>;
  consumeAll(options: ConsumeAllOptions): Promise<ConsumeAllResult>;

  // Composed workflows
  mintAndConsume(options: MintAndConsumeOptions): Promise<TransactionId>;

  // Preview (dry run — wraps executeForSummary)
  preview(options: PreviewOptions): Promise<TransactionSummary>;

  // Advanced: submit pre-built TransactionRequest
  submit(request: TransactionRequest, options?: TransactionOptions): Promise<TransactionId>;

  // Query
  list(query?: TransactionQuery): Promise<TransactionRecord[]>;

  // Wait — throws immediately on rejection, polls until committed or timeout
  waitFor(txId: string, options?: WaitOptions): Promise<void>;
}

interface NotesResource {
  // Query received notes (notes you can spend)
  list(query?: NoteQuery): Promise<InputNoteRecord[]>;
  get(noteId: string): Promise<InputNoteRecord | null>;

  // Query sent notes (notes you've created/sent to others)
  listSent(query?: NoteQuery): Promise<OutputNoteRecord[]>;

  // Available notes for a specific account (ready to consume right now)
  listAvailable(options: { account: AccountRef }): Promise<ConsumableNoteRecord[]>;

  // Import / Export
  import(noteFile: NoteFile): Promise<NoteId>;
  export(noteId: string, options?: ExportNoteOptions): Promise<NoteFile>;

  // Private note transport (fetch from transport, not chain sync)
  fetch(options?: FetchPrivateNotesOptions): Promise<void>;
  sendPrivate(options: SendPrivateOptions): Promise<void>;
}

interface TagsResource {
  add(tag: number): Promise<void>;
  remove(tag: number): Promise<void>;
  list(): Promise<number[]>;
}

interface SettingsResource {
  get<T = unknown>(key: string): Promise<T | null>;
  set(key: string, value: unknown): Promise<void>;
  remove(key: string): Promise<void>;
  listKeys(): Promise<string[]>;
}

// ════════════════════════════════════════════════════════════════
// Client
// ════════════════════════════════════════════════════════════════

declare class MidenClient {
  // Factory
  static create(options?: ClientOptions): Promise<MidenClient>;
  static createTestnet(options?: { autoSync?: boolean }): Promise<MidenClient>;
  static createMock(options?: MockOptions): Promise<MidenClient>;

  // Resources
  readonly accounts: AccountsResource;
  readonly transactions: TransactionsResource;
  readonly notes: NotesResource;
  readonly tags: TagsResource;
  readonly settings: SettingsResource;

  // Lifecycle
  sync(options?: { timeout?: number }): Promise<SyncSummary>;
  getSyncHeight(): Promise<number>;
  defaultTransactionProver(): TransactionProver;
  terminate(): void;

  // Store-level import/export
  exportStore(): Promise<StoreSnapshot>;
  importStore(snapshot: StoreSnapshot): Promise<void>;

  // Mock-only (available after createMock())
  /** Advances the mock chain by one block. Only available on mock clients. */
  proveBlock(): void;
  /** Returns true if this client uses a mock chain. */
  usesMockChain(): boolean;
  /** Serializes the mock chain state for snapshot/restore in tests. */
  serializeMockChain(): Uint8Array;
  /** Serializes the mock note transport node state. */
  serializeMockNoteTransportNode(): Uint8Array;
}

// ════════════════════════════════════════════════════════════════
// Standalone utilities (tree-shakeable)
// ════════════════════════════════════════════════════════════════

declare function createP2IDNote(options: NoteOptions): OutputNote;
declare function createP2IDENote(options: P2IDEOptions): OutputNote;

// Standalone request builders (for inspect-before-submit flow)
// Sync — the underlying WASM request builders are synchronous (no I/O)
declare function buildSendRequest(options: Omit<SendOptions, keyof TransactionOptions>): TransactionRequest;
declare function buildMintRequest(options: Omit<MintOptions, keyof TransactionOptions>): TransactionRequest;
declare function buildConsumeRequest(options: Omit<ConsumeOptions, keyof TransactionOptions>): TransactionRequest;
declare function buildSwapRequest(options: Omit<SwapOptions, keyof TransactionOptions>): TransactionRequest;

// Swap tag utility — computes the NoteTag for swap note matching
// Uses `offer`/`request` to match SwapOptions field names.
interface BuildSwapTagOptions {
  type?: NoteVisibility;                     // default: "public"
  offer: Asset;
  request: Asset;
}
declare function buildSwapTag(options: BuildSwapTagOptions): number;
```

> **Note on `Omit<>` in standalone builders:** The `Omit<>` types on standalone builder
> functions are acceptable because these are power-user utilities, not primary API surfaces.
> The primary `PreviewOptions` uses explicit interfaces for clean IntelliSense since that's
> a more commonly used API.
>
> **Re-exports:** The SDK package re-exports all WASM-generated types (`Account`, `AccountId`,
> `NoteId`, `TransactionRequest`, `TransactionRequestBuilder`, `TransactionProver`,
> `RpcClient`, etc.) so that downstream code doesn't need to import from the WASM module
> directly. `RpcClient` is available as a named export for advanced use cases (custom
> RPC calls, node inspection).

---

## Error Handling

All methods throw on failure. Errors are standard `Error` objects with:
- `message` — human-readable description with causal chain
- `help` — optional property with user-facing hint (from `ErrorHint`)

### Common Error Scenarios

| Scenario | Thrown by | Error |
|----------|-----------|-------|
| Invalid hex/bech32 string | Any method accepting ID strings | `"Invalid account ID: ..."` |
| Client not initialized | Any resource method | `"Client not initialized"` |
| RPC connection failure | `create()`, `sync()`, `transactions.send()` | `"failed to ...: transport error ..."` |
| Note ID not found | `transactions.consume()`, `notes.get()` | `"Note not found: 0x..."` |
| Transaction execution failure | `transactions.send()`, `transactions.preview()` | `"failed to execute transaction: ..."` |
| Timeout exceeded | `transactions.waitFor()`, any method with `waitForConfirmation` | `"Transaction confirmation timed out after Nms"` |
| Transaction rejected | `transactions.waitFor()` | `"Transaction rejected: ..."` (thrown immediately, no polling) |
| Client terminated | Any resource method after `terminate()` | `"Client terminated"` |
| Account not found | `accounts.getDetails()`, `accounts.getBalance()`, `accounts.export()` | `"Account not found: 0x..."` (note: `accounts.get()` returns `null` instead) |

### Composed Operation Failures

`mintAndConsume()` orchestrates multiple sub-operations. Partial failures are possible:

| Failure point | State | Recovery |
|---------------|-------|----------|
| Mint fails | No state change | Safe to retry the full `mintAndConsume()` |
| Mint succeeds, sync fails | Tokens minted on-chain, client unaware | Retry `client.sync()` until it succeeds, then `transactions.consumeAll()` |
| Mint + sync succeed, consume fails | Tokens visible locally, not consumed | Call `transactions.consumeAll()` directly |

The thrown error includes a `step` property indicating where the failure occurred:
```typescript
try {
  await client.transactions.mintAndConsume({ ... });
} catch (e) {
  if (e.step === "mint") {
    // Nothing happened on-chain — safe to retry entirely
    await client.transactions.mintAndConsume({ ... });
  } else if (e.step === "sync" || e.step === "consume") {
    // Mint succeeded on-chain — recover by syncing then consuming.
    // Retry sync with bounded attempts since it can fail transiently.
    let synced = false;
    for (let i = 0; i < 5; i++) {
      try { await client.sync(); synced = true; break; } catch { await sleep(2000); }
    }
    if (!synced) throw new Error("Sync failed after 5 attempts — mint succeeded on-chain but client state is stale. Call client.sync() manually before retrying.");
    await client.transactions.consumeAll({ account: "0xALICE" });
  }
}
```

---

## Design Notes

### WASM Types

Types such as `Account`, `AccountHeader`, `AccountId`, `AssetVault`, `AccountStorage`,
`AccountCode`, `AccountFile`, `Word`, `Felt`, `TransactionId`, `TransactionRequest`,
`TransactionRequestBuilder`, `TransactionSummary`, `TransactionRecord`, `TransactionProver`,
`InputNoteRecord`, `OutputNoteRecord`, `ConsumableNoteRecord`, `NoteId`, `NoteFile`, `Note`,
`OutputNote`, `SyncSummary`, and `RpcClient` are existing WASM-exported types from the Rust
layer. They are re-exported by the SDK package and do not need new definitions.

### Amount Types (`number | bigint`)

All amount fields accept `number | bigint` as input for convenience — developers can write
`amount: 100` instead of `amount: 100n`. Internally, all values are converted to `bigint`
via `BigInt(value)`. All **return values** use `bigint` exclusively (e.g., `getBalance()`
returns `bigint`). Be aware that comparing a `bigint` return with a `number` literal
requires the `n` suffix: `balance > 100n`.

### Multiple Client Instances

Multiple `MidenClient` instances can coexist using different `storeName` values in
`ClientOptions` for store isolation. Each instance creates its own Web Worker.
The existing Web Locks mechanism (`navigator.locks.request`) in `syncState` prevents
concurrent sync operations within a single store. Clients with different `storeName`
values operate independently.

### Future Considerations

**Event system.** A future version may add reactive events for state changes:
```typescript
client.on("sync", (summary: SyncSummary) => { /* new block */ });
client.on("noteReceived", (note: InputNoteRecord) => { /* new note */ });
```
This is not included in the initial release to keep scope manageable.

**Branded ID types.** A future version may introduce Viem-style branded types
(`type AccountIdHex = \`0x${string}\``) for compile-time validation of hex strings.
The current design uses plain `string` for simplicity and to avoid breaking changes
when branded types are introduced later.

**Paginated list returns.** A future version may change `list()` methods to return
`{ data: T[], hasMore: boolean, cursor?: string }` (Stripe-style) instead of bare
arrays. The current design returns `T[]` for simplicity. When pagination is needed,
the shape change will be breaking — consider wrapping early if this is a concern.

### `token` vs `faucet` Naming

In Miden, every token is issued by a faucet account, so a token is identified by its
faucet's account ID. The SDK uses `token` (not `faucet`) in fields that identify *which
token* — because developers think "send 100 DAG tokens" not "send 100 from the DAG faucet":

- `SendOptions.token` — which token to send
- `Asset.token` — which token this asset represents
- `getBalance(account, tokenId)` — balance of which token

The field is named `faucet` **only** where it refers to the faucet *as an executing account*:

- `MintAndConsumeOptions.faucet` — the faucet account that executes the mint
- `FaucetOptions.type: "faucet"` — you're creating a faucet account

Internally, the SDK resolves `token` fields to the faucet's `AccountId` before calling WASM.

### `from` vs `account` Naming

Standalone note creation utilities (`createP2IDNote`, `createP2IDENote`) use `from`/`to`
because they describe who a note is from and to — they are pure data constructors with
no transaction context. Resource methods (`transactions.send()`, `transactions.mint()`,
etc.) use `account` because it identifies the account *executing* the transaction, which
is a different semantic than "who the note is from." The two patterns don't conflict in
practice because developers use one or the other, never both together.

### `sendPrivate` Naming

The method is named `sendPrivate` (not `deliver` or `push`) because it directly maps to
what it does: send a note via a private transport channel. Alternatives were considered:
- `deliver` implies guaranteed delivery semantics, which this method doesn't provide
- `push` is too generic and doesn't convey the privacy aspect
- `send` is already used by `transactions.send()` for on-chain transfers

The options object (`{ noteId, to }`) makes the purpose unambiguous in practice.

### Post-Termination Behavior

After `client.terminate()`, all subsequent calls throw `Error("Client terminated")`.
The client maintains a `#terminated: boolean` flag checked at the entry point of every
resource method. The Web Worker is terminated immediately (`worker.terminate()`).
The client cannot be re-initialized — create a new `MidenClient` instance instead.
This follows the same pattern as `AbortController` — once aborted, it's done.

### Debug Logging

When `debug: true` is set in `ClientOptions`, the SDK logs structured diagnostics via
`console.debug()`. Each log entry includes: method name, arguments (with secrets
redacted), duration in ms, and result status (ok/error). Example:
```
[miden-sdk] transactions.send({ account: "0xA...", to: "0xB...", amount: 100n }) → ok (1240ms)
[miden-sdk] sync() → error: transport timeout (5012ms)
```
Debug mode is off by default and has no performance impact when disabled.

### Retry Behavior

The SDK does **not** retry failed operations internally. If `transactions.send()` fails
due to a transient network error, the error propagates to the caller. This is a
deliberate design choice:
- Retry policies are application-specific (some failures should not be retried)
- Idempotency of blockchain operations is complex (a "failed" mint may have actually
  succeeded on-chain — retrying would double-mint)
- Developers should use their own retry logic or libraries like `p-retry`

The `waitFor()` method is the exception — it polls internally (via `sync()` + check)
with configurable `interval` and `timeout`. If a transaction is rejected by the network,
`waitFor()` throws immediately rather than waiting until timeout.

### Mockability

`MidenClient.createMock()` returns the same resource interface backed by a mock chain,
suitable for integration tests. For unit tests, individual resources can be mocked
directly since they are plain TypeScript interfaces:

```typescript
import type { AccountsResource } from '@miden-sdk/miden-sdk';

const mockAccounts: AccountsResource = {
  create: vi.fn().mockResolvedValue(fakeAccount),
  get: vi.fn().mockResolvedValue(fakeAccount),
  list: vi.fn().mockResolvedValue([fakeHeader]),
  getDetails: vi.fn().mockResolvedValue(fakeDetails),
  getBalance: vi.fn().mockResolvedValue(100n),
  import: vi.fn().mockResolvedValue(fakeAccount),
  export: vi.fn().mockResolvedValue(fakeFile),
  addAddress: vi.fn().mockResolvedValue(undefined),
  removeAddress: vi.fn().mockResolvedValue(undefined),
};

// Inject into your component/service that accepts AccountsResource
const service = new MyService(mockAccounts);
```

This avoids the need for a heavyweight mock client when you only need to stub one resource.

### Standalone Builders: When and Why

The standalone builder functions (`buildSendRequest`, `buildMintRequest`, etc.) exist for
the "inspect before submit" pattern. They are **synchronous** (the underlying WASM request
constructors do no I/O) and return a `TransactionRequest` that can be logged, serialized,
or passed to `client.transactions.submit()`. Use cases:
- **Debugging:** `console.log(buildSendRequest({ ... }))` to see what will be submitted
- **Approval flows:** Show the user what a transaction will do before submitting
- **Batching:** Build multiple requests and submit them in sequence
- **Testing:** Construct requests for assertion without a live client

Most developers should use `client.transactions.send()` directly. The builders are
a power-user escape hatch.

### Note ID Resolution in `consume()`

When `consume()` receives string note IDs (e.g., `notes: ["0xnote1"]`), it resolves them
via `getInputNote()` internally to obtain the full `Note` object needed by the WASM
`newConsumeTransactionRequest()`. If a note ID doesn't resolve (not in local store),
the method throws `"Note not found: 0xnote1"`. To avoid this, `sync()` first or pass
`Note` objects directly.

### Concurrent `send()` Limitation

Miden transactions are built against the account's current nonce. Two concurrent `send()`
calls on the same account will produce transactions with the same nonce, and the second
will fail on-chain. The SDK does **not** serialize concurrent calls internally — this is
the caller's responsibility. For sequential sends, `await` each call:
```typescript
await client.transactions.send({ account: wallet, to: "0xBOB", ... });
await client.transactions.send({ account: wallet, to: "0xCHARLIE", ... });
```

### Note Export Format Mapping

`notes.export()` accepts lowercase format strings (`"id"`, `"full"`, `"details"`) which
are mapped to the WASM layer's capitalized values (`"Id"`, `"Full"`, `"Details"`)
internally. This keeps the JS API idiomatic while maintaining compatibility with the
Rust enum.

### Explicit Resource Management

`MidenClient` implements both `Symbol.dispose` and `Symbol.asyncDispose` so it works
with the TC39 [Explicit Resource Management](https://github.com/tc39/proposal-explicit-resource-management)
proposal. Since `terminate()` is synchronous, `Symbol.dispose` is the primary
implementation; `Symbol.asyncDispose` is provided for `await using` compatibility:
```typescript
{
  using client = await MidenClient.create();
  // ... use client ...
}  // client.terminate() called automatically

// Also works with `await using`:
{
  await using client = await MidenClient.create();
  // ... use client ...
}
```
This is a zero-cost addition (falls back to manual `terminate()` in runtimes without support).

---

## Implementation Plan

### Files to Modify

| File | Changes |
|------|---------|
| `crates/web-client/js/index.js` | Replace flat method forwarding with resource objects (`accounts`, `transactions`, `notes`, `tags`, `settings`) on `WebClient`. Each resource is a plain object with methods that call WASM. |
| `crates/web-client/js/types/index.d.ts` | Replace flat `WebClient` type with resource interfaces and `MidenClient` class |
| `crates/web-client/js/constants.js` | No changes needed — worker-delegated methods unchanged |
| `crates/web-client/js/workers/web-client-methods-worker.js` | No changes needed |

### Implementation Strategy

All simplified APIs are **JavaScript wrappers** that call existing Rust/WASM bindings:
- No changes to Rust code required
- Resource objects are plain JS objects constructed in the `WebClient` constructor
- Each resource method handles string→AccountId, number→BigInt, string→enum conversions
- The old JS-layer method names are **removed**; the underlying WASM methods remain
- Resource objects hold a reference to the underlying WASM `WebClient` instance

Example implementation sketch:
```javascript
/** Resolves AccountRef (string | Account | AccountId) to AccountId. */
function resolveAccountRef(ref) {
  if (typeof ref === "string") return AccountId.fromHex(ref);
  if (ref instanceof AccountId) return ref;
  return ref.id();  // Account object
}

class MidenClient {
  #wasm;          // internal WASM WebClient
  #terminated;    // post-termination guard

  constructor(wasm) {
    this.#wasm = wasm;
    this.#terminated = false;

    // Resources receive the WASM instance directly (not `this`) because
    // JS private fields (#wasm) are only accessible from the declaring class.
    this.accounts = new AccountsResource(wasm, this);
    this.transactions = new TransactionsResource(wasm, this);
    this.notes = new NotesResource(wasm, this);
    this.tags = new TagsResource(wasm, this);
    this.settings = new SettingsResource(wasm, this);
  }

  /** @internal — called by resource methods to guard post-termination use. */
  assertNotTerminated() {
    if (this.#terminated) throw new Error("Client terminated");
  }

  terminate() {
    this.#terminated = true;
    this.#wasm.terminate?.();  // terminate worker if available
  }

  // Support `using` and `await using` syntax
  [Symbol.dispose]() {
    this.terminate();
  }
  [Symbol.asyncDispose]() {
    this.terminate();
  }
}

class TransactionsResource {
  #wasm;
  #client;
  constructor(wasm, client) { this.#wasm = wasm; this.#client = client; }

  async send(opts) {
    this.#client.assertNotTerminated();

    const accountId = resolveAccountRef(opts.account);
    const targetId = resolveAccountRef(opts.to);
    const faucetId = resolveAccountRef(opts.token);
    const noteType = opts.type === "private" ? NoteType.Private : NoteType.Public;
    const amount = BigInt(opts.amount);

    const request = this.#wasm.newSendTransactionRequest(
      accountId, targetId, faucetId, noteType, amount,
      opts.reclaimAfter, opts.timelockUntil
    );

    const txId = await this.#wasm.submitNewTransaction(accountId, request);

    if (opts.waitForConfirmation) {
      await this.waitFor(txId.toString(), { timeout: opts.timeout });
    }

    return txId;
  }

  async consumeAll(opts) {
    this.#client.assertNotTerminated();
    // ... consume logic ...
    return { txId, consumed: consumedCount, remaining: remainingCount };
  }
}
```

### Web Worker Considerations

The current architecture offloads heavy operations to a dedicated Web Worker.
The following existing methods already run in the worker:
`newWallet`, `newFaucet`, `submitNewTransaction`, `submitNewTransactionWithProver`,
`executeTransaction`, `proveTransaction`, `syncState`.

**Resource methods** call through existing worker-delegated WASM methods internally.
No new worker methods are needed.

**Composed operations** (`mintAndConsume`, `consumeAll`) orchestrate multiple worker
calls sequentially. Each underlying call dispatches to the worker; the main thread is
free between steps.

### Testing

- `MidenClient.createMock()` returns the same resource interface backed by `MockWebClient`
- All resource methods work identically against the mock chain
- Composed operations (`mintAndConsume`) can be tested end-to-end against mock chain

---

## Summary

### Scorecard Target

| Dimension | Before | After | How |
|-----------|:------:|:-----:|-----|
| Discoverability | 3 | 5 | 5 resource namespaces instead of 30 flat methods. `client.` shows `accounts`, `transactions`, `notes`, `tags`, `settings`. |
| Learnability | 4 | 5 | One-call transactions (`send()` does everything). `account` field pattern learned once, works everywhere. Jargon-free names (`listSent`, `listAvailable`). Quick Start gets you sending tokens in 8 lines (no manual sync/consume ceremony). |
| Consistency | 2 | 5 | `AccountRef` everywhere (pass objects directly, no `.id().toString()`). `to` everywhere. `token` for token identifier, `faucet` only for executing account. `list`/`listSent`/`listAvailable` for collections. All resource methods async. All amounts `bigint`. Tags `number` only. `get()` → null, everything else → throw. Options objects everywhere (including `sendPrivate`, `listAvailable`). Sync builders for pure computation, async for I/O. |
| TypeScript | 3 | 5 | Discriminated unions for `CreateAccountOptions`, `ImportAccountInput`, `NoteQuery`, `TransactionQuery`. Explicit `PreviewSendOptions`/`PreviewMintOptions` (no `Omit<>` in primary APIs). `AuthScheme` const object. `AccountRef` type. Typed `WaitStatus`, `StoreSnapshot`, `ConsumeAllResult`, `BuildSwapTagOptions`. |
| Composability | 3 | 5 | Resource methods for common ops → sync standalone `buildSendRequest()` etc. for inspect-before-submit → `createP2IDNote()` + `TransactionRequestBuilder` + `submit()` for full custom. Mockable resource interfaces. `Symbol.asyncDispose` for RAII. |

### API Surface

| Resource | Methods |
|----------|---------|
| `MidenClient` | `create()`, `createTestnet()`, `createMock()`, `sync()`, `getSyncHeight()`, `defaultTransactionProver()`, `terminate()`, `exportStore()`, `importStore()`, `proveBlock()`, `usesMockChain()`, `serializeMockChain()`, `serializeMockNoteTransportNode()` |
| `client.accounts` | `create()`, `get()`, `list()`, `getDetails()`, `getBalance()`, `import()`, `export()`, `addAddress()`, `removeAddress()` |
| `client.transactions` | `send()`, `mint()`, `consume()`, `swap()`, `consumeAll()`, `mintAndConsume()`, `preview()`, `submit()`, `list()`, `waitFor()` |
| `client.notes` | `list()`, `get()`, `listSent()`, `listAvailable()`, `import()`, `export()`, `fetch()`, `sendPrivate()` |
| `client.tags` | `add()`, `remove()`, `list()` |
| `client.settings` | `get()`, `set()`, `remove()`, `listKeys()` |
| Standalone | `createP2IDNote()`, `createP2IDENote()`, `buildSendRequest()`, `buildMintRequest()`, `buildConsumeRequest()`, `buildSwapRequest()`, `buildSwapTag()`, `AuthScheme` |
