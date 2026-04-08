export const runExclusiveDirect = async <T>(fn: () => Promise<T>): Promise<T> =>
  fn();
