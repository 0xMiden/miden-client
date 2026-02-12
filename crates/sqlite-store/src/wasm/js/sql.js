/**
 * Generic SQL executor for the unified sqlite-store WASM backend.
 *
 * This replaces all entity-specific JS files with a single generic SQL execution layer.
 * Rust passes SQL + params, JS executes and returns results.
 */
import { getDatabase } from "./schema.js";

/**
 * Execute a SQL statement (INSERT, UPDATE, DELETE).
 * Returns the number of affected rows.
 */
export function sqlExecute(dbId, sql, params) {
  const db = getDatabase(dbId);
  const decodedParams = params.map(decodeParam);
  const result = db.run(sql, decodedParams);
  return result.changes;
}

/**
 * Execute a SELECT query and return all rows.
 * Returns an array of arrays (positional column values).
 * BLOBs are returned as Uint8Array.
 */
export function sqlQueryAll(dbId, sql, params) {
  const db = getDatabase(dbId);
  const decodedParams = params.map(decodeParam);
  const rows = db.all(sql, decodedParams);
  if (rows.length === 0) return [];

  const cols = Object.keys(rows[0]);
  return rows.map((r) => cols.map((c) => r[c]));
}

/**
 * Execute a SELECT query and return at most one row.
 * Returns an array of column values, or null if no row found.
 */
export function sqlQueryOne(dbId, sql, params) {
  const db = getDatabase(dbId);
  const decodedParams = params.map(decodeParam);
  const row = db.get(sql, decodedParams);
  if (!row) return null;

  const cols = Object.keys(row);
  return cols.map((c) => row[c]);
}

/**
 * Decode a parameter value from the Rust side.
 * Uint8Array values pass through directly (better-sqlite3 handles Buffer/Uint8Array).
 * All other types (null, number, string) pass through unchanged.
 */
function decodeParam(val) {
  return val;
}
