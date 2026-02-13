/**
 * Cross-Tab Write Lock Module
 *
 * Provides an exclusive write lock using the Web Locks API so that mutating
 * operations on the same IndexedDB database are serialized across browser tabs.
 *
 * When the Web Locks API is unavailable the lock is a no-op — the in-process
 * AsyncLock still protects against concurrent WASM access within a single tab.
 */

import { hasWebLocks } from "./syncLock.js";

/**
 * Execute `fn` while holding an exclusive cross-tab write lock for the given
 * store.  If the Web Locks API is not available, `fn` runs immediately.
 *
 * @param {string} storeName - Logical database / store name.
 * @param {() => Promise<T>} fn - The async work to perform under the lock.
 * @returns {Promise<T>}
 * @template T
 */
export async function withWriteLock(storeName, fn) {
  if (!hasWebLocks()) {
    return fn();
  }

  const lockName = `miden-db-${storeName || "default"}`;

  return navigator.locks.request(lockName, { mode: "exclusive" }, async () => {
    return fn();
  });
}
