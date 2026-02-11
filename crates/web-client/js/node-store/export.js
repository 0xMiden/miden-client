import { getDatabase } from "./schema.js";
import { uint8ArrayToBase64 } from "./utils.js";

const TABLE_NAMES = [
  "account_code",
  "account_storage",
  "storage_map_entries",
  "account_assets",
  "account_auth",
  "accounts",
  "tracked_accounts",
  "addresses",
  "transactions",
  "transaction_scripts",
  "input_notes",
  "output_notes",
  "notes_scripts",
  "state_sync",
  "block_headers",
  "partial_blockchain_nodes",
  "tags",
  "foreign_account_code",
  "settings",
];

function transformValueForExport(value) {
  if (value instanceof Uint8Array || Buffer.isBuffer(value)) {
    return Array.from(value);
  }
  if (Array.isArray(value)) {
    return value.map(transformValueForExport);
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([k, v]) => [k, transformValueForExport(v)])
    );
  }
  return value;
}

export async function exportStore(dbId) {
  const db = getDatabase(dbId);
  const dbJson = {};

  for (const tableName of TABLE_NAMES) {
    const rows = db.prepare(`SELECT * FROM "${tableName}"`).all();
    dbJson[tableName] = rows.map(transformValueForExport);
  }

  return JSON.stringify(dbJson);
}
