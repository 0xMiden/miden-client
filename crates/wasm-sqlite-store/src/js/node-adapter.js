/**
 * Node.js SQLite adapter using better-sqlite3.
 *
 * better-sqlite3 provides a synchronous API which is the fastest SQLite
 * option for Node.js. It's a native addon that links against the system
 * or bundled SQLite.
 */
export class NodeSqliteAdapter {
  db;
  constructor(dbPath) {
    // Dynamic require for better-sqlite3 since it's a native addon
    const Database = require("better-sqlite3");
    this.db = new Database(dbPath);
    // Enable WAL mode for better concurrent read performance
    this.db.pragma("journal_mode = WAL");
    // Enable foreign keys
    this.db.pragma("foreign_keys = ON");
  }
  run(sql, params = []) {
    const stmt = this.db.prepare(sql);
    const result = stmt.run(...params);
    return { changes: result.changes };
  }
  all(sql, params = []) {
    const stmt = this.db.prepare(sql);
    return stmt.all(...params);
  }
  get(sql, params = []) {
    const stmt = this.db.prepare(sql);
    return stmt.get(...params);
  }
  transaction(fn) {
    const txFn = this.db.transaction(fn);
    txFn();
  }
  close() {
    this.db.close();
  }
}
