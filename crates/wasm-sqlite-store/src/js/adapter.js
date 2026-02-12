/**
 * Unified SQLite adapter interface.
 *
 * This interface abstracts over different SQLite backends so that the Rust/WASM
 * code doesn't care whether it's running in Node.js (better-sqlite3) or a
 * browser (wa-sqlite + OPFS/IDB VFS).
 */
/**
 * Creates the appropriate SQLite adapter for the current environment.
 * In Node.js, uses better-sqlite3. In browsers, uses wa-sqlite.
 */
export async function createAdapter(dbName) {
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
