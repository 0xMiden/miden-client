/**
 * Sync Lock Module
 *
 * Provides coordination for concurrent syncState() calls using the Web Locks API
 * with an in-process mutex fallback for older browsers.
 *
 * Behavior:
 * - Uses "coalescing": if a sync is in progress, subsequent callers wait and receive
 *   the same result
 * - Web Locks for cross-tab coordination (Chrome 69+, Safari 15.4+)
 * - In-process mutex fallback when Web Locks unavailable
 * - Optional timeout support
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

/**
 * Internal state for tracking in-progress syncs and waiters per database.
 */
const syncStates = new Map();

/**
 * Get or create sync state for a database.
 */
function getSyncState(dbId) {
  let state = syncStates.get(dbId);
  if (!state) {
    state = {
      inProgress: false,
      result: null,
      error: null,
      waiters: [],
      releaseLock: null,
      syncGeneration: 0,
    };
    syncStates.set(dbId, state);
  }
  return state;
}

/**
 * Acquire a sync lock for the given database.
 *
 * If a sync is already in progress:
 * - Returns { acquired: false, coalescedResult } after waiting for the result
 *
 * If no sync is in progress:
 * - Returns { acquired: true } and the caller should perform the sync,
 *   then call releaseSyncLock() or releaseSyncLockWithError()
 *
 * @param {string} dbId - The database ID to lock
 * @param {number} timeoutMs - Optional timeout in milliseconds (0 = no timeout)
 * @returns {Promise<{acquired: boolean, coalescedResult?: any}>}
 */
export async function acquireSyncLock(dbId, timeoutMs = 0) {
  const state = getSyncState(dbId);

  // If a sync is already in progress, wait for it to complete (coalescing)
  if (state.inProgress) {
    return new Promise((resolve, reject) => {
      let timeoutId;
      if (timeoutMs > 0) {
        timeoutId = setTimeout(() => {
          const idx = state.waiters.findIndex((w) => w.resolve === onResult);
          if (idx !== -1) {
            state.waiters.splice(idx, 1);
          }
          reject(new Error("Sync lock acquisition timed out"));
        }, timeoutMs);
      }

      const onResult = (result) => {
        if (timeoutId) clearTimeout(timeoutId);
        resolve({ acquired: false, coalescedResult: result });
      };

      const onError = (error) => {
        if (timeoutId) clearTimeout(timeoutId);
        reject(error);
      };

      state.waiters.push({ resolve: onResult, reject: onError });
    });
  }

  // Mark sync as in progress and increment generation
  state.inProgress = true;
  state.result = null;
  state.error = null;
  state.syncGeneration++;
  const currentGeneration = state.syncGeneration;

  // Try to acquire Web Lock if available
  if (hasWebLocks()) {
    const lockName = `miden-sync-${dbId}`;

    return new Promise((resolve, reject) => {
      let timeoutId;
      let timedOut = false;

      if (timeoutMs > 0) {
        timeoutId = setTimeout(() => {
          timedOut = true;
          if (state.syncGeneration === currentGeneration) {
            state.inProgress = false;
            const error = new Error("Sync lock acquisition timed out");
            for (const waiter of state.waiters) {
              waiter.reject(error);
            }
            state.waiters = [];
          }
          reject(new Error("Sync lock acquisition timed out"));
        }, timeoutMs);
      }

      navigator.locks
        .request(lockName, { mode: "exclusive" }, async () => {
          if (timedOut || state.syncGeneration !== currentGeneration) {
            return;
          }

          if (timeoutId) clearTimeout(timeoutId);

          return new Promise((releaseLock) => {
            state.releaseLock = releaseLock;
            resolve({ acquired: true });
          });
        })
        .catch((err) => {
          if (timeoutId) clearTimeout(timeoutId);
          if (state.syncGeneration === currentGeneration) {
            state.inProgress = false;
          }
          reject(err instanceof Error ? err : new Error(String(err)));
        });
    });
  } else {
    // Fallback: no Web Locks, just use in-process state
    return { acquired: true };
  }
}

/**
 * Release the sync lock with a successful result.
 *
 * This notifies all waiting callers with the result and releases the lock.
 *
 * @param {string} dbId - The database ID
 * @param {any} result - The sync result to pass to waiters
 */
export function releaseSyncLock(dbId, result) {
  const state = getSyncState(dbId);

  if (!state.inProgress) {
    console.warn("releaseSyncLock called but no sync was in progress");
    return;
  }

  state.result = result;
  state.inProgress = false;

  for (const waiter of state.waiters) {
    waiter.resolve(result);
  }
  state.waiters = [];

  if (state.releaseLock) {
    state.releaseLock();
    state.releaseLock = null;
  }
}

/**
 * Release the sync lock due to an error.
 *
 * This notifies all waiting callers that the sync failed.
 *
 * @param {string} dbId - The database ID
 * @param {Error} error - The error to pass to waiters
 */
export function releaseSyncLockWithError(dbId, error) {
  const state = getSyncState(dbId);

  if (!state.inProgress) {
    console.warn("releaseSyncLockWithError called but no sync was in progress");
    return;
  }

  state.error = error;
  state.inProgress = false;

  for (const waiter of state.waiters) {
    waiter.reject(error);
  }
  state.waiters = [];

  if (state.releaseLock) {
    state.releaseLock();
    state.releaseLock = null;
  }
}
