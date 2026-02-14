export class AsyncLock {
  private pending: Promise<void> = Promise.resolve();

  runExclusive<T>(fn: () => Promise<T>): Promise<T> {
    const run = this.pending.then(fn, fn);
    this.pending = run.then(
      () => undefined,
      () => undefined
    );
    return run;
  }
}
