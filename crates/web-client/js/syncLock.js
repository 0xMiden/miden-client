/**
 * Sync Lock Module
 *
 * Coordinates concurrent sync calls using the Web Locks API.
 *
 * Behavior:
 * - Same-method coalescing: if a sync of the same method is in progress,
 *   subsequent callers share its result promise
 * - Different-method serialization: different methods (e.g. syncState vs
 *   syncNoteTransport) wait for each other via the Web Lock (or the
 *   WASM-level mutex when Web Locks are unavailable)
 * - Web Locks also serialize across tabs (Chrome 69+, Safari 15.4+)
 */

/**
 * Check if the Web Locks API is available.
 */
export function hasWebLocks() {
  return (
    typeof navigator !== "undefined" &&
    navigator.locks !== undefined &&
    typeof navigator.locks.request === "function"
  );
}

// Coalesce map keyed by `${dbId}:${methodId}` -> in-flight promise.
const inFlight = new Map();

/**
 * Build the coalesce-map key for an in-flight sync of `(dbId, methodId)`.
 *
 * @param {string} dbId
 * @param {string} methodId
 * @returns {string}
 */
function coalesceKey(dbId, methodId) {
  return `${dbId}:${methodId}`;
}

/**
 * Run `fn` while holding the per-db Web Lock. When Web Locks are unavailable,
 * runs `fn` directly and relies on the WASM-level mutex (`get_mut_inner`) to
 * serialize across methods within the tab.
 *
 * @param {string} dbId
 * @param {() => Promise<T>} fn
 * @returns {Promise<T>}
 * @template T
 */
function runUnderLock(dbId, fn) {
  if (!hasWebLocks()) {
    // No Web Locks: rely on the WASM-level mutex (get_mut_inner) to serialize
    // across methods within the tab.
    return Promise.resolve().then(fn);
  }
  return navigator.locks.request(
    `miden-sync-${dbId}`,
    { mode: "exclusive" },
    fn
  );
}

/**
 * Run `fn` under the sync lock for (dbId, methodId).
 *
 * Concurrent calls with the same (dbId, methodId) share the same promise
 * (coalescing). Concurrent calls on the same dbId with different methodIds
 * serialize via the Web Lock.
 *
 * @param {string} dbId - Database ID
 * @param {string} methodId - Method identifier (see MethodName constants)
 * @param {() => Promise<T>} fn - Work to run under the lock
 * @returns {Promise<T>}
 */
export function withSyncLock(dbId, methodId, fn) {
  const key = coalesceKey(dbId, methodId);

  let work = inFlight.get(key);
  if (!work) {
    work = runUnderLock(dbId, fn);
    inFlight.set(key, work);
    // Swallow on the derived promise so a rejection here doesn't surface as
    // an unhandled rejection; the caller still sees the error through `work`.
    work
      .finally(() => {
        if (inFlight.get(key) === work) inFlight.delete(key);
      })
      .catch(() => {});
  }

  return work;
}
