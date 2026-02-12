/**
 * Unified SQLite adapter interface.
 *
 * This interface abstracts over different SQLite backends so that the Rust/WASM
 * code doesn't care whether it's running in Node.js (better-sqlite3) or a
 * browser (wa-sqlite + OPFS/IDB VFS).
 */

export interface SqliteRow {
  [key: string]: unknown;
}

export interface RunResult {
  changes: number;
}

export interface SqliteAdapter {
  /** Execute a statement that modifies data. */
  run(sql: string, params?: unknown[]): RunResult;

  /** Execute a query and return all matching rows. */
  all<T extends SqliteRow = SqliteRow>(sql: string, params?: unknown[]): T[];

  /** Execute a query and return the first matching row, or undefined. */
  get<T extends SqliteRow = SqliteRow>(
    sql: string,
    params?: unknown[]
  ): T | undefined;

  /** Execute multiple statements inside a transaction. */
  transaction(fn: () => void): void;

  /** Close the database connection. */
  close(): void;
}

/**
 * Creates the appropriate SQLite adapter for the current environment.
 * In Node.js, uses better-sqlite3. In browsers, uses wa-sqlite.
 */
export async function createAdapter(dbName: string): Promise<SqliteAdapter> {
  if (
    typeof process !== "undefined" &&
    process.versions != null &&
    process.versions.node != null
  ) {
    const { NodeSqliteAdapter } = await import("./node-adapter.js");
    return new NodeSqliteAdapter(dbName);
  } else {
    // Browser adapter - will be implemented in Phase 2
    throw new Error(
      "Browser SQLite adapter not yet implemented. Use Node.js or the IndexedDB store for browsers."
    );
  }
}
