/**
 * Transaction operations for the WASM SQLite store.
 */

import { getDatabase } from "./schema.js";
import { logError, uint8ArrayToBase64 } from "./utils.js";

export interface TransactionRow {
  id: string;
  details: string; // base64
  scriptRoot: string | null;
  txScript: string | null; // base64
  blockNum: number;
  statusVariant: number;
  status: string; // base64
}

function toBase64(data: Uint8Array | ArrayBuffer | null): string {
  if (!data) return "";
  const bytes = data instanceof Uint8Array ? data : new Uint8Array(data);
  return uint8ArrayToBase64(bytes);
}

export function getTransactions(
  dbId: string,
  filter: string
): TransactionRow[] {
  try {
    const db = getDatabase(dbId);
    const rows = db.all<{
      id: string;
      script: Uint8Array | null;
      details: Uint8Array;
      status: Uint8Array;
      script_root: string | null;
      block_num: number;
      status_variant: number;
    }>(filter);

    return rows.map((row) => ({
      id: row.id,
      details: toBase64(row.details),
      scriptRoot: row.script_root,
      txScript: row.script ? toBase64(row.script) : null,
      blockNum: row.block_num,
      statusVariant: row.status_variant,
      status: toBase64(row.status),
    }));
  } catch (error) {
    logError(error, "Error fetching transactions");
    return [];
  }
}

export function insertTransactionScript(
  dbId: string,
  scriptRoot: Uint8Array,
  txScript: Uint8Array | null
): void {
  try {
    const db = getDatabase(dbId);
    // Convert scriptRoot bytes to hex string for the TEXT column
    const scriptRootHex = Array.from(scriptRoot)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    db.run(
      "INSERT OR REPLACE INTO transaction_scripts (script_root, script) VALUES (?, ?)",
      [scriptRootHex, txScript]
    );
  } catch (error) {
    logError(error, "Error inserting transaction script");
  }
}

export function upsertTransactionRecord(
  dbId: string,
  transactionId: string,
  details: Uint8Array,
  blockNum: number,
  statusVariant: number,
  status: Uint8Array,
  scriptRoot: Uint8Array | null
): void {
  try {
    const db = getDatabase(dbId);
    // Convert scriptRoot bytes to hex string if present
    const scriptRootHex = scriptRoot
      ? Array.from(scriptRoot)
          .map((b) => b.toString(16).padStart(2, "0"))
          .join("")
      : null;
    db.run(
      `INSERT OR REPLACE INTO transactions
       (id, details, script_root, block_num, status_variant, status)
       VALUES (?, ?, ?, ?, ?, ?)`,
      [transactionId, details, scriptRootHex, blockNum, statusVariant, status]
    );
  } catch (error) {
    logError(error, `Error upserting transaction: ${transactionId}`);
  }
}
