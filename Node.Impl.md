# Implementation Plan: Node.js Support for Miden Web Client

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Storage swap approach | **Pre-build file swap** | Copy `node-store/*.js` → `idxdb-store/src/js/*.js` before Rollup build. Clean — no Dexie/IndexedDB in the Node bundle. WASM compilation is cached; only wasm-bindgen + Rollup re-run. |
| SQLite library | **better-sqlite3** | Synchronous, fast, prebuilt binaries. All functions return `Promise.resolve(result)` to match the async interface. |
| Package structure | **Conditional export** | Same `@miden-sdk/miden-sdk` package. `package.json` `"exports"` field auto-resolves `dist-node/` for Node.js. |
| Node.js version | **22+ (LTS)** | Native fetch, crypto, WebAssembly all stable. No experimental flags needed. |

## Architecture

```
Browser consumer:  import { WebClient } from "@miden-sdk/miden-sdk"
                   → resolves to dist/index.js (Dexie/IndexedDB, Web Workers)

Node.js consumer:  import { WebClient } from "@miden-sdk/miden-sdk"
                   → resolves to dist-node/index.js (SQLite/better-sqlite3, no Workers)
```

**Same WASM binary**, different JS storage layer. The WASM binary calls 58 JS functions via wasm-bindgen. We provide two implementations:
- **Browser**: `idxdb-store/src/js/*.js` (existing, uses Dexie/IndexedDB)
- **Node.js**: `web-client/js/node-store/*.js` (new, uses better-sqlite3)

### Pre-build file swap process

```
1. Browser build:  npm run build           → dist/ (existing, untouched)
2. File swap:      cp node-store/*.js → idxdb-store/src/js/   (save originals first)
3. Node build:     rollup -c rollup.config.node.js  → dist-node/
4. Restore:        cp saved-originals/*.js → idxdb-store/src/js/
```

The Rust WASM compilation (2+ min) is cached between steps 1 and 3. Only wasm-bindgen + Rollup re-run (~30s).

## Implementation Steps

### Step 1: Create `node-store/utils.js`

**File**: `crates/web-client/js/node-store/utils.js`

Drop-in replacement for `idxdb-store/src/js/utils.js`. Provides:
- `logWebStoreError(error, context)` — Same error logging (minus Dexie-specific handling)
- `uint8ArrayToBase64(bytes)` — Same binary-to-base64 conversion
- `mapOption(value, func)` — Same option mapping helper

This file is imported by all other node-store files. No Dexie dependency.

### Step 2: Create `node-store/schema.js`

**File**: `crates/web-client/js/node-store/schema.js`

Drop-in replacement for `idxdb-store/src/js/schema.js`. Three exports:
- `openDatabase(network, clientVersion)` → Opens/creates SQLite DB, runs schema, returns `network` as dbId
- `getDatabase(dbId)` → Returns the better-sqlite3 `Database` instance from registry
- `CLIENT_VERSION_SETTING_KEY` → Constant `"clientVersion"`, used by `settings.js` to filter internal keys

**Key implementation**:

```javascript
import Database from "better-sqlite3";
import { logWebStoreError } from "./utils.js";

const databaseRegistry = new Map();

export function getDatabase(dbId) {
  const db = databaseRegistry.get(dbId);
  if (!db) throw new Error(`Database not found for id: ${dbId}. Call openDatabase first.`);
  return db;
}

export const CLIENT_VERSION_SETTING_KEY = "clientVersion";

export async function openDatabase(network, clientVersion) {
  // DB path: configurable via global, defaults to ./{network}.sqlite
  const dbPath = globalThis.__MIDEN_STORE_PATH || `./${network}.sqlite`;
  const db = new Database(dbPath);
  db.pragma("journal_mode = WAL");
  db.pragma("foreign_keys = ON");

  // Create tables (see SQL Schema section below)
  db.exec(SCHEMA_SQL);

  // Version check (same logic as idxdb-store: reset on major/minor bump)
  ensureClientVersion(db, clientVersion);

  databaseRegistry.set(network, db);
  return network;
}
```

**SQL Schema** (adapted from `sqlite-store/src/store.sql` + idxdb-store Dexie schema):

```sql
CREATE TABLE IF NOT EXISTS account_code (
  root TEXT PRIMARY KEY,
  code BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS account_storage (
  commitment TEXT NOT NULL,
  slotName TEXT NOT NULL,
  slotValue TEXT,
  slotType INTEGER NOT NULL,
  PRIMARY KEY (commitment, slotName)
);

CREATE TABLE IF NOT EXISTS storage_map_entries (
  root TEXT NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  PRIMARY KEY (root, key)
);

CREATE TABLE IF NOT EXISTS account_assets (
  root TEXT NOT NULL,
  vaultKey TEXT NOT NULL,
  faucetIdPrefix TEXT NOT NULL,
  asset TEXT,
  PRIMARY KEY (root, vaultKey)
);

CREATE TABLE IF NOT EXISTS account_auth (
  pubKeyCommitmentHex TEXT PRIMARY KEY,
  secretKeyHex TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS accounts (
  id TEXT NOT NULL,
  codeRoot TEXT NOT NULL,
  storageRoot TEXT NOT NULL,
  vaultRoot TEXT NOT NULL,
  nonce TEXT NOT NULL,
  committed INTEGER NOT NULL,
  accountSeed BLOB,
  accountCommitment TEXT NOT NULL UNIQUE,
  locked INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_accounts_id ON accounts(id);
CREATE INDEX IF NOT EXISTS idx_accounts_id_nonce ON accounts(id, nonce);

CREATE TABLE IF NOT EXISTS tracked_accounts (
  id TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS addresses (
  id TEXT NOT NULL,
  address BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS transactions (
  id TEXT PRIMARY KEY,
  details BLOB NOT NULL,
  scriptRoot TEXT,
  blockNum INTEGER NOT NULL,
  statusVariant INTEGER NOT NULL,
  status BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS transaction_scripts (
  scriptRoot TEXT PRIMARY KEY,
  txScript BLOB
);

CREATE TABLE IF NOT EXISTS input_notes (
  noteId TEXT PRIMARY KEY,
  assets BLOB NOT NULL,
  serialNumber BLOB NOT NULL,
  inputs BLOB NOT NULL,
  scriptRoot TEXT NOT NULL,
  nullifier TEXT NOT NULL,
  stateDiscriminant INTEGER NOT NULL,
  state BLOB NOT NULL,
  serializedCreatedAt TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_input_notes_state ON input_notes(stateDiscriminant);
CREATE INDEX IF NOT EXISTS idx_input_notes_nullifier ON input_notes(nullifier);

CREATE TABLE IF NOT EXISTS output_notes (
  noteId TEXT PRIMARY KEY,
  assets BLOB NOT NULL,
  recipientDigest TEXT NOT NULL,
  metadata BLOB NOT NULL,
  nullifier TEXT,
  expectedHeight INTEGER NOT NULL,
  stateDiscriminant INTEGER NOT NULL,
  state BLOB NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_output_notes_state ON output_notes(stateDiscriminant);
CREATE INDEX IF NOT EXISTS idx_output_notes_nullifier ON output_notes(nullifier);

CREATE TABLE IF NOT EXISTS notes_scripts (
  scriptRoot TEXT PRIMARY KEY,
  serializedNoteScript BLOB
);

CREATE TABLE IF NOT EXISTS state_sync (
  id INTEGER PRIMARY KEY,
  blockNum INTEGER NOT NULL DEFAULT 0
);
INSERT OR IGNORE INTO state_sync (id, blockNum) VALUES (1, 0);

CREATE TABLE IF NOT EXISTS block_headers (
  blockNum INTEGER PRIMARY KEY,
  header BLOB NOT NULL,
  partialBlockchainPeaks BLOB NOT NULL,
  hasClientNotes TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS partial_blockchain_nodes (
  id INTEGER PRIMARY KEY,
  node TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tags (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  tag TEXT NOT NULL,
  source_note_id TEXT DEFAULT '',
  source_account_id TEXT DEFAULT ''
);

CREATE TABLE IF NOT EXISTS foreign_account_code (
  accountId TEXT PRIMARY KEY,
  codeRoot TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value BLOB NOT NULL
);
```

**Column naming**: Uses camelCase to match the Dexie object property names, since the WASM code deserializes return objects by property name.

**Version management**: Same logic as idxdb-store — store client version in `settings` table, reset DB on major/minor version bump.

### Step 3: Create `node-store/accounts.js`

**File**: `crates/web-client/js/node-store/accounts.js`

22 exported functions. Each uses `getDatabase(dbId)` to get the better-sqlite3 instance and runs synchronous SQL wrapped in `Promise.resolve()`.

**Functions and their SQL patterns**:

| Function | SQL | Returns |
|----------|-----|---------|
| `getAccountIds(dbId)` | `SELECT id FROM tracked_accounts` | `string[]` |
| `getAllAccountHeaders(dbId)` | `SELECT * FROM accounts` + group by id, keep highest nonce | `{id, nonce, vaultRoot, storageRoot, codeRoot, accountSeed?, locked, committed, accountCommitment}[]` — `accountSeed` returned as base64 |
| `getAccountHeader(dbId, accountId)` | `SELECT * FROM accounts WHERE id = ? ORDER BY CAST(nonce AS INTEGER) DESC LIMIT 1` | `{id, nonce, vaultRoot, storageRoot, codeRoot, accountSeed?, locked}` — single or `null`. Note: does NOT include `committed` or `accountCommitment` (see idxdb-store accounts.js:78-87). |
| `getAccountHeaderByCommitment(dbId, commitment)` | `SELECT * FROM accounts WHERE accountCommitment = ?` | `{id, nonce, vaultRoot, storageRoot, codeRoot, accountSeed?, locked}` — single or `undefined`. Same shape as `getAccountHeader`. |
| `getAccountCode(dbId, codeRoot)` | `SELECT * FROM account_code WHERE root = ?` | `{root, code}` — `code` as base64 |
| `getAccountStorage(dbId, storageCommitment)` | `SELECT slotName, slotValue, slotType FROM account_storage WHERE commitment = ?` | `{slotName, slotValue, slotType}[]` |
| `getAccountStorageMaps(dbId, roots)` | `SELECT * FROM storage_map_entries WHERE root IN (...)` | `{root, key, value}[]` |
| `getAccountVaultAssets(dbId, vaultRoot)` | `SELECT asset FROM account_assets WHERE root = ?` | `{asset}[]` |
| `getAccountAuthByPubKeyCommitment(dbId, hex)` | `SELECT secretKeyHex FROM account_auth WHERE pubKeyCommitmentHex = ?` | `{secretKey}` or throws |
| `getAccountAddresses(dbId, accountId)` | `SELECT * FROM addresses WHERE id = ?` | `{id, address}[]` |
| `upsertAccountCode(dbId, codeRoot, code)` | `INSERT OR REPLACE INTO account_code (root, code) VALUES (?, ?)` | void |
| `upsertAccountStorage(dbId, slots)` | `INSERT OR REPLACE INTO account_storage ...` for each slot | void |
| `upsertStorageMapEntries(dbId, entries)` | `INSERT OR REPLACE INTO storage_map_entries ...` for each entry | void |
| `upsertVaultAssets(dbId, assets)` | `INSERT OR REPLACE INTO account_assets ...` for each asset | void |
| `upsertAccountRecord(dbId, id, codeRoot, storageRoot, vaultRoot, nonce, committed, commitment, accountSeed)` | `INSERT OR REPLACE INTO accounts ...` + `INSERT OR REPLACE INTO tracked_accounts ...` | void |
| `insertAccountAuth(dbId, pubKeyHex, secretKey)` | `INSERT INTO account_auth ...` | void |
| `insertAccountAddress(dbId, accountId, address)` | `INSERT OR REPLACE INTO addresses ...` | void |
| `removeAccountAddress(dbId, address)` | `DELETE FROM addresses WHERE address = ?` | void |
| `upsertForeignAccountCode(dbId, accountId, code, codeRoot)` | `INSERT OR REPLACE INTO account_code ...` + `INSERT OR REPLACE INTO foreign_account_code ...` | void |
| `getForeignAccountCode(dbId, accountIds)` | `SELECT f.accountId, c.code FROM foreign_account_code f JOIN account_code c ON f.codeRoot = c.root WHERE f.accountId IN (...)` | `{accountId, code}[]` — `code` as base64 |
| `lockAccount(dbId, accountId)` | `UPDATE accounts SET locked = 1 WHERE id = ?` | void |
| `undoAccountStates(dbId, commitments)` | `DELETE FROM accounts WHERE accountCommitment IN (...)` | void |

**Data format notes**:
- `accountSeed`: Received as `Uint8Array` (or `null`), stored as BLOB, returned as base64 string (via `uint8ArrayToBase64`)
- `code`: Received as `Uint8Array`, stored as BLOB, returned as base64 string
- `nonce`: Stored and returned as string (hex)
- `committed`: Stored as INTEGER (0/1), returned as boolean. **Must explicitly convert**: `committed: !!row.committed` — `better-sqlite3` returns integers, not booleans.
- `locked`: Stored as INTEGER (0/1), returned as boolean. **Must explicitly convert**: `locked: !!row.locked`
- `address`: Received as `Uint8Array`, stored as BLOB, returned as-is (Uint8Array/Buffer)

### Step 4: Create `node-store/chainData.js`

**File**: `crates/web-client/js/node-store/chainData.js`

9 exported functions.

| Function | SQL | Returns |
|----------|-----|---------|
| `insertBlockHeader(dbId, blockNum, header, peaks, hasClientNotes)` | `INSERT OR REPLACE INTO block_headers ...` | void |
| `insertPartialBlockchainNodes(dbId, ids, nodes)` | `INSERT OR REPLACE INTO partial_blockchain_nodes ...` per node | void |
| `getBlockHeaders(dbId, blockNumbers)` | `SELECT * FROM block_headers WHERE blockNum IN (...)` then build a lookup map and iterate `blockNumbers` in order, returning `null` for missing entries. SQL `IN (...)` does NOT preserve input order. | `({blockNum, header, partialBlockchainPeaks, hasClientNotes} | null)[]` — `header` and `peaks` as base64. `hasClientNotes` as boolean. |
| `getTrackedBlockHeaders(dbId)` | `SELECT * FROM block_headers WHERE hasClientNotes = 'true'` | Same shape as above |
| `getPartialBlockchainPeaksByBlockNum(dbId, blockNum)` | `SELECT partialBlockchainPeaks FROM block_headers WHERE blockNum = ?` | `{peaks: base64 | undefined}` |
| `getPartialBlockchainNodesAll(dbId)` | `SELECT * FROM partial_blockchain_nodes` | `{id, node}[]` |
| `getPartialBlockchainNodes(dbId, ids)` | `SELECT * FROM partial_blockchain_nodes WHERE id IN (...)` then build a lookup map and iterate `ids` in order, returning `undefined` for missing entries. SQL `IN (...)` does NOT preserve input order. | `({id, node} | undefined)[]` |
| `getPartialBlockchainNodesUpToInOrderIndex(dbId, maxId)` | `SELECT * FROM partial_blockchain_nodes WHERE id <= ?` | `{id, node}[]` |
| `pruneIrrelevantBlocks(dbId)` | `DELETE FROM block_headers WHERE hasClientNotes = 'false' AND blockNum != 0 AND blockNum != (SELECT blockNum FROM state_sync WHERE id = 1)` | void |

**Data format notes**:
- `header`, `partialBlockchainPeaks`: Received as `Uint8Array`, stored as BLOB, returned as base64
- `hasClientNotes`: Stored as TEXT "true"/"false" (matching idxdb-store), returned as boolean
- `node`: Stored and returned as TEXT (hex string)
- `id` (partial blockchain): Received as string, stored as INTEGER (`Number(id)`), returned as number

### Step 5: Create `node-store/notes.js`

**File**: `crates/web-client/js/node-store/notes.js`

11 exported functions.

| Function | Returns |
|----------|---------|
| `getOutputNotes(dbId, states)` | `{assets, recipientDigest, metadata, expectedHeight, state}[]` — `assets`, `metadata`, `state` as base64 |
| `getInputNotes(dbId, states)` | `{assets, serialNumber, inputs, createdAt, serializedNoteScript, state}[]` — blobs as base64, joins `notes_scripts` for script |
| `getInputNotesFromIds(dbId, noteIds)` | Same shape as above, filtered by noteId |
| `getInputNotesFromNullifiers(dbId, nullifiers)` | Same shape, filtered by nullifier |
| `getOutputNotesFromNullifiers(dbId, nullifiers)` | Same shape as getOutputNotes, filtered by nullifier |
| `getOutputNotesFromIds(dbId, noteIds)` | Same shape as getOutputNotes, filtered by noteId |
| `getUnspentInputNoteNullifiers(dbId)` | `string[]` — nullifiers where stateDiscriminant IN (2, 4, 5) |
| `getNoteScript(dbId, scriptRoot)` | `{scriptRoot, serializedNoteScript}` or `undefined` |
| `upsertInputNote(dbId, noteId, assets, serialNumber, inputs, scriptRoot, serializedNoteScript, nullifier, serializedCreatedAt, stateDiscriminant, state)` | void. INSERT OR REPLACE into `input_notes` + `notes_scripts` in a transaction. |
| `upsertOutputNote(dbId, noteId, assets, recipientDigest, metadata, nullifier, expectedHeight, stateDiscriminant, state)` | void. INSERT OR REPLACE into `output_notes`. |
| `upsertNoteScript(dbId, scriptRoot, serializedNoteScript)` | void. INSERT OR REPLACE into `notes_scripts`. |

**Data format notes**:
- All BLOB fields (`assets`, `serialNumber`, `inputs`, `state`, `metadata`, `serializedNoteScript`) received as `Uint8Array`, stored as BLOB, returned as base64 string
- `processInputNotes`: Joins with `notes_scripts` table to include script data in return
- `upsertInputNote`: Wraps both inserts in a SQLite transaction (`db.transaction(...)`)

### Step 6: Create `node-store/transactions.js`

**File**: `crates/web-client/js/node-store/transactions.js`

3 exported functions.

| Function | Returns |
|----------|---------|
| `getTransactions(dbId, filter)` | `{id, details, scriptRoot, txScript, blockNum, statusVariant, status}[]` — `details`, `status` as base64, `txScript` as base64 or undefined. Joins with `transaction_scripts`. |
| `insertTransactionScript(dbId, scriptRoot, txScript)` | void. `scriptRoot` received as `Uint8Array`, converted to base64 before storage. `txScript` is optional `Uint8Array`, stored as BLOB. |
| `upsertTransactionRecord(dbId, transactionId, details, blockNum, statusVariant, status, scriptRoot)` | void. `scriptRoot` is optional `Uint8Array`, converted to base64 before storage. |

**Filter protocol** (string-based):
- `"Uncommitted"` → WHERE statusVariant = 0
- `"Ids:id1,id2,id3"` → WHERE id IN (...)
- `"ExpiredPending:blockNum"` → WHERE blockNum < ? AND statusVariant NOT IN (1, 2)
- anything else → all transactions

### Step 7: Create `node-store/sync.js`

**File**: `crates/web-client/js/node-store/sync.js`

6 exported functions. This file imports from `./transactions.js`, `./notes.js`, and `./accounts.js` (same as the idxdb-store version).

| Function | Returns |
|----------|---------|
| `getNoteTags(dbId)` | `{tag, sourceNoteId, sourceAccountId}[]` — SQL columns are `source_note_id`/`source_account_id` but return objects use camelCase. Empty strings mapped to `undefined`. |
| `getSyncHeight(dbId)` | `{blockNum}` or `null` |
| `addNoteTag(dbId, tag, sourceNoteId, sourceAccountId)` | void. `tag` converted to base64 before storage. `sourceNoteId`/`sourceAccountId` stored in `source_note_id`/`source_account_id` columns, empty string for null. |
| `removeNoteTag(dbId, tag, sourceNoteId, sourceAccountId)` | Returns **delete count** (number). The Rust side uses this as `Result<usize>`. |
| `applyStateSync(dbId, stateUpdate)` | void. Complex multi-table update — see below. |
| `discardTransactions(dbId, transactions)` | void. `DELETE FROM transactions WHERE id IN (...)` |

**`tags` table column naming**: The Dexie schema uses `source_note_id` / `source_account_id` (snake_case) for index names. The JS objects stored/returned use `sourceNoteId` / `sourceAccountId` (camelCase) for the Rust serde deserialization. Our SQL schema uses snake_case columns, but `getNoteTags` maps them to camelCase in the return objects: `{ tag: row.tag, sourceNoteId: row.source_note_id || undefined, sourceAccountId: row.source_account_id || undefined }`.

The `updateCommittedNoteTags` helper (used inside `applyStateSync`) deletes tags by `source_note_id` column, matching the idxdb-store which queries `.where("source_note_id")`.

**`applyStateSync` implementation**:

This is the most complex function. It receives a `JsStateSyncUpdate` object from WASM and must atomically update multiple tables.

**Critical**: `applyStateSync` must use **inline SQL statements** directly, NOT call the async wrapper functions (like `upsertInputNote`, `upsertAccountRecord` etc.) because those return Promises, but `better-sqlite3`'s `db.transaction()` callback is synchronous. Calling `await` inside it would break atomicity.

```javascript
export async function applyStateSync(dbId, stateUpdate) {
  const db = getDatabase(dbId);
  const {
    blockNum, flattenedNewBlockHeaders, flattenedPartialBlockChainPeaks,
    newBlockNums, blockHasRelevantNotes, serializedNodeIds, serializedNodes,
    committedNoteIds, serializedInputNotes, serializedOutputNotes,
    accountUpdates, transactionUpdates,
  } = stateUpdate;

  const newBlockHeaders = reconstructFlattenedVec(flattenedNewBlockHeaders);
  const partialBlockchainPeaks = reconstructFlattenedVec(flattenedPartialBlockChainPeaks);

  // Prepare statements outside transaction for performance
  const stmts = {
    updateSyncHeight: db.prepare("UPDATE state_sync SET blockNum = ? WHERE id = 1"),
    getSyncHeight: db.prepare("SELECT blockNum FROM state_sync WHERE id = 1"),
    upsertInputNote: db.prepare(`INSERT OR REPLACE INTO input_notes
      (noteId, assets, serialNumber, inputs, scriptRoot, nullifier, stateDiscriminant, state, serializedCreatedAt)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`),
    upsertNoteScript: db.prepare(`INSERT OR REPLACE INTO notes_scripts
      (scriptRoot, serializedNoteScript) VALUES (?, ?)`),
    upsertOutputNote: db.prepare(`INSERT OR REPLACE INTO output_notes
      (noteId, assets, recipientDigest, metadata, nullifier, expectedHeight, stateDiscriminant, state)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)`),
    upsertTx: db.prepare(`INSERT OR REPLACE INTO transactions
      (id, details, blockNum, statusVariant, status, scriptRoot) VALUES (?, ?, ?, ?, ?, ?)`),
    upsertTxScript: db.prepare(`INSERT OR REPLACE INTO transaction_scripts
      (scriptRoot, txScript) VALUES (?, ?)`),
    upsertAccountStorage: db.prepare(`INSERT OR REPLACE INTO account_storage
      (commitment, slotName, slotValue, slotType) VALUES (?, ?, ?, ?)`),
    upsertStorageMap: db.prepare(`INSERT OR REPLACE INTO storage_map_entries
      (root, key, value) VALUES (?, ?, ?)`),
    upsertVaultAsset: db.prepare(`INSERT OR REPLACE INTO account_assets
      (root, vaultKey, faucetIdPrefix, asset) VALUES (?, ?, ?, ?)`),
    upsertAccount: db.prepare(`INSERT OR REPLACE INTO accounts
      (id, codeRoot, storageRoot, vaultRoot, nonce, committed, accountCommitment, accountSeed, locked)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0)`),
    upsertTracked: db.prepare(`INSERT OR REPLACE INTO tracked_accounts (id) VALUES (?)`),
    getBlockHeader: db.prepare("SELECT blockNum FROM block_headers WHERE blockNum = ?"),
    insertBlockHeader: db.prepare(`INSERT INTO block_headers
      (blockNum, header, partialBlockchainPeaks, hasClientNotes) VALUES (?, ?, ?, ?)`),
    upsertNode: db.prepare(`INSERT OR REPLACE INTO partial_blockchain_nodes (id, node) VALUES (?, ?)`),
    deleteTagByNoteId: db.prepare("DELETE FROM tags WHERE source_note_id = ?"),
  };

  // Execute everything in one atomic transaction
  const applySync = db.transaction(() => {
    // 1. Update sync height (only forward)
    const current = stmts.getSyncHeight.get();
    if (!current || current.blockNum < blockNum) {
      stmts.updateSyncHeight.run(blockNum);
    }

    // 2. Upsert input notes
    for (const note of serializedInputNotes) {
      stmts.upsertInputNote.run(note.noteId, note.noteAssets, note.serialNumber,
        note.inputs, note.noteScriptRoot, note.nullifier, note.stateDiscriminant,
        note.state, note.createdAt);
      stmts.upsertNoteScript.run(note.noteScriptRoot, note.noteScript);
    }

    // 3. Upsert output notes
    for (const note of serializedOutputNotes) {
      stmts.upsertOutputNote.run(note.noteId, note.noteAssets, note.recipientDigest,
        note.metadata, note.nullifier || null, note.expectedHeight,
        note.stateDiscriminant, note.state);
    }

    // 4. Upsert transactions + scripts
    for (const tx of transactionUpdates) {
      stmts.upsertTx.run(tx.id, tx.details, tx.blockNum, tx.statusVariant,
        tx.status, tx.scriptRoot ? uint8ArrayToBase64(tx.scriptRoot) : null);
      if (tx.scriptRoot && tx.txScript) {
        stmts.upsertTxScript.run(uint8ArrayToBase64(tx.scriptRoot), tx.txScript);
      }
    }

    // 5. Upsert account updates
    for (const acct of accountUpdates) {
      for (const slot of acct.storageSlots) {
        stmts.upsertAccountStorage.run(slot.commitment, slot.slotName, slot.slotValue, slot.slotType);
      }
      for (const entry of acct.storageMapEntries) {
        stmts.upsertStorageMap.run(entry.root, entry.key, entry.value);
      }
      for (const asset of acct.assets) {
        stmts.upsertVaultAsset.run(asset.root, asset.vaultKey, asset.faucetIdPrefix, asset.asset);
      }
      stmts.upsertAccount.run(acct.accountId, acct.codeRoot, acct.storageRoot,
        acct.assetVaultRoot, acct.nonce, acct.committed ? 1 : 0,
        acct.accountCommitment, acct.accountSeed || null);
      stmts.upsertTracked.run(acct.accountId);
    }

    // 6. Insert block headers (skip if already exists)
    for (let i = 0; i < newBlockHeaders.length; i++) {
      if (!stmts.getBlockHeader.get(newBlockNums[i])) {
        stmts.insertBlockHeader.run(newBlockNums[i], newBlockHeaders[i],
          partialBlockchainPeaks[i], (blockHasRelevantNotes[i] === 1).toString());
      }
    }

    // 7. Insert/update partial blockchain nodes
    for (let i = 0; i < serializedNodeIds.length; i++) {
      stmts.upsertNode.run(Number(serializedNodeIds[i]), serializedNodes[i]);
    }

    // 8. Delete committed note tags
    for (const noteId of committedNoteIds) {
      stmts.deleteTagByNoteId.run(noteId);
    }
  });

  applySync();
}
```

**Key difference from idxdb-store**: The idxdb version uses `await Promise.all(...)` with Dexie transactions. Our version wraps everything in a single **synchronous** `better-sqlite3` transaction — simpler and guaranteed atomic.

**`reconstructFlattenedVec`**: This helper is identical to the idxdb-store version. It unpacks `FlattenedU8Vec` WASM objects (which have `.data()` and `.lengths()` methods) into arrays of `Uint8Array`. Must be kept as-is since it operates on WASM types.

### Step 8: Create `node-store/settings.js`

**File**: `crates/web-client/js/node-store/settings.js`

4 exported functions.

| Function | SQL | Returns |
|----------|-----|---------|
| `getSetting(dbId, key)` | `SELECT * FROM settings WHERE key = ?` | `{key, value}` where value is base64, or `null` |
| `insertSetting(dbId, key, value)` | `INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)` | void |
| `removeSetting(dbId, key)` | `DELETE FROM settings WHERE key = ?` | void |
| `listSettingKeys(dbId)` | `SELECT key FROM settings` | `string[]` — filtered to exclude internal keys (like `clientVersion`) |

### Step 9: Create `node-store/export.js`

**File**: `crates/web-client/js/node-store/export.js`

1 exported function.

```javascript
export async function exportStore(dbId) {
  const db = getDatabase(dbId);
  const dbJson = {};
  // List all tables, iterate each, read all rows
  // Convert BLOBs (Buffer) to arrays of numbers for JSON serialization
  // Same recursive transform as idxdb-store (Uint8Array → number[], Blob → base64)
  return JSON.stringify(dbJson);
}
```

The table list is hardcoded (same 19 tables as schema). For each table, `SELECT * FROM tableName`, convert Buffer columns to `Array.from(buffer)`, serialize to JSON.

### Step 10: Create `node-store/import.js`

**File**: `crates/web-client/js/node-store/import.js`

1 exported function.

```javascript
export async function forceImportStore(dbId, jsonStr) {
  const db = getDatabase(dbId);
  // Match existing idxdb-store behavior exactly:
  // 1. Parse once (may get a string if double-encoded)
  // 2. If result is still a string, parse again
  let dbJson = JSON.parse(jsonStr);
  if (typeof dbJson === "string") {
    dbJson = JSON.parse(dbJson);
  }

  db.transaction(() => {
    // Clear all tables
    // For each table in JSON: INSERT rows (convert number[] back to Buffer for BLOBs)
  })();
}
```

**Note**: The `jsonStr` parameter arrives as `JsValue` from Rust, which wasm-bindgen converts to a JS value. If it's already a string, `JSON.parse` works directly. If it's a double-encoded string, the second parse handles it. This matches the existing idxdb-store behavior exactly.

### Step 11: Create `rollup.config.node.js`

**File**: `crates/web-client/rollup.config.node.js`

This config is simpler than the browser config — no Worker build, reuses the same Rust plugin for WASM compilation (cached).

```javascript
import rust from "@wasm-tool/rollup-plugin-rust";
import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";

const devMode = process.env.MIDEN_WEB_DEV === "true";

const cargoArgsUseDebugSymbols = [
  "--config", "profile.release.debug='full'",
  "--config", "profile.release.strip='none'",
];

const baseCargoArgs = [
  "--features", "testing",
  "--config", `build.rustflags=["-C", "target-feature=+atomics,+bulk-memory,+mutable-globals", "-C", "link-arg=--max-memory=4294967296", "-C", "panic=abort"]`,
  "--no-default-features",
].concat(devMode ? cargoArgsUseDebugSymbols : []);

const wasmOptArgs = [
  devMode ? "-O0" : "-O3",
  "--enable-bulk-memory",
  "--enable-nontrapping-float-to-int",
];

export default [
  {
    input: ["./js/node-entry.js"],
    output: {
      dir: "dist-node",
      format: "es",
      sourcemap: true,
      assetFileNames: "assets/[name][extname]",
    },
    external: [
      "better-sqlite3",
      "undici",
      /^node:/,
    ],
    plugins: [
      rust({
        verbose: true,
        extraArgs: {
          cargo: [...baseCargoArgs],
          wasmOpt: wasmOptArgs,
          wasmBindgen: devMode ? ["--keep-debug"] : [],
        },
        experimental: {
          typescriptDeclarationDir: "dist-node/crates",
        },
        optimize: { release: true, rustc: !devMode },
      }),
      resolve(),
      commonjs(),
    ],
  },
  // No worker build — Node.js uses main-thread fallback
];
```

**Key differences from browser config**:
- Single entry point (`node-entry.js`), no worker build
- `external: ["better-sqlite3", "undici", /^node:/]` — don't bundle native modules
- Output to `dist-node/` instead of `dist/`
- Supports `MIDEN_WEB_DEV=true` for debug builds (same env var as browser config)
- Reuses cached WASM compilation from browser build when Rust sources haven't changed

### Step 12: Create `node-entry.js`

**File**: `crates/web-client/js/node-entry.js`

```javascript
// Node.js polyfills — applied before WASM imports
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { Agent, setGlobalDispatcher } from "undici";

// 1. HTTP/2 fetch — gRPC-Web requires HTTP/2
setGlobalDispatcher(new Agent({ allowH2: true }));

// 2. file:// fetch interception — WASM loads via fetch(new URL("...wasm", import.meta.url))
const _originalFetch = globalThis.fetch;
globalThis.fetch = async function patchedFetch(input, init) {
  let url;
  if (input instanceof URL) url = input;
  else if (input instanceof Request) url = new URL(input.url);
  else if (typeof input === "string") {
    try { url = new URL(input); } catch { return _originalFetch(input, init); }
  }
  if (url && url.protocol === "file:") {
    const buffer = readFileSync(fileURLToPath(url));
    return new Response(buffer, {
      status: 200,
      headers: { "Content-Type": "application/wasm" },
    });
  }
  return _originalFetch(input, init);
};

// 3. globalThis.self — used by some wasm-bindgen glue code
if (typeof globalThis.self === "undefined") {
  globalThis.self = globalThis;
}

// Re-export everything from the main entry
export * from "./index.js";
```

**Note**: No `fake-indexeddb` needed — the pre-build file swap eliminates all Dexie/IndexedDB code from the bundle. Only `undici` (for HTTP/2 fetch) is required as a runtime dependency.

### Step 13: Update `package.json`

**File**: `crates/web-client/package.json`

Add conditional exports and new scripts:

```json
{
  "exports": {
    ".": {
      "node": {
        "import": "./dist-node/index.js"
      },
      "default": {
        "import": "./dist/index.js"
      }
    }
  },
  "scripts": {
    "build-node": "node scripts/build-node.mjs"
  },
  "optionalDependencies": {
    "better-sqlite3": "^11.0.0",
    "undici": "^7.21.0"
  }
}
```

`better-sqlite3` and `undici` are `optionalDependencies` so browser consumers skip them.

### Step 14: Create build script

**File**: `crates/web-client/scripts/build-node.mjs`

Automates the file swap + build + restore:

```javascript
#!/usr/bin/env node
import { cpSync, mkdirSync, rmSync, existsSync } from "node:fs";
import { execSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const webClientDir = resolve(__dirname, "..");
const idxdbJsDir = resolve(webClientDir, "../idxdb-store/src/js");
const nodeStoreDir = resolve(webClientDir, "js/node-store");
const backupDir = resolve(webClientDir, ".idxdb-backup");

// 1. Backup original idxdb-store JS files
console.log("Backing up idxdb-store/src/js/ ...");
if (existsSync(backupDir)) rmSync(backupDir, { recursive: true });
mkdirSync(backupDir, { recursive: true });
cpSync(idxdbJsDir, backupDir, { recursive: true });

try {
  // 2. Copy node-store files over idxdb-store
  console.log("Swapping in node-store files ...");
  cpSync(nodeStoreDir, idxdbJsDir, { recursive: true });

  // 3. Build with node Rollup config
  console.log("Building dist-node/ ...");
  execSync("npx rollup -c rollup.config.node.js", {
    cwd: webClientDir,
    stdio: "inherit",
    env: { ...process.env, ROLLUP_NODE_BUILD: "true" },
  });
} finally {
  // 4. Restore original files (always, even on error)
  console.log("Restoring original idxdb-store/src/js/ ...");
  cpSync(backupDir, idxdbJsDir, { recursive: true });
  rmSync(backupDir, { recursive: true });
}

console.log("Node.js build complete → dist-node/");
```

### Step 15: Add Makefile targets

**File**: `Makefile` (add to existing)

```makefile
.PHONY: build-wasm-node test-node-client

## Build Node.js variant of web-client (reuses cached WASM)
build-wasm-node:
	cd crates/web-client && npm run build-node

## Run Node.js integration tests against devnet
test-node-client:
	cd crates/web-client/test/node && npm install && node integration.mjs
```

### Step 16: Update integration tests

**File**: `crates/web-client/test/node/integration.mjs`

Update to use the Node.js build (`dist-node/`) instead of the browser build (`dist/`):

```javascript
// Change:
const distDir = resolve(dirname(fileURLToPath(import.meta.url)), "../../dist");
// To:
const distDir = resolve(dirname(fileURLToPath(import.meta.url)), "../../dist-node");
```

Remove the `import "./setup.mjs"` line — the Node entry point (`dist-node/index.js`) includes all polyfills automatically. Also remove `fake-indexeddb` from `test/node/package.json` dependencies.

Add a **persistence test** after the existing tests:

```javascript
// ── 19. Persistence test ──────────────────────────────────
log("19", "Testing persistence across client restart...");
client.terminate();

// Create a new client pointing to the same SQLite DB
globalThis.__MIDEN_STORE_PATH = "./test-miden-store.sqlite";
const client2 = await sdk.WebClient.createClient(RPC_URL);
const accounts2 = await client2.getAccounts();
log("19", `Accounts after restart: ${accounts2.length} (expected: 3)`);
assert(accounts2.length === 3, `expected 3 accounts after restart, got ${accounts2.length}`);
client2.terminate();
```

## File Summary

### Files to create (13)

| File | Lines (est.) | Description |
|------|-------------|-------------|
| `js/node-store/utils.js` | ~30 | Error logging, base64 conversion |
| `js/node-store/schema.js` | ~120 | SQLite init, DB registry, version management |
| `js/node-store/accounts.js` | ~350 | 22 account CRUD functions |
| `js/node-store/chainData.js` | ~150 | 9 block header / MMR node functions |
| `js/node-store/notes.js` | ~180 | 11 input/output note functions |
| `js/node-store/transactions.js` | ~100 | 3 transaction functions |
| `js/node-store/sync.js` | ~180 | 6 sync functions incl. `applyStateSync` |
| `js/node-store/settings.js` | ~50 | 4 settings functions |
| `js/node-store/export.js` | ~50 | 1 store export function |
| `js/node-store/import.js` | ~50 | 1 store import function |
| `js/node-entry.js` | ~30 | Node.js polyfills + re-export |
| `rollup.config.node.js` | ~50 | Rollup config for Node build |
| `scripts/build-node.mjs` | ~40 | File swap + build automation |

### Files to modify (3)

| File | Change |
|------|--------|
| `crates/web-client/package.json` | Add `exports`, `optionalDependencies`, `build-node` script |
| `Makefile` | Add `build-wasm-node` and `test-node-client` targets |
| `crates/web-client/test/node/integration.mjs` | Point to `dist-node/`, remove `setup.mjs` import, add persistence test |

### Files unchanged
- All Rust code
- `crates/idxdb-store/` (only temporarily swapped during build, restored after)
- `crates/web-client/dist/` (browser build untouched)
- `crates/web-client/js/index.js` (Worker fallback already handles Node.js)

## Implementation Order

1. **`utils.js`** + **`schema.js`** — Foundation
2. **`settings.js`** — Simplest module (4 functions), good for validating the approach
3. **`rollup.config.node.js`** + **`scripts/build-node.mjs`** + **`node-entry.js`** — Build pipeline
4. **Build and smoke test** — Create client, sync, verify `getSyncHeight` works
5. **`accounts.js`** — Largest module (22 functions)
6. **`chainData.js`** — Block headers and MMR nodes
7. **`notes.js`** — Input/output notes
8. **`transactions.js`** — Transaction records
9. **`sync.js`** — State sync (most complex function: `applyStateSync`)
10. **`export.js`** + **`import.js`** — Store serialization
11. **Update `package.json`** — Conditional exports
12. **Update `Makefile`** — Build/test targets
13. **Full integration test** — Run `integration.mjs` against devnet
14. **Persistence test** — Verify data survives client restart

Steps 1-4 are a minimal prototype that validates the entire approach (file swap, Rollup build, WASM loading, RPC). If the smoke test passes, steps 5-14 are mechanical.

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Rollup `rust` plugin doesn't cache WASM compilation | **Medium** | If it recompiles, the Node build takes ~3 min instead of ~30s. Acceptable for CI. Can cache `target/` directory. |
| File swap fails to restore on build error | **Low** | `finally` block in `build-node.mjs` always restores. `.idxdb-backup/` is a safety net. |
| `better-sqlite3` Buffer vs Uint8Array mismatch | **Medium** | `better-sqlite3` returns `Buffer` for BLOB columns. `Buffer` extends `Uint8Array` in Node.js so it should be compatible, but test carefully. If needed, wrap returns in `new Uint8Array(buffer)`. |
| `applyStateSync` atomicity | **Low** | SQLite transactions are truly atomic (unlike Dexie which can fail partially in edge cases). This is actually safer. |
| `reconstructFlattenedVec` depends on WASM types | **Low** | The function only calls `.data()` and `.lengths()` on the WASM object. These are wasm-bindgen generated methods that work identically in both builds. |

## Implementation Gotchas (from code review)

These are non-obvious details that must be handled correctly during implementation:

1. **`getAccountAuthByPubKeyCommitment`**: SQL column is `secretKeyHex`, but return object must use property name `secretKey` (not `secretKeyHex`). The Rust `AccountAuthIdxdbObject` deserializes `secret_key` → `secretKey`.

2. **`getAccountHeader` / `getAccountHeaderByCommitment`**: Do NOT include `committed` or `accountCommitment` in return. Only `getAllAccountHeaders` includes those fields. See `idxdb-store/src/js/accounts.js:78-87`.

3. **`tags` table uses snake_case columns** (`source_note_id`, `source_account_id`), but Rust expects camelCase (`sourceNoteId`, `sourceAccountId`). The `getNoteTags` function must map: `{ sourceNoteId: row.source_note_id || undefined }`.

4. **`getBlockHeaders` and `getPartialBlockchainNodes` must preserve input order**. SQL `WHERE x IN (...)` does NOT preserve order. Build a Map from query results, then iterate the input array to produce ordered output with `null`/`undefined` for missing entries.

5. **`better-sqlite3` returns integers, not booleans**. Every function returning `committed` or `locked` fields must explicitly convert: `!!row.committed`.

6. **`applyStateSync` must use inline SQL, not async wrappers**. The `db.transaction()` callback is synchronous. Calling the async `upsertInputNote()` etc. would break transactional atomicity. Use prepared statements directly.

7. **`removeNoteTag` must return the delete count** (number), not void. Rust deserializes this as `Result<usize>`.

8. **`getNoteScript` return shape**: Match the existing idxdb-store exactly (`{scriptRoot, serializedNoteScript}`). The Rust model expects `noteScriptRoot`, but the existing browser code returns `scriptRoot` and works — follow that pattern.

9. **`insertTransactionScript`**: The `scriptRoot` param arrives as `Uint8Array`, must be converted to base64 before storage. The `txScript` param is `Option<Vec<u8>>` — store as BLOB (or null).

10. **`exportStore` / `forceImportStore` format compatibility**: The export format must be compatible between browser (Dexie/IndexedDB) and Node (SQLite). Both must convert `Uint8Array`/`Buffer` to number arrays (`Array.from(buffer)`) in the export, and convert back on import.
