// Shared test utilities for vitest + fake-indexeddb tests.

let dbCounter = 0;

/** Returns a unique database name to avoid collisions between tests. */
export function uniqueDbName(): string {
  return `test-miden-${++dbCounter}-${Date.now()}`;
}
