import { useCallback, useState } from "react";
import { NoteFile } from "@miden-sdk/miden-sdk";
import { useMiden } from "../context/MidenProvider";

export interface UseImportNoteResult {
  /** Import a note from serialized bytes (e.g. from QR code or dApp request) */
  importNote: (noteBytes: Uint8Array) => Promise<string>;
  /** Whether the import is in progress */
  isImporting: boolean;
  /** Error if import failed */
  error: Error | null;
  /** Reset the hook state */
  reset: () => void;
}

/**
 * Hook to import a note from serialized NoteFile bytes.
 *
 * @example
 * ```tsx
 * function ImportNoteButton({ noteBytes }: { noteBytes: Uint8Array }) {
 *   const { importNote, isImporting, error } = useImportNote();
 *
 *   const handleImport = async () => {
 *     const noteId = await importNote(noteBytes);
 *     console.log('Imported note:', noteId);
 *   };
 *
 *   return (
 *     <button onClick={handleImport} disabled={isImporting}>
 *       {isImporting ? 'Importing...' : 'Import Note'}
 *     </button>
 *   );
 * }
 * ```
 */
export function useImportNote(): UseImportNoteResult {
  const { client, isReady, runExclusive, sync } = useMiden();

  const [isImporting, setIsImporting] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const importNote = useCallback(
    async (noteBytes: Uint8Array): Promise<string> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      setIsImporting(true);
      setError(null);

      try {
        const noteFile = NoteFile.deserialize(noteBytes);
        const noteId = await runExclusive(() =>
          client.importNoteFile(noteFile)
        );

        await sync();

        return noteId.toString();
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
    importNote,
    isImporting,
    error,
    reset,
  };
}
