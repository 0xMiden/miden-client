# Implementation Plan: SimplifiedAPI for Miden Web-Client SDK

## Context

The `SimplifiedAPI.md` design document specifies replacing the flat 30+ method `WebClient` God Object with a resource-based `MidenClient` class: `client.accounts.*`, `client.transactions.*`, `client.notes.*`, `client.tags.*`, `client.settings.*`. Per the design doc, the old JS-layer method names (`createClient`, `newWallet`, `newFaucet`, `newMintTransactionRequest`, `newSendTransactionRequest`, etc.) **are removed** from the JS wrapper — callers must migrate to the new signatures. The underlying WASM methods remain accessible through the proxy for any edge case, but the explicit JS wrappers are deleted.

This is primarily a **JavaScript-only change**. Two small Rust additions are required (see [Required Rust Additions](#required-rust-additions) below).

## Architecture

`MidenClient` wraps the existing proxy-wrapped `WebClient`. Resource classes receive the proxy client and a WASM module accessor, handling all type conversions (string → AccountId, number → BigInt, string → enum). The worker, WASM bindings, and sync lock are unchanged.

```
MidenClient
  ├── #inner (proxy-wrapped WebClient — already forwards to WASM)
  ├── #getWasm() → lazy WASM module accessor for type construction
  ├── accounts: AccountsResource
  ├── transactions: TransactionsResource
  ├── notes: NotesResource
  ├── tags: TagsResource
  └── settings: SettingsResource
```

Resource classes receive `(inner, getWasm, client)` in their constructors:
- `inner` — the proxy-wrapped WebClient (calls methods on WASM through the proxy/worker)
- `getWasm` — a function returning the WASM module (for constructing types: `AccountId.fromHex()`, `NoteType.Public`, etc.)
- `client` — the parent `MidenClient` (for `assertNotTerminated()` checks and `defaultProver` access)

## Required Rust Additions

Two small `#[wasm_bindgen]` exports are needed before implementation:

1. **`TransactionId.fromHex(hex: &str) -> Result<TransactionId, JsValue>`** — Required by `transactions.list({ ids: [...] })` to convert hex strings to `TransactionId` objects. Currently `TransactionId` only has `toHex()`, `asBytes()`, `asElements()`, and `inner()` — no construction from strings. Without this, the `ids` query variant cannot accept string arrays as the design doc specifies.

2. **`AccountId.fromHex` panic guard** — The current `AccountId.fromHex()` uses `.unwrap()` and panics on invalid input (WASM trap). Should be changed to return `Result<AccountId, JsValue>` for graceful error handling. This is high priority since `resolveAccountRef` is called by virtually every resource method and a WASM trap can leave the module in an undefined state. *(Can be worked around with try-catch in JS as an interim measure, but the WASM trap makes error recovery unreliable.)*

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `js/utils.js` | **CREATE** | `resolveAccountRef`, `resolveAddress`, `resolveNoteType`, `resolveStorageMode`, `resolveAuthScheme`, `hashSeed` |
| `js/resources/accounts.js` | **CREATE** | AccountsResource class |
| `js/resources/transactions.js` | **CREATE** | TransactionsResource class |
| `js/resources/notes.js` | **CREATE** | NotesResource class |
| `js/resources/tags.js` | **CREATE** | TagsResource class |
| `js/resources/settings.js` | **CREATE** | SettingsResource class |
| `js/standalone.js` | **CREATE** | `createP2IDNote`, `createP2IDENote`, `buildSwapTag` |
| `js/client.js` | **CREATE** | MidenClient class composing all resources |
| `js/index.js` | **MODIFY** | Remove old flat method wrappers from `WebClient`; export `MidenClient`, `AuthScheme`, standalone utils; keep `WebClient`/`MockWebClient` classes (deprecated) |
| `js/types/index.d.ts` | **MODIFY** | Add all new type definitions from SimplifiedAPI.md |

**Unchanged files:** `js/constants.js`, `js/syncLock.js`, `js/wasm.js`, `js/workers/web-client-methods-worker.js`.

## Migration & Backward Compatibility

Per SimplifiedAPI.md (source of truth), old JS-layer method names are **removed**. The migration strategy is:

1. **`WebClient` and `MockWebClient` classes remain exported** but are marked `@deprecated` in types and emit a `console.warn` on first construction: `"WebClient is deprecated. Use MidenClient.create() instead."`
2. **Explicit JS wrapper methods** on `WebClient` (e.g., `newWallet`, `newFaucet`, `submitNewTransaction` wrappers with worker serialization) are **retained** internally — `MidenClient` resource classes call them. They are removed from the public API surface (not documented, not in types), but remain functional for the proxy to use.
3. **Worker-forwarded methods** (`newWallet`, `newFaucet`, `submitNewTransaction`, `submitNewTransactionWithProver`, `executeTransaction`, `proveTransaction`, `syncState`) stay on `WebClient` because the resource classes call them through `#inner`. These methods handle worker serialization/deserialization and must remain.
4. **Proxy pass-through** for WASM methods (e.g., `getAccount`, `getInputNotes`, `listTags`) continues to work — the proxy on the `WebClient` instance forwards any property access to `wasmWebClient`.
5. **All WASM re-exports** (`export * from "../Cargo.toml"`) remain unchanged.

## Implementation Steps

### Step 1: `js/utils.js` — Utility functions

Shared helpers used by all resource classes. Each accepts a `wasm` parameter (the WASM module) for constructing typed objects.

- **`resolveAccountRef(ref, wasm)`** — Accepts `string | Account | AccountId`.
  - Strings starting with `0x`/`0X` → wrap `wasm.AccountId.fromHex()` in try-catch to convert WASM trap into a thrown `Error("Invalid account ID: ...")`. *(Note: `AccountId.fromHex()` currently panics on invalid input — see [Required Rust Additions](#required-rust-additions).)*
  - Other strings (assumed bech32) → `wasm.AccountId.fromBech32()` (returns `Result`, naturally throws on error).
  - Objects with `.id()` method (Account) → call `.id()` to get AccountId.
  - Otherwise assume `AccountId` pass-through.

- **`resolveAddress(ref, wasm)`** — Accepts `AccountRef` and returns a WASM `Address` object.
  - String starting with `0x`/`0X` → `resolveAccountRef(ref, wasm)` → `wasm.Address.fromAccountId(accountId, null)`.
  - String starting with bech32 prefix (e.g. `mtst1`, `ma1`) → `wasm.Address.fromBech32(ref)`.
  - Account object → `.id()` → `wasm.Address.fromAccountId(accountId, null)`.
  - AccountId → `wasm.Address.fromAccountId(ref, null)`.
  - Used by `notes.sendPrivate()` and `accounts.addAddress()`/`removeAddress()`.

- **`resolveNoteType(type, wasm)`** — Maps `"public"` → `wasm.NoteType.Public` (numeric `1`), `"private"` → `wasm.NoteType.Private` (numeric `2`). Defaults to Public. *(NoteType is a `#[repr(u8)]` enum.)*

- **`resolveStorageMode(mode, wasm)`** — Maps `"private"` → `wasm.AccountStorageMode.private()`, `"public"` → `wasm.AccountStorageMode.public()`, `"network"` → `wasm.AccountStorageMode.network()`. Defaults to private. *(These are static factory methods returning struct instances.)*

- **`resolveAuthScheme(scheme, wasm)`** — Maps `"falcon"` → `wasm.AuthScheme.AuthRpoFalcon512` (numeric `0`), `"ecdsa"` → `wasm.AuthScheme.AuthEcdsaK256Keccak` (numeric `1`). Defaults to falcon. *(AuthScheme is a `#[repr(u8)]` enum.)*

- **`hashSeed(seed)`** — If string, hashes via `crypto.subtle.digest("SHA-256", new TextEncoder().encode(seed))` → `Uint8Array`. If already `Uint8Array`, pass through.

### Step 2: `js/resources/tags.js` — TagsResource (simplest, validates pattern)

Constructor receives `(inner, getWasm, client)`.

- `add(tag: number)` → `client.assertNotTerminated()`, `this.#inner.addTag(String(tag))`
- `remove(tag: number)` → `client.assertNotTerminated()`, `this.#inner.removeTag(String(tag))`
- `list()` → `client.assertNotTerminated()`, `this.#inner.listTags()` then map each element to `Number()`

### Step 3: `js/resources/settings.js` — SettingsResource

Constructor receives `(inner, getWasm, client)`.

- `get(key)` → `client.assertNotTerminated()`, `this.#inner.getSetting(key)`, normalize `undefined` → `null`
- `set(key, value)` → `client.assertNotTerminated()`, `this.#inner.setSetting(key, value)`
- `remove(key)` → `client.assertNotTerminated()`, `this.#inner.removeSetting(key)`
- `listKeys()` → `client.assertNotTerminated()`, `this.#inner.listSettingKeys()`

### Step 4: `js/resources/accounts.js` — AccountsResource

Constructor receives `(inner, getWasm, client)`.

WASM method mapping:
- **`create(opts?)`** → Discriminate on `opts.type`:
  - `"faucet"` → Resolve options: `const storageMode = resolveStorageMode(opts.storage ?? "public", wasm)`, `const authScheme = resolveAuthScheme(opts.auth, wasm)`. Call `this.#inner.newFaucet(storageMode, false, opts.symbol, opts.decimals, BigInt(opts.maxSupply), authScheme)` (worker-forwarded — the JS wrapper serializes `storageMode.asStr()` for the worker).
  - default wallet → Resolve options: `const storageMode = resolveStorageMode(opts?.storage ?? "private", wasm)`, `const authScheme = resolveAuthScheme(opts?.auth, wasm)`. Call `this.#inner.newWallet(storageMode, opts?.mutable ?? true, authScheme, opts?.seed ? await hashSeed(opts.seed) : undefined)` (worker-forwarded).
- **`get(ref)`** → `this.#inner.getAccount(resolveAccountRef(ref, wasm))`, return `null` if not found (catch/check for WASM None)
- **`list()`** → `this.#inner.getAccounts()`
- **`getDetails(ref)`** → Resolve id, then call `this.#inner.getAccount(id)`. If null, throw `"Account not found: 0x..."`. Return `{ account, vault: account.vault(), storage: account.storage(), code: account.code(), keys: this.#inner.getPublicKeyCommitmentsOfAccount(id) }`.
- **`getBalance(accountRef, tokenRef)`** → `this.#inner.accountReader(resolveAccountRef(accountRef, wasm)).getBalance(resolveAccountRef(tokenRef, wasm))`. Uses `accountReader` (async, fetches from store) per the design doc note that `getBalance()` wraps `accountReader()`.
- **`import(input)`** → Discriminate by input shape:
  - string → `await this.#inner.importAccountById(resolveAccountRef(input, wasm))`, then `await this.#inner.getAccount(resolveAccountRef(input, wasm))` to return the `Account` object. *(Note: WASM `importAccountById` returns `undefined`, not `Account` — the extra `getAccount` call is needed to satisfy the `Promise<Account>` return type.)*
  - `{file}` → `await this.#inner.importAccountFile(file)`, then extract account ID from the result string and `await this.#inner.getAccount(id)` to return the `Account`. *(Note: WASM `importAccountFile` returns a status string `"Imported account with ID: 0x..."`, not an `Account` object.)*
  - `{seed}` → `await this.#inner.importPublicAccountFromSeed(seed, mutable ?? true, resolveAuthScheme(auth, wasm))` — this WASM method directly returns `Account`, no extra call needed.
- **`export(ref)`** → `this.#inner.exportAccountFile(resolveAccountRef(ref, wasm))`
- **`addAddress(ref, addr)`** → `this.#inner.insertAccountAddress(resolveAccountRef(ref, wasm), wasm.Address.fromBech32(addr))`
- **`removeAddress(ref, addr)`** → `this.#inner.removeAccountAddress(resolveAccountRef(ref, wasm), wasm.Address.fromBech32(addr))`

### Step 5: `js/resources/notes.js` — NotesResource

Constructor receives `(inner, getWasm, client)`.

WASM method mapping:
- **`list(query?)`** → Build `NoteFilter` from query using WASM constructor: `new wasm.NoteFilter(filterType, noteIds)`. No query → `new wasm.NoteFilter(wasm.NoteFilterTypes.All, undefined)`. `{status}` → map lowercase status to `wasm.NoteFilterTypes.Committed/Consumed/Expected/Processing/Unverified`, then `new wasm.NoteFilter(filterType, undefined)`. `{ids}` → convert each string to `wasm.NoteId.fromHex(id)`, then `new wasm.NoteFilter(wasm.NoteFilterTypes.List, noteIds)`. Call `this.#inner.getInputNotes(filter)`. *(Note: `getInputNote(string)` and `NoteId.fromHex(string)` both accept the same hex string format — both use `Word::try_from(hex)` internally.)*
- **`get(noteId)`** → `this.#inner.getInputNote(noteId)`, return `null` if WASM returns None/undefined.
- **`listSent(query?)`** → Same filter logic as `list()`, call `this.#inner.getOutputNotes(filter)`.
- **`listAvailable(opts)`** → `this.#inner.getConsumableNotes(resolveAccountRef(opts.account, wasm))`. Note: WASM takes `Option<AccountId>` — we always pass a resolved value.
- **`import(noteFile)`** → `this.#inner.importNoteFile(noteFile)`
- **`export(noteId, opts?)`** → Map `opts.format` lowercase → capitalized (`"full"` → `"Full"`, `"id"` → `"Id"`, `"details"` → `"Details"`), default to `"Full"`, call `this.#inner.exportNoteFile(noteId, format)`.
- **`fetch(opts?)`** → `opts?.mode === "all"` → `this.#inner.fetchAllPrivateNotes()`, else `this.#inner.fetchPrivateNotes()`.
- **`sendPrivate(opts)`** → Get note via `await this.#inner.getInputNote(opts.noteId)` then `.toNote()`. Resolve address via `resolveAddress(opts.to, wasm)` (handles bech32 strings, hex strings, Account objects, and AccountId objects per the `AccountRef` type). Call `this.#inner.sendPrivateNote(note, address)`.

### Step 6: `js/resources/transactions.js` — TransactionsResource (most complex)

Constructor receives `(inner, getWasm, client)`. Accesses `client.defaultProver` for the client-level default prover (set from `ClientOptions.proverUrl`).

**Common pattern:** Each method calls `client.assertNotTerminated()`, resolves `AccountRef`/`NoteType` via utils, builds a `TransactionRequest`, then calls `#submitOrSubmitWithProver(accountId, request, opts?.prover)`. The helper resolves the prover (per-call > client-level > WASM default). If `opts.waitForConfirmation`, calls `waitFor()` afterward.

- **`send(opts)`** → `newSendTransactionRequest(accountId, targetId, faucetId, noteType, BigInt(amount), reclaimAfter, timelockUntil)` → submit with `accountId`.
- **`mint(opts)`** → `newMintTransactionRequest(targetId, accountId, noteType, BigInt(amount))` → submit with `accountId` (the faucet). Note: WASM signature is `(target, faucet, noteType, amount)`.
- **`consume(opts)`** → Normalize `opts.notes` to array. Resolve each note input: string → `(await this.#inner.getInputNote(str)).toNote()` (throw `"Note not found: ..."` if null), `InputNoteRecord` → `.toNote()`, `Note` → pass-through. Then `newConsumeTransactionRequest(notes)` → submit with `opts.account`.
- **`consumeAll(opts)`** → `getConsumableNotes(accountId)` → slice by `opts.maxNotes` if set → extract notes via `.inputNoteRecord().toNote()` → if no notes, return `{txId: null, consumed: 0, remaining: 0}`. Otherwise build consume request → submit → return `{txId, consumed: consumedCount, remaining: totalAvailable - consumedCount}`.
- **`swap(opts)`** → `newSwapTransactionRequest(accountId, offeredFaucetId, BigInt(offer.amount), requestedFaucetId, BigInt(request.amount), noteType, paybackNoteType)` → submit with `accountId`.
- **`mintAndConsume(opts)`** → Three steps with error tagging:
  1. Build mint request via `newMintTransactionRequest(targetId, faucetId, noteType, BigInt(amount))` + submit with `faucetId`. On error: `const err = new Error(original.message); err.step = "mint"; err.cause = original; throw err;`
  2. `await this.waitFor(mintTxId.toHex())`. On error: tag with `step: "sync"`.
  3. `getConsumableNotes(targetId)` → extract notes → build consume request → submit with `targetId`. On error: tag with `step: "consume"`.
  Returns mint `txId`.
- **`preview(opts)`** → Discriminate on `opts.operation`, build the matching request (same as send/mint/consume/swap), then call `this.#inner.executeForSummary(accountId, request)`.
- **`submit(account, request, opts?)`** → Resolve `account` via `resolveAccountRef`. Call `submitNewTransaction(accountId, request)` or `submitNewTransactionWithProver(accountId, request, prover)` if `opts.prover` is set. **Deviation from design doc:** The design doc's `submit(request, options?)` assumes account is embedded in the request, but the WASM `submitNewTransaction` requires `accountId` as a separate parameter, and `TransactionRequest` does not expose an `accountId` getter. We add `account: AccountRef` as the first positional argument. The type signature becomes: `submit(account: AccountRef, request: TransactionRequest, options?: TransactionOptions)`.
- **`list(query?)`** → No query → `TransactionFilter.all()`. `{status: "uncommitted"}` → `TransactionFilter.uncommitted()`. `{ids}` → convert each hex string to `TransactionId` via `wasm.TransactionId.fromHex(id)` *(requires [Rust addition](#required-rust-additions))*, then `TransactionFilter.ids(txIds)`. `{expiredBefore}` → `TransactionFilter.expiredBefore(n)`. Call `this.#inner.getTransactions(filter)`.
- **`waitFor(txId, opts?)`** → Poll loop: call `this.#inner.syncStateWithTimeout(0)` → query transaction by ID → check status (committed → resolve, discarded → throw immediately with `"Transaction rejected: ..."`). Configurable `timeout` (default 60_000ms) and `interval` (default 5_000ms). Calls `opts.onProgress?.(status)` with `"pending"`, `"submitted"`, or `"committed"`. Throws `"Transaction confirmation timed out after Nms"` on timeout.

Private helper: `#submitOrSubmitWithProver(accountId, request, prover)` — Prover resolution order: (1) explicit `prover` argument (from `opts.prover`), (2) client-level `#defaultProver` (from `ClientOptions.proverUrl`), (3) WASM client's internal default prover. If a prover is resolved (cases 1 or 2), delegates to `this.#inner.submitNewTransactionWithProver(accountId, request, prover)`. Otherwise delegates to `this.#inner.submitNewTransaction(accountId, request)`, which uses the WASM client's internal default.

### Step 7: `js/standalone.js` — Standalone utilities

These functions need access to the WASM module. They receive it via a module-level `_wasm` reference set by `index.js` after WASM initialization (see Step 9).

- **`createP2IDNote(opts)`** — Resolve `from`/`to` to AccountId via `resolveAccountRef`. Build `NoteAssets` from `opts.assets` (normalize single Asset to array, create `FungibleAsset(faucetId, BigInt(amount))` for each, wrap in `NoteAssets`). Call `wasm.Note.createP2IDNote(sender, target, noteAssets, noteType, attachment)` (static method on `Note` — confirmed, no `self` param). Wrap result in `OutputNote.full(note)`.
- **`createP2IDENote(opts)`** — Same as above but with `wasm.Note.createP2IDENote(sender, target, noteAssets, reclaimAfter, timelockUntil, noteType, attachment)`.
- **`buildSwapTag(opts)`** — Call `WebClient.buildSwapTag(noteType, offeredFaucetId, BigInt(offer.amount), requestedFaucetId, BigInt(request.amount))`. **Confirmed:** `build_swap_tag` has no `self` parameter in Rust — it is a static method on the WASM `WebClient` class. `copyWebClientStatics()` in `index.js` copies it to the JS `WebClient` class. This standalone utility accesses it via the injected `WebClient` reference.

**Deferred standalone builders:** `buildSendRequest`, `buildMintRequest`, `buildConsumeRequest`, `buildSwapRequest` are deferred. The underlying `newXxxTransactionRequest` WASM methods are instance methods that require `client.rng()` access. Implementing these as standalone functions would require either WASM-level `TransactionRequestBuilder` usage (complex) or passing a client reference (defeating the purpose). Planned for a future phase.

### Step 8: `js/client.js` — MidenClient class

- **Constructor** takes `(inner, getWasm, defaultProver)` where `inner` is the proxy-wrapped WebClient, `getWasm` is the WASM module accessor, and `defaultProver` is an optional `TransactionProver`. Stores `#defaultProver` for use by `TransactionsResource`. Creates all resource instances passing `(inner, getWasm, this)`. Exposes `get defaultProver()` getter for `TransactionsResource` to access.
- **`static create(options?)`** — Hashes seed if string via `hashSeed()`. Constructs `WebClient`, calls `createClient(rpcUrl, noteTransportUrl, seed, storeName)` or `createClientWithExternalKeystore(rpcUrl, noteTransportUrl, seed, storeName, getCb, insertCb, signCb)` based on `options.keystore`. If `options.proverUrl` is set, creates `wasm.TransactionProver.newRemoteProver(options.proverUrl, undefined)` as the default prover. Wraps the result in `MidenClient`. If `options.autoSync`, calls `sync()` before returning. *(Note: `options.debug` is accepted in the type signature but its implementation is deferred — see [Deferred Features](#deferred-features).)*
- **`static createTestnet(options?)`** — Calls `create()` with `autoSync: true` by default (overridable via `options.autoSync`). The testnet prover URL is not hardcoded — users who need a remote prover should pass `proverUrl` explicitly via `MidenClient.create({ proverUrl: "...", autoSync: true })`. `createTestnet()` is a convenience for `create({ autoSync: true })`.
- **`static createMock(options?)`** — Calls `MockWebClient.createClient(serializedMockChain, serializedNoteTransport, seed)` internally, wraps result.
- **Lifecycle:**
  - `sync(opts?)` → `this.#inner.syncStateWithTimeout(opts?.timeout ?? 0)`. Maps to the existing `syncStateWithTimeout` method which handles Web Locks coordination and worker delegation.
  - `getSyncHeight()` → `this.#inner.getSyncHeight()` (proxied to WASM).
  - `terminate()` → sets `#terminated = true`, calls `this.#inner.terminate()`.
  - `[Symbol.dispose]()` → calls `terminate()`.
  - `[Symbol.asyncDispose]()` → calls `terminate()`.
- **`defaultTransactionProver()`** — **Deferred.** The Rust `client.prover()` method is not exported via `#[wasm_bindgen]`. Users who need a prover can construct one directly: `TransactionProver.newLocalProver()` or `TransactionProver.newRemoteProver(endpoint, timeout)` — both are available as WASM exports. This method will be added when a `#[wasm_bindgen]` export for the client's default prover is implemented in Rust.
- **Store:** `exportStore()` → calls `this.#inner.exportStore()`, wraps in `{version: 1, data}`. `importStore(snapshot)` → calls `this.#inner.forceImportStore(snapshot.data, "")` (second param is unused `store_name` — pass empty string).
- **Mock-only:** `proveBlock()`, `usesMockChain()`, `serializeMockChain()`, `serializeMockNoteTransportNode()` — forward to `this.#inner`. These only work on mock clients; throw on non-mock clients.
- **Termination guard:** `#terminated` flag. `assertNotTerminated()` method throws `Error("Client terminated")` if flag is set.

Circular import resolution: `MidenClient` needs `WebClient`/`MockWebClient` from `index.js`. Use static property injection — `index.js` sets `MidenClient._WebClient = WebClient` after both are defined. Factory methods reference `MidenClient._WebClient`.

### Step 9: `js/index.js` — Update exports

- Export `getWasmOrThrow` (currently file-local).
- Import and re-export `MidenClient` from `./client.js`.
- Wire dependencies: `MidenClient._WebClient = WebClient; MidenClient._MockWebClient = MockWebClient; MidenClient._getWasmOrThrow = getWasmOrThrow;`
- Import and re-export standalone utilities from `./standalone.js`.
- Export `AuthScheme` constant: `{ Falcon: "falcon", ECDSA: "ecdsa" }`.
- In `ensureWasm()` callback, set the WASM module reference for standalone utilities.
- **Remove old flat methods from WebClient types** (they remain as implementation details but are no longer public API). Keep the worker-forwarded methods (`newWallet`, `newFaucet`, `submitNewTransaction`, `submitNewTransactionWithProver`, `executeTransaction`, `proveTransaction`, `syncState`, `syncStateWithTimeout`) as internal methods since resource classes call them through `#inner`.
- **Deprecation warning:** Add a `console.warn` in `WebClient` constructor (first call only): `"WebClient is deprecated. Use MidenClient.create() instead. See migration guide."`.
- Keep all existing re-exports (`MidenArrays`, `export * from "../Cargo.toml"`) for WASM type re-exports.

### Step 10: `js/types/index.d.ts` — Type declarations

Add all types from SimplifiedAPI.md "Type Definitions" section:
- `AuthScheme` constant + `AuthSchemeType`
- `ClientOptions` (with `seed?: string` — string is hashed, `Uint8Array` passthrough handled internally by `hashSeed`)
- `AccountRef`, `Asset`, `NoteVisibility`, `NoteInput`
- `CreateAccountOptions` (discriminated union: `WalletOptions | FaucetOptions`)
- `AccountDetails`, `ImportAccountInput`
- All transaction options: `TransactionOptions`, `SendOptions`, `MintOptions`, `ConsumeOptions`, `ConsumeAllOptions`, `SwapOptions`, `MintAndConsumeOptions`
- Preview options (discriminated union): `PreviewSendOptions`, `PreviewMintOptions`, `PreviewConsumeOptions`, `PreviewSwapOptions`, `PreviewOptions`
- `WaitOptions`, `WaitStatus`
- `ConsumeAllResult` — **Updated:** `txId: TransactionId | null` (null when no consumable notes exist)
- `TransactionQuery`
- Note types: `NoteQuery`, `NoteOptions`, `P2IDEOptions`, `ExportNoteOptions`, `FetchPrivateNotesOptions`, `SendPrivateOptions`
- `MockOptions`, `StoreSnapshot`, `ExportAccountOptions`
- Resource interfaces: `AccountsResource`, `TransactionsResource`, `NotesResource`, `TagsResource`, `SettingsResource`
- **Updated `TransactionsResource.submit` signature:**
  ```typescript
  submit(account: AccountRef, request: TransactionRequest, options?: TransactionOptions): Promise<TransactionId>;
  ```
  Deviation from design doc documented with JSDoc explaining that WASM requires `accountId` separately.
- `MidenClient` class declaration (with `defaultTransactionProver()` **omitted** until Rust export is added)
- Standalone function declarations: `createP2IDNote`, `createP2IDENote`, `buildSwapTag`
- Standalone request builders (`buildSendRequest`, `buildMintRequest`, `buildConsumeRequest`, `buildSwapRequest`) — **declared but documented as `@planned` / not yet implemented**
- `BuildSwapTagOptions`
- Mark `WebClient`/`MockWebClient` with `@deprecated` JSDoc

## Error Handling

All resource methods throw standard `Error` objects on failure. Specific conventions:

| Scenario | Error |
|----------|-------|
| Invalid hex/bech32 string | `"Invalid account ID: ..."` (wrapped from WASM trap/error) |
| Client terminated | `"Client terminated"` |
| Entity not found (`get()` methods) | Return `null` — **do not throw** |
| Entity not found (other methods) | `"Account not found: 0x..."` / `"Note not found: 0x..."` |
| `mintAndConsume` partial failure | Error has `.step` property: `"mint"`, `"sync"`, or `"consume"` and `.cause` with original error |
| Transaction rejected during `waitFor` | Thrown immediately: `"Transaction rejected: ..."` |
| Timeout during `waitFor` | `"Transaction confirmation timed out after Nms"` |

## Deferred Features

The following design doc features are accepted in type signatures for forward compatibility but are **not implemented** in the initial release:

1. **`ClientOptions.debug`** — Structured logging via `console.debug`. The design doc describes method-level timing and argument logging (lines 1199-1207). Deferred because it requires wrapping every resource method with instrumentation. Will be added in a follow-up.
2. **`defaultTransactionProver()`** — Accessing the client's *internal* WASM-level default prover. Requires a Rust `#[wasm_bindgen]` export of `client.prover()`. Note: `ClientOptions.proverUrl` IS implemented — it creates a `TransactionProver` at the JS level that is used as the default for all transaction methods. `defaultTransactionProver()` would additionally expose the WASM client's built-in prover, which is a different concern.
3. **Standalone request builders** — `buildSendRequest`, `buildMintRequest`, `buildConsumeRequest`, `buildSwapRequest`. The underlying WASM methods require `client.rng()` access. Planned for a future phase using WASM-level `TransactionRequestBuilder`.

## Verification

1. **Rust additions:** Add `TransactionId.fromHex()` and fix `AccountId.fromHex()` to return `Result`.
2. **Build:** `cd crates/web-client && yarn build` — verifies rollup bundles all new files correctly.
3. **Type check:** `yarn check:wasm-types` — update `scripts/check-bindgen-types.js` allowed list for new JS-only exports if needed.
4. **Integration tests:** Existing Playwright tests continue passing — the proxy still forwards to WASM, worker-forwarded methods remain.
5. **Manual smoke test:**
   ```js
   const client = await MidenClient.createMock();
   const wallet = await client.accounts.create();
   const faucet = await client.accounts.create({ type: "faucet", symbol: "DAG", decimals: 8, maxSupply: 10_000_000n });
   client.proveBlock();
   await client.sync();
   await client.transactions.mint({ account: faucet, to: wallet, amount: 1000n });
   ```
6. **Format:** `make format` before committing.
