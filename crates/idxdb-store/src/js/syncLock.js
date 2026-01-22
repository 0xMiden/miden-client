/**
 * Sync Lock Module
 *
 * Provides coordination for concurrent sync_state() calls using the Web Locks API
 * with an in-process mutex fallback for older browsers.
 *
 * Behavior:
 * - Uses "coalescing": if a sync is in progress, subsequent callers wait and receive
 *   the same result
 * - Web Locks for cross-tab coordination (Chrome 69+, Safari 15.4+)
 * - In-process mutex fallback when Web Locks unavailable
 * - Optional timeout support
 */
// Registry of sync states per database
const syncStates = new Map();
/**
 * Check if the Web Locks API is available.
 */
export function hasWebLocks() {
    return (typeof navigator !== "undefined" &&
        navigator.locks !== undefined &&
        typeof navigator.locks.request === "function");
}
/**
 * Get or create sync state for a database.
 */
function getSyncState(dbId) {
    let state = syncStates.get(dbId);
    if (!state) {
        state = {
            inProgress: false,
            errored: false,
            waiters: [],
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
 * - Returns { acquired: false } and the caller should wait for the coalesced result
 *   via the returned promise
 *
 * If no sync is in progress:
 * - Returns { acquired: true } and the caller should perform the sync, then call
 *   releaseSyncLock() or releaseSyncLockWithError()
 *
 * @param dbId - The database ID to lock
 * @param timeoutMs - Optional timeout in milliseconds (0 = no timeout)
 * @returns Promise resolving to a SyncLockHandle
 */
export async function acquireSyncLock(dbId, timeoutMs) {
    const state = getSyncState(dbId);
    // If a sync is already in progress, wait for it to complete (coalescing)
    if (state.inProgress) {
        return new Promise((resolve, reject) => {
            // Set up timeout if specified
            let timeoutId;
            if (timeoutMs > 0) {
                timeoutId = setTimeout(() => {
                    // Remove this waiter from the list
                    const idx = state.waiters.findIndex((w) => w.resolve === onResult);
                    if (idx !== -1) {
                        state.waiters.splice(idx, 1);
                    }
                    reject(new Error("Sync lock acquisition timed out"));
                }, timeoutMs);
            }
            const onResult = (result) => {
                if (timeoutId)
                    clearTimeout(timeoutId);
                resolve({ acquired: false, coalescedResult: result });
            };
            const onError = (error) => {
                if (timeoutId)
                    clearTimeout(timeoutId);
                reject(error);
            };
            state.waiters.push({ resolve: onResult, reject: onError });
        });
    }
    // Mark sync as in progress and increment generation
    state.inProgress = true;
    state.result = undefined;
    state.errored = false;
    state.syncGeneration++;
    const currentGeneration = state.syncGeneration;
    // Try to acquire Web Lock if available
    if (hasWebLocks()) {
        const lockName = `miden-sync-${dbId}`;
        return new Promise((resolve, reject) => {
            // Set up timeout if specified
            let timeoutId;
            let timedOut = false;
            if (timeoutMs > 0) {
                timeoutId = setTimeout(() => {
                    timedOut = true;
                    // Only clean up state if we're still the current sync operation
                    if (state.syncGeneration === currentGeneration) {
                        state.inProgress = false;
                        // Reject all waiters since this sync operation is being cancelled
                        const error = new Error("Sync lock acquisition timed out");
                        for (const waiter of state.waiters) {
                            waiter.reject(error);
                        }
                        state.waiters = [];
                    }
                    reject(new Error("Sync lock acquisition timed out"));
                }, timeoutMs);
            }
            // Request the Web Lock
            navigator.locks
                .request(lockName, { mode: "exclusive" }, async () => {
                // Check if timed out or if a newer sync operation has started
                if (timedOut || state.syncGeneration !== currentGeneration) {
                    // Stale lock acquisition, just return to release the Web Lock
                    return;
                }
                if (timeoutId)
                    clearTimeout(timeoutId);
                // Create a promise that will be resolved when releaseSyncLock is called
                return new Promise((releaseLock) => {
                    state.releaseLock = releaseLock;
                    resolve({ acquired: true });
                });
            })
                .catch((err) => {
                if (timeoutId)
                    clearTimeout(timeoutId);
                // Only clean up state if we're still the current sync operation
                if (state.syncGeneration === currentGeneration) {
                    state.inProgress = false;
                }
                reject(err instanceof Error ? err : new Error(String(err)));
            });
        });
    }
    else {
        // Fallback: no Web Locks, just use in-process state
        // The lock is already "acquired" via the inProgress flag
        return { acquired: true };
    }
}
/**
 * Release the sync lock with a successful result.
 *
 * This notifies all waiting callers with the result and releases the lock.
 *
 * @param dbId - The database ID
 * @param result - The serialized sync result
 */
export function releaseSyncLock(dbId, result) {
    const state = getSyncState(dbId);
    if (!state.inProgress) {
        console.warn("releaseSyncLock called but no sync was in progress");
        return;
    }
    state.result = result;
    state.inProgress = false;
    // Notify all waiters
    for (const waiter of state.waiters) {
        waiter.resolve(result);
    }
    state.waiters = [];
    // Release the Web Lock if we have one
    if (state.releaseLock) {
        state.releaseLock();
        state.releaseLock = undefined;
    }
}
/**
 * Release the sync lock due to an error.
 *
 * This notifies all waiting callers that the sync failed.
 *
 * @param dbId - The database ID
 */
export function releaseSyncLockWithError(dbId) {
    const state = getSyncState(dbId);
    if (!state.inProgress) {
        console.warn("releaseSyncLockWithError called but no sync was in progress");
        return;
    }
    state.errored = true;
    state.inProgress = false;
    // Notify all waiters of the error
    const error = new Error("Sync operation failed");
    for (const waiter of state.waiters) {
        waiter.reject(error);
    }
    state.waiters = [];
    // Release the Web Lock if we have one
    if (state.releaseLock) {
        state.releaseLock();
        state.releaseLock = undefined;
    }
}
