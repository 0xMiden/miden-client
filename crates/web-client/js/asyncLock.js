/**
 * A simple promise-chain mutex for serializing async operations.
 *
 * Error-resilient: if one operation throws, the next still runs (no deadlocks).
 */
export class AsyncLock {
  constructor() {
    this._pending = Promise.resolve();
  }

  /**
   * Queue `fn` so that it runs only after all previously queued operations
   * have settled (resolved or rejected).
   *
   * @template T
   * @param {() => Promise<T>} fn
   * @returns {Promise<T>}
   */
  runExclusive(fn) {
    const run = this._pending.then(fn, fn);
    // Swallow the result/error so the chain itself never rejects
    this._pending = run.then(
      () => undefined,
      () => undefined
    );
    return run;
  }
}
