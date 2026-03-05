import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";

export interface UseExportStoreResult {
  /** Export the IndexedDB store as a serializable snapshot */
  exportStore: () => Promise<unknown>;
  /** Whether the export is in progress */
  isExporting: boolean;
  /** Error if export failed */
  error: Error | null;
  /** Reset the hook state */
  reset: () => void;
}

/**
 * Hook to export the IndexedDB store for backup/restore.
 *
 * @example
 * ```tsx
 * function BackupButton() {
 *   const { exportStore, isExporting, error } = useExportStore();
 *
 *   const handleBackup = async () => {
 *     const snapshot = await exportStore();
 *     // Save snapshot to file, encrypted storage, etc.
 *   };
 *
 *   return (
 *     <button onClick={handleBackup} disabled={isExporting}>
 *       {isExporting ? 'Exporting...' : 'Backup Wallet'}
 *     </button>
 *   );
 * }
 * ```
 */
export function useExportStore(): UseExportStoreResult {
  const { client, isReady, runExclusive } = useMiden();

  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const exportStore = useCallback(async (): Promise<unknown> => {
    if (!client || !isReady) {
      throw new Error("Miden client is not ready");
    }

    setIsExporting(true);
    setError(null);

    try {
      const snapshot = await runExclusive(() => client.exportStore());
      return snapshot;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      throw error;
    } finally {
      setIsExporting(false);
    }
  }, [client, isReady, runExclusive]);

  const reset = useCallback(() => {
    setIsExporting(false);
    setError(null);
  }, []);

  return {
    exportStore,
    isExporting,
    error,
    reset,
  };
}
