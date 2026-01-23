import { useCallback, useEffect, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import {
  useMidenStore,
  useNotesStore,
  useConsumableNotesStore,
} from "../store/MidenStore";
import { NoteFilter, NoteFilterTypes, AccountId } from "@miden-sdk/miden-sdk";
import type { NotesFilter, NotesResult } from "../types";

/**
 * Hook to list notes.
 *
 * @param options - Optional filter options
 *
 * @example
 * ```tsx
 * function NotesList() {
 *   const { notes, consumableNotes, isLoading, refetch } = useNotes();
 *
 *   if (isLoading) return <div>Loading...</div>;
 *
 *   return (
 *     <div>
 *       <h2>All Notes ({notes.length})</h2>
 *       {notes.map(n => (
 *         <div key={n.id().toString()}>
 *           Note: {n.id().toString()} - {n.isConsumed() ? 'Consumed' : 'Pending'}
 *         </div>
 *       ))}
 *
 *       <h2>Consumable Notes ({consumableNotes.length})</h2>
 *       {consumableNotes.map(n => (
 *         <div key={n.inputNoteRecord().id().toString()}>
 *           {n.inputNoteRecord().id().toString()}
 *         </div>
 *       ))}
 *
 *       <button onClick={refetch}>Refresh</button>
 *     </div>
 *   );
 * }
 * ```
 */
export function useNotes(options?: NotesFilter): NotesResult {
  const { client, isReady } = useMiden();
  const notes = useNotesStore();
  const consumableNotes = useConsumableNotesStore();
  const isLoadingNotes = useMidenStore((state) => state.isLoadingNotes);
  const setLoadingNotes = useMidenStore((state) => state.setLoadingNotes);
  const setNotes = useMidenStore((state) => state.setNotes);
  const setConsumableNotes = useMidenStore((state) => state.setConsumableNotes);

  const [error, setError] = useState<Error | null>(null);

  const refetch = useCallback(async () => {
    if (!client || !isReady) return;

    setLoadingNotes(true);
    setError(null);

    try {
      // Get the appropriate filter type
      const filterType = getNoteFilterType(options?.status);
      const filter = new NoteFilter(filterType);

      // Fetch input notes
      const fetchedNotes = await client.getInputNotes(filter);
      setNotes(fetchedNotes);

      // Fetch consumable notes
      let fetchedConsumable;
      if (options?.accountId) {
        const accountIdObj = AccountId.fromHex(options.accountId);
        fetchedConsumable = await client.getConsumableNotes(accountIdObj);
      } else {
        fetchedConsumable = await client.getConsumableNotes();
      }
      setConsumableNotes(fetchedConsumable);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setLoadingNotes(false);
    }
  }, [
    client,
    isReady,
    options?.status,
    options?.accountId,
    setLoadingNotes,
    setNotes,
    setConsumableNotes,
  ]);

  // Initial fetch
  useEffect(() => {
    if (isReady && notes.length === 0) {
      refetch();
    }
  }, [isReady, notes.length, refetch]);

  return {
    notes,
    consumableNotes,
    isLoading: isLoadingNotes,
    error,
    refetch,
  };
}

function getNoteFilterType(status?: NotesFilter["status"]): NoteFilterTypes {
  switch (status) {
    case "consumed":
      return NoteFilterTypes.Consumed;
    case "committed":
      return NoteFilterTypes.Committed;
    case "expected":
      return NoteFilterTypes.Expected;
    case "processing":
      return NoteFilterTypes.Processing;
    case "all":
    default:
      return NoteFilterTypes.All;
  }
}
