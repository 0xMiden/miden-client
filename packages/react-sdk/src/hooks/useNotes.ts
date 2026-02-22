import { useCallback, useEffect, useMemo, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import {
  useMidenStore,
  useNotesStore,
  useConsumableNotesStore,
  useSyncStateStore,
} from "../store/MidenStore";
import { NoteFilter } from "@miden-sdk/miden-sdk";
import type { NotesFilter, NotesResult, NoteSummary } from "../types";
import { runExclusiveDirect } from "../utils/runExclusive";
import { getNoteSummary } from "../utils/notes";
import { useAssetMetadata } from "./useAssetMetadata";
import { parseAccountId } from "../utils/accountParsing";
import { normalizeAccountId } from "../utils/accountId";
import { getNoteFilterType } from "../utils/noteFilters";

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
  const setNotesIfChanged = useMidenStore((state) => state.setNotesIfChanged);
  const setConsumableNotesIfChanged = useMidenStore(
    (state) => state.setConsumableNotesIfChanged
  );
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

      // Smart refetch: only update store if note IDs changed (prevents unnecessary re-renders)
      setNotesIfChanged(fetchedNotes);
      setConsumableNotesIfChanged(fetchedConsumable);
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
    setNotesIfChanged,
    setConsumableNotesIfChanged,
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

  // Normalize sender once outside the loop to avoid per-note WASM allocations
  const normalizedSender = useMemo(() => {
    if (!options?.sender) return null;
    try {
      return normalizeAccountId(options.sender);
    } catch {
      return options.sender;
    }
  }, [options?.sender]);

  // Serialize excludeIds to a stable string key so array literals don't defeat memoization
  const excludeIdsKey = useMemo(() => {
    if (!options?.excludeIds || options.excludeIds.length === 0) return "";
    return [...options.excludeIds].sort().join("\0");
  }, [options?.excludeIds]);

  // Helper: normalize a sender string with a cache to avoid repeated WASM allocations.
  // normalizeAccountId calls parseAccountId (WASM) + toBech32 per invocation.
  const filterBySender = useCallback(
    (summaries: NoteSummary[], target: string): NoteSummary[] => {
      const cache = new Map<string, string>();
      return summaries.filter((s) => {
        if (!s.sender) return false;
        let normalized = cache.get(s.sender);
        if (normalized === undefined) {
          try {
            normalized = normalizeAccountId(s.sender);
          } catch {
            normalized = s.sender;
          }
          cache.set(s.sender, normalized);
        }
        return normalized === target;
      });
    },
    []
  );

  // Build summaries with optional sender and excludeIds filters
  const noteSummaries = useMemo(() => {
    let summaries = notes
      .map((note) => getNoteSummary(note, getMetadata))
      .filter(Boolean) as NoteSummary[];

    if (normalizedSender) {
      summaries = filterBySender(summaries, normalizedSender);
    }

    if (excludeIdsKey) {
      const excludeSet = new Set(excludeIdsKey.split("\0"));
      summaries = summaries.filter((s) => !excludeSet.has(s.id));
    }

    return summaries;
  }, [notes, getMetadata, normalizedSender, excludeIdsKey, filterBySender]);

  const consumableNoteSummaries = useMemo(() => {
    let summaries = consumableNotes
      .map((note) => getNoteSummary(note, getMetadata))
      .filter(Boolean) as NoteSummary[];

    if (normalizedSender) {
      summaries = filterBySender(summaries, normalizedSender);
    }

    if (excludeIdsKey) {
      const excludeSet = new Set(excludeIdsKey.split("\0"));
      summaries = summaries.filter((s) => !excludeSet.has(s.id));
    }

    return summaries;
  }, [
    consumableNotes,
    getMetadata,
    normalizedSender,
    excludeIdsKey,
    filterBySender,
  ]);

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
