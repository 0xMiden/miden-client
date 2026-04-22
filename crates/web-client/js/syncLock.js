/**
 * Sync Lock Module
 *
 * Coordinates concurrent sync calls using the Web Locks API with an in-process
 * mutex fallback for older browsers.
 *
 * Behavior:
 * - Same-method coalescing: if a sync of the same method is in progress,
 *   subsequent callers wait and receive the same result
 * - Different-method serialization: different methods (e.g. syncState vs
 *   syncNoteTransport) do not coalesce; the later call waits for the earlier
 *   to finish, then runs on its own
 * - Web Locks for cross-tab coordination (Chrome 69+, Safari 15.4+)
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
 * Per-(dbId, methodId) coalesce state. Each method tracks its own in-flight
 * call so same-method callers share results while different-method callers
 * serialize.
 */
const syncStates = new Map();

function coalesceKey(dbId, methodId) {
  return `${dbId}:${methodId}`;
}

function getSyncState(dbId, methodId) {
  const key = coalesceKey(dbId, methodId);
  let state = syncStates.get(key);
  if (!state) {
    state = {
      inProgress: false,
      result: null,
      error: null,
      waiters: [],
      releaseLock: null,
      syncGeneration: 0,
    };
    syncStates.set(key, state);
  }
  return state;
}

/**
 * Acquire a sync lock for (dbId, methodId).
 *
 * If a sync of the same method is already in progress, the caller waits and
 * receives the same result (coalescing). If a sync of a different method is
 * in progress, the caller waits for it to release, then acquires the lock
 * and runs its own work.
 *
 * @param {string} dbId - Database ID (scopes the lock across methods and tabs)
 * @param {string} methodId - Method identifier (see MethodName constants)
 * @param {number} timeoutMs - Optional timeout in milliseconds (0 = no timeout)
 * @returns {Promise<{acquired: boolean, coalescedResult?: any}>}
 */
export async function acquireSyncLock(dbId, methodId, timeoutMs = 0) {
  const state = getSyncState(dbId, methodId);

  // Same-method coalescing: a sync of this method is already running, so wait
  // for its result instead of starting another one.
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

  // Mark this method as in progress and increment generation
  state.inProgress = true;
  state.result = null;
  state.error = null;
  state.syncGeneration++;
  const currentGeneration = state.syncGeneration;

  // Try to acquire Web Lock if available. The Web Lock is keyed by `dbId`
  // (not the method) so it serializes across methods within a tab and across
  // tabs.
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
    // Fallback: no Web Locks. The WASM-level mutex inside the Client already
    // serializes across methods within a tab, so we don't need an additional
    // in-process JS mutex — `inProgress` just gates same-method coalescing.
    return { acquired: true };
  }
}

/**
 * Release the sync lock with a successful result.
 *
 * Notifies all same-method waiters with the result and releases the Web Lock.
 *
 * @param {string} dbId - Database ID
 * @param {string} methodId - Method identifier (must match the acquire call)
 * @param {any} result - Sync result to pass to waiters
 */
export function releaseSyncLock(dbId, methodId, result) {
  const state = getSyncState(dbId, methodId);

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
 * Notifies all same-method waiters that the sync failed.
 *
 * @param {string} dbId - Database ID
 * @param {string} methodId - Method identifier (must match the acquire call)
 * @param {Error} error - Error to pass to waiters
 */
export function releaseSyncLockWithError(dbId, methodId, error) {
  const state = getSyncState(dbId, methodId);

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
