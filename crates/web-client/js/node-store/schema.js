import Database from "better-sqlite3";
import { logWebStoreError } from "./utils.js";
import * as semver from "semver";

export const CLIENT_VERSION_SETTING_KEY = "clientVersion";

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

const databaseRegistry = new Map();

const SCHEMA_SQL = `
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
`;

/**
 * Get a database instance from the registry by its ID.
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
 * Opens a database for the given network and registers it in the registry.
 */
export async function openDatabase(network, clientVersion) {
  const dbPath =
    globalThis.__MIDEN_STORE_PATH || `./${network}.sqlite`;
  console.log(
    `Opening database ${network} at ${dbPath} for client version ${clientVersion}...`
  );

  const db = new Database(dbPath);
  db.pragma("journal_mode = WAL");
  db.pragma("foreign_keys = ON");
  db.exec(SCHEMA_SQL);

  ensureClientVersion(db, clientVersion);

  databaseRegistry.set(network, db);
  console.log("Database opened successfully");
  return network;
}

function ensureClientVersion(db, clientVersion) {
  if (!clientVersion) {
    console.warn(
      "openDatabase called without a client version; skipping version enforcement."
    );
    return;
  }

  const storedVersion = getStoredClientVersion(db);
  if (!storedVersion) {
    persistClientVersion(db, clientVersion);
    return;
  }

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
      persistClientVersion(db, clientVersion);
      return;
    }
  } else {
    console.warn(
      `Failed to parse semver (${storedVersion} vs ${clientVersion}), forcing store reset.`
    );
  }

  console.warn(
    `SQLite client version mismatch (stored=${storedVersion}, expected=${clientVersion}). Resetting store.`
  );
  resetDatabase(db);
  persistClientVersion(db, clientVersion);
}

function getStoredClientVersion(db) {
  const record = db
    .prepare("SELECT value FROM settings WHERE key = ?")
    .get(CLIENT_VERSION_SETTING_KEY);
  if (!record) return null;
  return textDecoder.decode(record.value);
}

function persistClientVersion(db, clientVersion) {
  db.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
    .run(CLIENT_VERSION_SETTING_KEY, textEncoder.encode(clientVersion));
}

function resetDatabase(db) {
  const tables = db
    .prepare(
      "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    )
    .all()
    .map((r) => r.name);

  db.exec("PRAGMA foreign_keys = OFF");
  for (const table of tables) {
    db.exec(`DELETE FROM "${table}"`);
  }
  db.exec("PRAGMA foreign_keys = ON");
  // Re-insert default state_sync row
  db.prepare("INSERT OR IGNORE INTO state_sync (id, blockNum) VALUES (1, 0)").run();
}
