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
 * @param {number} [timeoutMs=0] - Optional timeout in milliseconds for
 *   acquiring the lock. 0 (default) means wait indefinitely. When the timeout
 *   fires the lock request is aborted and the returned promise rejects with an
 *   `AbortError`. Has no effect when the Web Locks API is unavailable.
 * @returns {Promise<T>}
 * @template T
 */
export async function withWriteLock(storeName, fn, timeoutMs = 0) {
  if (!hasWebLocks()) {
    return fn();
  }

  const lockName = `miden-db-${storeName || "default"}`;

  if (timeoutMs > 0) {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

    try {
      return await navigator.locks.request(
        lockName,
        { mode: "exclusive", signal: controller.signal },
        async () => {
          clearTimeout(timeoutId);
          return fn();
        }
      );
    } catch (err) {
      clearTimeout(timeoutId);
      throw err;
    }
  }

  return navigator.locks.request(lockName, { mode: "exclusive" }, async () => {
    return fn();
  });
}
