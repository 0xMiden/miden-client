# Migration Guide: Miden Client 0.11.x to 0.12.0

This guide covers all breaking changes in version 0.12.0 and provides step-by-step migration instructions for both Rust and TypeScript/JavaScript users.

## Table of Contents

- [MSRV Update](#msrv-update)
- [RPC Client Rename](#rpc-client-rename)
- [Store Crate Separation](#store-crate-separation)
- [Script Compilation Changes](#script-compilation-changes)
- [BlockNumber Type Unification](#blocknumber-type-unification)
- [Transaction API Refactoring](#transaction-api-refactoring)
- [WebKeyStore Remote Storage](#webkeystore-remote-storage)
- [Note Transport Integration](#note-transport-integration)
- [Account Store Upsert Semantics](#account-store-upsert-semantics)
- [AccountFile Implementation](#accountfile-implementation)
- [Account Component Templates to Packages](#account-component-templates-to-packages)
- [Removed web-tonic Feature](#removed-web-tonic-feature)
- [Troubleshooting](#troubleshooting)
- [Need Help?](#need-help)

---

## MSRV Update

**PR:** [#1415](https://github.com/0xMiden/miden-client/pull/1415)

### Summary

The Minimum Supported Rust Version (MSRV) has been incremented to **1.90**.

### Migration Steps

1. Update your Rust toolchain:
   ```bash
   rustup update stable
   rustup default 1.90  # or newer
   ```

2. Verify your version:
   ```bash
   rustc --version
   # Should show 1.90.0 or higher
   ```

---

## RPC Client Rename

**PR:** [#1360](https://github.com/0xMiden/miden-client/pull/1360)

### Summary

`TonicRpcClient` has been renamed to `GrpcClient` for clarity and consistency. The builder method `tonic_rpc_client()` has also been renamed to `grpc_client()`.

### Affected Code

**Rust:**
```diff
- use miden_client::rpc::TonicRpcClient;
+ use miden_client::rpc::GrpcClient;

- let rpc_client = Arc::new(TonicRpcClient::new(&endpoint, 10_000));
+ let rpc_client = Arc::new(GrpcClient::new(&endpoint, 10_000));
```

**TypeScript:**
No direct changes required for TypeScript users - the WebClient abstracts the RPC client internally.

### Migration Steps

1. Find and replace all occurrences of `TonicRpcClient` with `GrpcClient`
2. Find and replace all occurrences of `tonic_rpc_client()` with `grpc_client()`
3. Update import statements

### Common Errors

| Error Message | Cause | Solution |
|---------------|-------|----------|
| `cannot find type TonicRpcClient` | Old type name used | Replace with `GrpcClient` |
| `no method named tonic_rpc_client` | Old builder method | Replace with `grpc_client()` |

---

## Store Crate Separation

**PR:** [#1253](https://github.com/0xMiden/miden-client/pull/1253)

### Summary

`SqliteStore` and `WebStore` have been moved to their own separate crates for better modularity and reduced compile times.

### Affected Code

**Rust (SQLite):**
```diff
  # Cargo.toml
  [dependencies]
  miden-client = "0.12"
+ miden-client-sqlite-store = "0.12"
```

```diff
- use miden_client::store::SqliteStore;
+ use miden_client_sqlite_store::SqliteStore;
```

**Rust (WebStore/WASM):**
```diff
  # Cargo.toml
  [dependencies]
  miden-client = "0.12"
+ miden-idxdb-store = "0.12"
```

```diff
- use miden_client::store::WebStore;
+ use idxdb_store::WebStore;
```

### Migration Steps

1. Add the appropriate store crate to your `Cargo.toml`:
   - For native apps: `miden-client-sqlite-store = "0.12"`
   - For WASM apps: `miden-idxdb-store = "0.12"`
2. Update all import statements to use the new crate paths
3. Remove any feature flags related to store selection from `miden-client`

### Common Errors

| Error Message | Cause | Solution |
|---------------|-------|----------|
| `cannot find SqliteStore in miden_client::store` | Store moved to separate crate | Add `miden-client-sqlite-store` dependency |
| `cannot find WebStore in miden_client::store` | Store moved to separate crate | Add `miden-idxdb-store` dependency |

---

## Script Compilation Changes

**PR:** [#1331](https://github.com/0xMiden/miden-client/pull/1331)

### Summary

The `compileNoteScript()` method and compile methods on `TransactionScript`/`NoteScript` have been removed. Use `ScriptBuilder` instead.

### Affected Code

**Rust:**
```diff
- let note_script = NoteScript::compile(source_code)?;
+ let builder = client.script_builder();
+ let note_script = builder.compile_note_script(source_code)?;
```

```diff
- let tx_script = TransactionScript::compile(source_code)?;
+ let builder = client.script_builder();
+ let tx_script = builder.compile_tx_script(source_code)?;
```

**TypeScript:**
```diff
- const noteScript = client.compileNoteScript(sourceCode);
+ const scriptBuilder = client.createScriptBuilder();
+ const noteScript = scriptBuilder.compileNoteScript(sourceCode);
```

```diff
- const txScript = TransactionScript.compile(sourceCode);
+ const scriptBuilder = client.createScriptBuilder();
+ const txScript = scriptBuilder.compileTxScript(sourceCode);
```

### ScriptBuilder Capabilities

The new `ScriptBuilder` provides additional functionality:

```typescript
const builder = client.createScriptBuilder();

// Link modules before compilation
builder.linkModule("my_lib::module", moduleCode);

// Link libraries
builder.linkStaticLibrary(library);
builder.linkDynamicLibrary(foreignLibrary);  // For FPI

// Compile scripts
const noteScript = builder.compileNoteScript(noteCode);
const txScript = builder.compileTxScript(txCode);

// Build libraries
const library = builder.buildLibrary("lib::path", sourceCode);
```

### Migration Steps

1. Replace all direct `compile()` calls with `ScriptBuilder` methods
2. Obtain `ScriptBuilder` from client via `script_builder()` (Rust) or `createScriptBuilder()` (TypeScript)
3. If using custom modules/libraries, use `linkModule()` or `linkStaticLibrary()` before compilation

### Common Errors

| Error Message | Cause | Solution |
|---------------|-------|----------|
| `no method named compileNoteScript` | Method removed | Use `ScriptBuilder.compileNoteScript()` |
| `NoteScript::compile not found` | Static method removed | Use `client.script_builder().compile_note_script()` |

---

## BlockNumber Type Unification

**PR:** [#1415](https://github.com/0xMiden/miden-client/pull/1415)

### Summary

All block number references now use the dedicated `BlockNumber` type instead of raw `u32` values.

### Affected Code

**Rust:**
```diff
- fn get_block_header(&self, block_num: u32) -> Result<BlockHeader>;
+ fn get_block_header(&self, block_num: BlockNumber) -> Result<BlockHeader>;
```

```diff
+ use miden_client::BlockNumber;

- let block: u32 = 100;
+ let block = BlockNumber::new(100);

- store.get_block_header_by_num(0).await?;
+ store.get_block_header_by_num(BlockNumber::GENESIS).await?;
```

**TypeScript:**
```diff
- const blockNum: number = 100;
+ const blockNum = new BlockNumber(100n);
```

### Migration Steps

1. Import `BlockNumber` from `miden_client` (Rust) or the SDK (TypeScript)
2. Replace all `u32` block number parameters with `BlockNumber`
3. Use `BlockNumber::GENESIS` for genesis block references
4. Use `BlockNumber::new(n)` to create from numeric values

### Common Errors

| Error Message | Cause | Solution |
|---------------|-------|----------|
| `expected BlockNumber, found u32` | Type mismatch | Wrap value with `BlockNumber::new()` |
| `mismatched types` on store methods | API signature changed | Update all block number arguments |

---

## Transaction API Refactoring

**PR:** [#1407](https://github.com/0xMiden/miden-client/pull/1407)

### Summary

Transaction APIs have been refactored to provide more granular control over the transaction lifecycle. The new `TransactionResult` type captures execution artifacts.

### Affected Code

**Rust - New Granular API:**
```rust
// Step 1: Execute transaction locally
let tx_result: TransactionResult = client
    .execute_transaction(account_id, transaction_request)
    .await?;

// Step 2: Prove the transaction
let proven_tx: ProvenTransaction = client
    .prove_transaction(&tx_result)
    .await?;

// Step 3: Submit to network
let submission_height: BlockNumber = client
    .submit_proven_transaction(proven_tx, tx_result.clone())
    .await?;

// Step 4: Apply to local store
client.apply_transaction(&tx_result, submission_height).await?;
```

**Rust - Convenience Method (unchanged):**
```rust
// Still available for simple cases
let tx_id = client
    .submit_new_transaction(account_id, transaction_request)
    .await?;
```

**TypeScript:**
```typescript
// Granular control
const txResult = await client.executeTransaction(accountId, request);
const provenTx = await client.proveTransaction(txResult);
const height = await client.submitProvenTransaction(provenTx, txResult);
await client.applyTransaction(txResult, height);

// Or use convenience method
const txId = await client.submitNewTransaction(accountId, request);
```

### New TransactionResult Methods

```rust
tx_result.id()                    // Transaction ID
tx_result.executed_transaction()  // Full execution details
tx_result.created_notes()         // Output notes generated
tx_result.future_notes()          // Notes expected in follow-up txs
tx_result.account_delta()         // Account state changes
tx_result.consumed_notes()        // Input notes consumed
```

### Migration Steps

1. If using the simple `submit_new_transaction()`, no changes required
2. If you need granular control, switch to the new multi-step API
3. Update any code that accessed transaction internals to use `TransactionResult` methods

---

## WebKeyStore Remote Storage

**PR:** [#1371](https://github.com/0xMiden/miden-client/pull/1371)

### Summary

`WebKeyStore` now supports remote key storage and signature delegation via callbacks.

### Affected Code

**TypeScript - Default (Local Storage):**
```typescript
// No changes needed for local-only keystore
const client = await WebClient.createClient(rpcUrl, transportUrl, seed);
```

**TypeScript - External Keystore:**
```typescript
// New: delegate key operations to external service
const getKeyCb = async (pubKeyCommitment: Uint8Array) => {
  return await remoteKeyStore.get(pubKeyCommitment);
};

const insertKeyCb = async (pubKeyCommitment: Uint8Array, secretKey: Uint8Array) => {
  await remoteKeyStore.insert(pubKeyCommitment, secretKey);
};

const signCb = async (pubKeyCommitment: Uint8Array, signingInputs: Uint8Array) => {
  return await remoteKeyStore.sign(pubKeyCommitment, signingInputs);
};

const client = await WebClient.createClientWithExternalKeystore(
  rpcUrl,
  transportUrl,
  seed,
  getKeyCb,
  insertKeyCb,
  signCb
);
```

### Migration Steps

1. For local-only keystore: no changes required
2. For remote key storage: use `createClientWithExternalKeystore()` with callbacks
3. Callbacks can be sync or async (return `Promise`)

---

## Note Transport Integration

**PR:** [#1374](https://github.com/0xMiden/miden-client/pull/1374)

### Summary

Added connectivity to the Transport Layer for private note exchange, with new Client field and Store methods.

### Affected Code

**TypeScript - Client Creation:**
```diff
  const client = await WebClient.createClient(
    rpcUrl,
+   noteTransportUrl,  // NEW parameter
    seed
  );
```

**TypeScript - New Methods:**
```typescript
// Send a private note to an address
await client.sendPrivateNote(note, recipientAddress);

// Fetch private notes (with pagination)
await client.fetchPrivateNotes();

// Fetch all private notes (no pagination)
await client.fetchAllPrivateNotes();
```

### Migration Steps

1. Update `createClient()` calls to include `noteTransportUrl` parameter (can be `undefined` to disable)
2. Use `sendPrivateNote()` to send notes via transport layer
3. Use `fetchPrivateNotes()` to retrieve incoming notes

---

## Account Store Upsert Semantics

**PR:** [#1274](https://github.com/0xMiden/miden-client/pull/1274)

### Summary

Web Client account store functions changed from insert-only to upsert semantics with an explicit overwrite flag.

### Affected Code

**TypeScript:**
```diff
- await client.addAccount(account);
+ await client.newAccount(account, false);  // false = don't overwrite existing
```

```typescript
// To update an existing account
await client.newAccount(account, true);  // true = allow overwrite
```

### Migration Steps

1. Update all `addAccount()` calls to `newAccount()` with explicit overwrite flag
2. Use `false` for new account creation (throws if exists)
3. Use `true` when updating/importing existing accounts

---

## AccountFile Implementation

**PR:** [#1258](https://github.com/0xMiden/miden-client/pull/1258)

### Summary

New `AccountFile` type for WebClient enables account backup and restore with authentication keys.

### Affected Code

**TypeScript:**
```typescript
// Export account with all keys
const accountFile: AccountFile = await client.exportAccountFile(accountId);

// Serialize for storage/transfer
const bytes: Uint8Array = accountFile.serialize();

// Deserialize
const restored = AccountFile.deserialize(bytes);

// Import account file
await client.importAccountFile(accountFile);
```

### Migration Steps

1. Use `exportAccountFile()` for account backups (replaces manual account + key export)
2. Use `importAccountFile()` for account restoration (automatically restores keys)

---

## Account Component Templates to Packages

**PR:** [#1313](https://github.com/0xMiden/miden-client/pull/1313)

### Summary

`AccountComponentTemplates` have been replaced with `Packages` for account creation.

### Affected Code

**Rust:**
```diff
- use miden_client::accounts::AccountComponentTemplate;
+ use miden_client::accounts::Package;

- let template = AccountComponentTemplate::BasicWallet;
+ let package = Package::BasicWallet;
```

### Migration Steps

1. Replace `AccountComponentTemplate` imports with `Package`
2. Update account creation code to use `Package` variants

---

## Removed web-tonic Feature

**PR:** [#1268](https://github.com/0xMiden/miden-client/pull/1268)

### Summary

The `web-tonic` feature flag has been removed. WASM RPC is now handled differently.

### Affected Code

**Cargo.toml:**
```diff
  [dependencies]
- miden-client = { version = "0.12", features = ["web-tonic"] }
+ miden-client = "0.12"
```

### Migration Steps

1. Remove `web-tonic` from your feature flags
2. For WASM builds, the RPC client is automatically configured

---

## Troubleshooting

| Error | Cause | Solution |
|-------|-------|----------|
| `cannot find type TonicRpcClient` | Renamed to GrpcClient | Replace with `GrpcClient` |
| `cannot find SqliteStore in miden_client` | Moved to separate crate | Add `miden-client-sqlite-store` dependency |
| `no method named compileNoteScript` | Removed in favor of ScriptBuilder | Use `client.createScriptBuilder().compileNoteScript()` |
| `expected BlockNumber, found u32` | Type system change | Wrap with `BlockNumber::new()` |
| `feature web-tonic not found` | Feature removed | Remove from Cargo.toml features |
| `addAccount is not a function` | API changed to newAccount | Use `newAccount(account, overwrite)` |

---

## Need Help?

- [GitHub Issues](https://github.com/0xMiden/miden-client/issues)
- [Discord Community](https://discord.gg/miden)
- [Documentation](https://docs.miden.io)
