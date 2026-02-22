export interface MigrateStorageOptions {
  /** Current version string to compare against stored version */
  version: string;
  /** localStorage key for version tracking. Default: "miden:storageVersion" */
  versionKey?: string;
  /** Callback before clearing storage (e.g., to save data) */
  onBeforeClear?: () => void | Promise<void>;
  /** Reload page after clearing. Default: true */
  reloadOnClear?: boolean;
}

/**
 * Check if stored version matches current version. If not, clear Miden IndexedDB
 * databases and localStorage, then optionally reload.
 * Returns true if migration was triggered.
 */
export async function migrateStorage(
  options: MigrateStorageOptions
): Promise<boolean> {
  if (typeof window === "undefined") return false;

  const versionKey = options.versionKey ?? "miden:storageVersion";
  const stored = localStorage.getItem(versionKey);

  if (stored === options.version) return false;

  if (options.onBeforeClear) {
    await options.onBeforeClear();
  }

  await clearMidenStorage();
  localStorage.setItem(versionKey, options.version);

  if (options.reloadOnClear !== false) {
    window.location.reload();
  }

  return true;
}

/**
 * Clear all Miden-related IndexedDB databases.
 */
export async function clearMidenStorage(): Promise<void> {
  if (typeof indexedDB === "undefined") return;

  try {
    const databases = await indexedDB.databases();
    const midenDbs = databases.filter((db) =>
      db.name?.toLowerCase().includes("miden")
    );
    await Promise.all(
      midenDbs.map(
        (db) =>
          new Promise<void>((resolve, reject) => {
            if (!db.name) {
              resolve();
              return;
            }
            const req = indexedDB.deleteDatabase(db.name);
            req.onsuccess = () => resolve();
            req.onerror = () => reject(req.error);
            req.onblocked = () => resolve();
          })
      )
    );
  } catch {
    // indexedDB.databases() may not be available in all browsers
  }
}

/**
 * Create a namespaced localStorage helper for app state persistence.
 */
export function createMidenStorage(prefix: string) {
  const fullKey = (key: string) => `${prefix}:${key}`;

  return {
    get<T>(key: string): T | null {
      try {
        const raw = localStorage.getItem(fullKey(key));
        if (raw === null) return null;
        return JSON.parse(raw) as T;
      } catch {
        return null;
      }
    },

    set<T>(key: string, value: T): void {
      try {
        localStorage.setItem(fullKey(key), JSON.stringify(value));
      } catch {
        // localStorage may be full or unavailable
      }
    },

    remove(key: string): void {
      localStorage.removeItem(fullKey(key));
    },

    clear(): void {
      const keysToRemove: string[] = [];
      for (let i = 0; i < localStorage.length; i++) {
        const k = localStorage.key(i);
        if (k?.startsWith(`${prefix}:`)) {
          keysToRemove.push(k);
        }
      }
      keysToRemove.forEach((k) => localStorage.removeItem(k));
    },
  };
}
