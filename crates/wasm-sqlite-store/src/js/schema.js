/**
 * Database schema and initialization for the WASM SQLite store.
 *
 * The schema is identical to the native sqlite-store's store.sql, ensuring
 * full database file compatibility between native and WASM environments.
 */
import { createAdapter } from "./adapter.js";
import { logError } from "./utils.js";
import * as semver from "semver";
export const CLIENT_VERSION_SETTING_KEY = "clientVersion";
/**
 * The SQL schema for the database. This must match the native sqlite-store's store.sql exactly.
 */
const STORE_SQL = `
-- Table for storing database migrations data.
CREATE TABLE IF NOT EXISTS migrations (
    name  TEXT NOT NULL,
    value ANY,
    PRIMARY KEY (name),
    CONSTRAINT migration_name_is_not_empty CHECK (length(name) > 0)
) STRICT, WITHOUT ROWID;

-- Table for storing different settings in run-time, which need to persist over runs.
CREATE TABLE IF NOT EXISTS settings (
    name  TEXT NOT NULL,
    value BLOB NOT NULL,
    PRIMARY KEY (name),
    CONSTRAINT setting_name_is_not_empty CHECK (length(name) > 0)
) STRICT, WITHOUT ROWID;

-- Create account_code table
CREATE TABLE IF NOT EXISTS account_code (
    commitment TEXT NOT NULL,
    code BLOB NOT NULL,
    PRIMARY KEY (commitment)
);

-- Create account_storage table
CREATE TABLE IF NOT EXISTS account_storage (
    commitment TEXT NOT NULL,
    slot_name TEXT NOT NULL,
    slot_value TEXT NULL,
    slot_type INTEGER NOT NULL,
    PRIMARY KEY (commitment, slot_name)
) WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_account_storage_commitment ON account_storage(commitment);

-- Create storage_map_entries table
CREATE TABLE IF NOT EXISTS storage_map_entries (
    root TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (root, key)
) WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_storage_map_entries_root ON storage_map_entries(root);

-- Create account_assets table
CREATE TABLE IF NOT EXISTS account_assets (
    root TEXT NOT NULL,
    vault_key TEXT NOT NULL,
    faucet_id_prefix TEXT NOT NULL,
    asset TEXT NULL,
    PRIMARY KEY (root, vault_key)
) WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_account_assets_root ON account_assets(root);
CREATE INDEX IF NOT EXISTS idx_account_assets_root_faucet_prefix ON account_assets(root, faucet_id_prefix);

-- Create foreign_account_code table
CREATE TABLE IF NOT EXISTS foreign_account_code(
    account_id TEXT NOT NULL,
    code_commitment TEXT NOT NULL,
    PRIMARY KEY (account_id),
    FOREIGN KEY (code_commitment) REFERENCES account_code(commitment)
);

-- Create accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id UNSIGNED BIG INT NOT NULL,
    account_commitment TEXT NOT NULL UNIQUE,
    code_commitment TEXT NOT NULL,
    storage_commitment TEXT NOT NULL,
    vault_root TEXT NOT NULL,
    nonce BIGINT NOT NULL,
    account_seed BLOB NULL,
    locked BOOLEAN NOT NULL,
    PRIMARY KEY (account_commitment),
    FOREIGN KEY (code_commitment) REFERENCES account_code(commitment),
    CONSTRAINT check_seed_nonzero CHECK (NOT (nonce = 0 AND account_seed IS NULL))
);
CREATE INDEX IF NOT EXISTS idx_accounts_id_nonce ON accounts(id, nonce DESC);
CREATE INDEX IF NOT EXISTS idx_accounts_id ON accounts(id);

-- Create transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id TEXT NOT NULL,
    details BLOB NOT NULL,
    script_root TEXT,
    block_num UNSIGNED BIG INT,
    status_variant INT NOT NULL,
    status BLOB NOT NULL,
    FOREIGN KEY (script_root) REFERENCES transaction_scripts(script_root),
    PRIMARY KEY (id)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS idx_transactions_uncommitted ON transactions(status_variant);

CREATE TABLE IF NOT EXISTS transaction_scripts (
    script_root TEXT NOT NULL,
    script BLOB,
    PRIMARY KEY (script_root)
) WITHOUT ROWID;

-- Create input notes table
CREATE TABLE IF NOT EXISTS input_notes (
    note_id TEXT NOT NULL,
    assets BLOB NOT NULL,
    serial_number BLOB NOT NULL,
    inputs BLOB NOT NULL,
    script_root TEXT NOT NULL,
    nullifier TEXT NOT NULL,
    state_discriminant UNSIGNED INT NOT NULL,
    state BLOB NOT NULL,
    created_at UNSIGNED BIG INT NOT NULL,
    PRIMARY KEY (note_id),
    FOREIGN KEY (script_root) REFERENCES notes_scripts(script_root)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS idx_input_notes_state ON input_notes(state_discriminant);
CREATE INDEX IF NOT EXISTS idx_input_notes_nullifier ON input_notes(nullifier);

-- Create output notes table
CREATE TABLE IF NOT EXISTS output_notes (
    note_id TEXT NOT NULL,
    recipient_digest TEXT NOT NULL,
    assets BLOB NOT NULL,
    metadata BLOB NOT NULL,
    nullifier TEXT NULL,
    expected_height UNSIGNED INT NOT NULL,
    state_discriminant UNSIGNED INT NOT NULL,
    state BLOB NOT NULL,
    PRIMARY KEY (note_id)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS idx_output_notes_state ON output_notes(state_discriminant);
CREATE INDEX IF NOT EXISTS idx_output_notes_nullifier ON output_notes(nullifier);

-- Create note's scripts table
CREATE TABLE IF NOT EXISTS notes_scripts (
    script_root TEXT NOT NULL,
    serialized_note_script BLOB,
    PRIMARY KEY (script_root)
);

-- Create state sync table
CREATE TABLE IF NOT EXISTS state_sync (
    block_num UNSIGNED BIG INT NOT NULL,
    PRIMARY KEY (block_num)
);

-- Create tags table
CREATE TABLE IF NOT EXISTS tags (
    tag BLOB NOT NULL,
    source BLOB NOT NULL
);

-- insert initial row into state_sync table
INSERT OR IGNORE INTO state_sync (block_num)
SELECT 0
WHERE (
    SELECT COUNT(*) FROM state_sync
) = 0;

-- Create block headers table
CREATE TABLE IF NOT EXISTS block_headers (
    block_num UNSIGNED BIG INT NOT NULL,
    header BLOB NOT NULL,
    partial_blockchain_peaks BLOB NOT NULL,
    has_client_notes BOOL NOT NULL,
    PRIMARY KEY (block_num)
);
CREATE INDEX IF NOT EXISTS idx_block_headers_has_notes ON block_headers(block_num) WHERE has_client_notes = 1;

-- Create partial blockchain nodes
CREATE TABLE IF NOT EXISTS partial_blockchain_nodes (
    id UNSIGNED BIG INT NOT NULL,
    node BLOB NOT NULL,
    PRIMARY KEY (id)
) WITHOUT ROWID;

-- Create addresses table
CREATE TABLE IF NOT EXISTS addresses (
    address BLOB NOT NULL,
    account_id UNSIGNED BIG INT NOT NULL,
    PRIMARY KEY (address)
) WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_addresses_account_id ON addresses(account_id);

-- Create tracked_accounts table
CREATE TABLE IF NOT EXISTS tracked_accounts (
    id TEXT NOT NULL PRIMARY KEY
);
`;
// Global registry for database instances
const databaseRegistry = new Map();
/**
 * Get a database instance from the registry by its ID.
 * Throws if the database hasn't been opened yet.
 */
export function getDatabase(dbId) {
  const db = databaseRegistry.get(dbId);
  if (!db) {
    throw new Error(
      `Database not found for id: ${dbId}. Call openDatabase first.`
    );
  }
  return db;
}
/**
 * Opens a database for the given name and registers it in the registry.
 * Returns the database ID which can be used to retrieve the database later.
 */
export async function openDatabase(dbName, clientVersion) {
  try {
    const adapter = await createAdapter(dbName);
    // Apply schema (uses IF NOT EXISTS so it's safe to run multiple times)
    const statements = STORE_SQL.split(";")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
    for (const stmt of statements) {
      adapter.run(stmt + ";");
    }
    // Handle client version enforcement
    ensureClientVersion(adapter, clientVersion);
    databaseRegistry.set(dbName, adapter);
    return dbName;
  } catch (error) {
    logError(error, `Failed to open database: ${dbName}`);
    throw error;
  }
}
function ensureClientVersion(adapter, clientVersion) {
  if (!clientVersion) {
    console.warn(
      "openDatabase called without a client version; skipping version enforcement."
    );
    return;
  }
  const row = adapter.get("SELECT value FROM settings WHERE name = ?", [
    CLIENT_VERSION_SETTING_KEY,
  ]);
  if (!row) {
    // First time - store the version
    const encoder = new TextEncoder();
    adapter.run("INSERT OR REPLACE INTO settings (name, value) VALUES (?, ?)", [
      CLIENT_VERSION_SETTING_KEY,
      encoder.encode(clientVersion),
    ]);
    return;
  }
  const decoder = new TextDecoder();
  const storedVersion =
    row.value instanceof Uint8Array
      ? decoder.decode(row.value)
      : decoder.decode(new Uint8Array(row.value));
  if (storedVersion === clientVersion) {
    return;
  }
  const validCurrent = semver.valid(clientVersion);
  const validStored = semver.valid(storedVersion);
  if (validCurrent && validStored) {
    const parsedCurrent = semver.parse(validCurrent);
    const parsedStored = semver.parse(validStored);
    const sameMajorMinor =
      parsedCurrent?.major === parsedStored?.major &&
      parsedCurrent?.minor === parsedStored?.minor;
    if (sameMajorMinor || !semver.gt(clientVersion, storedVersion)) {
      // Compatible version - update stored version
      const encoder = new TextEncoder();
      adapter.run(
        "INSERT OR REPLACE INTO settings (name, value) VALUES (?, ?)",
        [CLIENT_VERSION_SETTING_KEY, encoder.encode(clientVersion)]
      );
      return;
    }
  }
  // Incompatible version - need to reset the database
  console.warn(
    `SQLite client version mismatch (stored=${storedVersion}, expected=${clientVersion}). Database needs to be recreated.`
  );
  // Drop all tables and recreate
  const tables = adapter.all(
    "SELECT name FROM sqlite_master WHERE type='table' AND name != 'sqlite_sequence'"
  );
  for (const table of tables) {
    adapter.run(`DROP TABLE IF EXISTS "${table.name}"`);
  }
  // Re-apply schema
  const statements = STORE_SQL.split(";")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
  for (const stmt of statements) {
    adapter.run(stmt + ";");
  }
  // Store the new version
  const encoder = new TextEncoder();
  adapter.run("INSERT OR REPLACE INTO settings (name, value) VALUES (?, ?)", [
    CLIENT_VERSION_SETTING_KEY,
    encoder.encode(clientVersion),
  ]);
}
