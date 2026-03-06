import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";

export interface UseImportStoreResult {
  /** Import a previously exported store snapshot */
  importStore: (dump: unknown, storeName: string) => Promise<void>;
  /** Whether the import is in progress */
  isImporting: boolean;
  /** Error if import failed */
  error: Error | null;
  /** Reset the hook state */
  reset: () => void;
}

/**
 * Hook to import a previously exported IndexedDB store for restore.
 *
 * @example
 * ```tsx
 * function RestoreButton({ snapshot }: { snapshot: unknown }) {
 *   const { importStore, isImporting, error } = useImportStore();
 *
 *   const handleRestore = async () => {
 *     await importStore(snapshot, 'RestoredStore');
 *     // Store has been restored — sync or reload
 *   };
 *
 *   return (
 *     <button onClick={handleRestore} disabled={isImporting}>
 *       {isImporting ? 'Restoring...' : 'Restore Wallet'}
 *     </button>
 *   );
 * }
 * ```
 */
export function useImportStore(): UseImportStoreResult {
  const { client, isReady, runExclusive, sync } = useMiden();

  const [isImporting, setIsImporting] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const importStore = useCallback(
    async (dump: unknown, storeName: string): Promise<void> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      setIsImporting(true);
      setError(null);

      try {
        await runExclusive(() => client.forceImportStore(dump, storeName));
        await sync();
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      } finally {
        setIsImporting(false);
      }
    },
    [client, isReady, runExclusive, sync]
  );

  const reset = useCallback(() => {
    setIsImporting(false);
    setError(null);
  }, []);

  return {
    importStore,
    isImporting,
    error,
    reset,
  };
}
