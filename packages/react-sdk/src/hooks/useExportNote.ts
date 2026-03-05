import { useCallback, useState } from "react";
import { NoteExportFormat } from "@miden-sdk/miden-sdk";
import { useMiden } from "../context/MidenProvider";

export interface UseExportNoteResult {
  /** Export a note as serialized bytes */
  exportNote: (noteId: string) => Promise<Uint8Array>;
  /** Whether the export is in progress */
  isExporting: boolean;
  /** Error if export failed */
  error: Error | null;
  /** Reset the hook state */
  reset: () => void;
}

/**
 * Hook to export a note as serialized NoteFile bytes.
 *
 * @example
 * ```tsx
 * function ExportNoteButton({ noteId }: { noteId: string }) {
 *   const { exportNote, isExporting, error } = useExportNote();
 *
 *   const handleExport = async () => {
 *     const bytes = await exportNote(noteId);
 *     // Share bytes via QR code, file download, etc.
 *   };
 *
 *   return (
 *     <button onClick={handleExport} disabled={isExporting}>
 *       {isExporting ? 'Exporting...' : 'Export Note'}
 *     </button>
 *   );
 * }
 * ```
 */
export function useExportNote(): UseExportNoteResult {
  const { client, isReady, runExclusive } = useMiden();

  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const exportNote = useCallback(
    async (noteId: string): Promise<Uint8Array> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      setIsExporting(true);
      setError(null);

      try {
        const noteFile = await runExclusive(() =>
          client.exportNoteFile(noteId, NoteExportFormat.Full)
        );
        return noteFile.serialize();
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      } finally {
        setIsExporting(false);
      }
    },
    [client, isReady, runExclusive]
  );

  const reset = useCallback(() => {
    setIsExporting(false);
    setError(null);
  }, []);

  return {
    exportNote,
    isExporting,
    error,
    reset,
  };
}
