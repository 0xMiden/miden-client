import { useCallback, useEffect, useMemo, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import {
  useMidenStore,
  useNotesStore,
  useConsumableNotesStore,
  useSyncStateStore,
} from "../store/MidenStore";
import { NoteFilter, NoteFilterTypes } from "@miden-sdk/miden-sdk";
import type { NotesFilter, NotesResult, NoteSummary } from "../types";
import { runExclusiveDirect } from "../utils/runExclusive";
import { getNoteSummary } from "../utils/notes";
import { useAssetMetadata } from "./useAssetMetadata";
import { parseAccountId } from "../utils/accountParsing";

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
  const { client, isReady, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;
  const notes = useNotesStore();
  const consumableNotes = useConsumableNotesStore();
  const isLoadingNotes = useMidenStore((state) => state.isLoadingNotes);
  const setLoadingNotes = useMidenStore((state) => state.setLoadingNotes);
  const setNotes = useMidenStore((state) => state.setNotes);
  const setConsumableNotes = useMidenStore((state) => state.setConsumableNotes);
  const { lastSyncTime } = useSyncStateStore();

  const [error, setError] = useState<Error | null>(null);

  const refetch = useCallback(async () => {
    if (!client || !isReady) return;

    setLoadingNotes(true);
    setError(null);

    try {
      const { fetchedNotes, fetchedConsumable } = await runExclusiveSafe(
        async () => {
          const filterType = getNoteFilterType(options?.status);
          const filter = new NoteFilter(filterType);

          const notesResult = await client.getInputNotes(filter);

          let consumableResult;
          if (options?.accountId) {
            const accountIdObj = parseAccountId(options.accountId);
            consumableResult = await client.getConsumableNotes(accountIdObj);
          } else {
            consumableResult = await client.getConsumableNotes();
          }

          return {
            fetchedNotes: notesResult,
            fetchedConsumable: consumableResult,
          };
        }
      );

      setNotes(fetchedNotes);
      setConsumableNotes(fetchedConsumable);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setLoadingNotes(false);
    }
  }, [
    client,
    isReady,
    runExclusive,
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

  // Refresh after successful syncs to keep notes current
  useEffect(() => {
    if (!isReady || !lastSyncTime) return;
    refetch();
  }, [isReady, lastSyncTime, refetch]);

  const noteAssetIds = useMemo(() => {
    const ids = new Set<string>();
    const collect = (note: unknown) => {
      const summary = getNoteSummary(note as never);
      if (!summary) return;
      summary.assets.forEach((asset) => ids.add(asset.assetId));
    };

    notes.forEach(collect);
    consumableNotes.forEach(collect);

    return Array.from(ids);
  }, [notes, consumableNotes]);

  const { assetMetadata } = useAssetMetadata(noteAssetIds);
  const getMetadata = useCallback(
    (assetId: string) => assetMetadata.get(assetId),
    [assetMetadata]
  );

  const noteSummaries = useMemo(
    () =>
      notes
        .map((note) => getNoteSummary(note, getMetadata))
        .filter(Boolean) as NoteSummary[],
    [notes, getMetadata]
  );

  const consumableNoteSummaries = useMemo(
    () =>
      consumableNotes
        .map((note) => getNoteSummary(note, getMetadata))
        .filter(Boolean) as NoteSummary[],
    [consumableNotes, getMetadata]
  );

  return {
    notes,
    consumableNotes,
    noteSummaries,
    consumableNoteSummaries,
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
