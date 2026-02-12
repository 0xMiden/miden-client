/**
 * Node.js SQLite adapter using better-sqlite3.
 *
 * better-sqlite3 provides a synchronous API which is the fastest SQLite
 * option for Node.js. It's a native addon that links against the system
 * or bundled SQLite.
 */

import type { SqliteAdapter, SqliteRow, RunResult } from "./adapter.js";

// better-sqlite3 types - we use dynamic import since it's a native addon
// that may not be available in all environments.
interface BetterSqlite3Database {
  prepare(sql: string): BetterSqlite3Statement;
  exec(sql: string): void;
  transaction<T>(fn: () => T): () => T;
  close(): void;
  pragma(pragma: string): unknown;
}

interface BetterSqlite3Statement {
  run(...params: unknown[]): { changes: number };
  all(...params: unknown[]): unknown[];
  get(...params: unknown[]): unknown;
}

export class NodeSqliteAdapter implements SqliteAdapter {
  private db: BetterSqlite3Database;

  constructor(dbPath: string) {
    // Dynamic require for better-sqlite3 since it's a native addon
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const Database = require("better-sqlite3");
    this.db = new Database(dbPath) as BetterSqlite3Database;

    // Enable WAL mode for better concurrent read performance
    this.db.pragma("journal_mode = WAL");
    // Enable foreign keys
    this.db.pragma("foreign_keys = ON");
  }

  run(sql: string, params: unknown[] = []): RunResult {
    const stmt = this.db.prepare(sql);
    const result = stmt.run(...params);
    return { changes: result.changes };
  }

  all<T extends SqliteRow = SqliteRow>(
    sql: string,
    params: unknown[] = []
  ): T[] {
    const stmt = this.db.prepare(sql);
    return stmt.all(...params) as T[];
  }

  get<T extends SqliteRow = SqliteRow>(
    sql: string,
    params: unknown[] = []
  ): T | undefined {
    const stmt = this.db.prepare(sql);
    return stmt.get(...params) as T | undefined;
  }

  transaction(fn: () => void): void {
    const txFn = this.db.transaction(fn);
    txFn();
  }

  close(): void {
    this.db.close();
  }
}
