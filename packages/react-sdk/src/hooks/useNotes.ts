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
import { accountIdsEqual } from "../utils/accountId";
import { getNoteFilterType } from "../utils/noteFilters";

export function useNotes(options?: NotesFilter): NotesResult {
  const { client, isReady, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;
  const notes = useNotesStore();
  const consumableNotes = useConsumableNotesStore();
  const isLoadingNotes = useMidenStore((state) => state.isLoadingNotes);
  const setLoadingNotes = useMidenStore((state) => state.setLoadingNotes);
  const setNotesIfChanged = useMidenStore(
    (state) => state.setNotesIfChanged
  );
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

  // Build summaries with optional sender and excludeIds filters
  const noteSummaries = useMemo(() => {
    let summaries = notes
      .map((note) => getNoteSummary(note, getMetadata))
      .filter(Boolean) as NoteSummary[];

    if (options?.sender) {
      const senderFilter = options.sender;
      summaries = summaries.filter(
        (s) => s.sender && accountIdsEqual(s.sender, senderFilter)
      );
    }

    if (options?.excludeIds && options.excludeIds.length > 0) {
      const excludeSet = new Set(options.excludeIds);
      summaries = summaries.filter((s) => !excludeSet.has(s.id));
    }

    return summaries;
  }, [notes, getMetadata, options?.sender, options?.excludeIds]);

  const consumableNoteSummaries = useMemo(() => {
    let summaries = consumableNotes
      .map((note) => getNoteSummary(note, getMetadata))
      .filter(Boolean) as NoteSummary[];

    if (options?.sender) {
      const senderFilter = options.sender;
      summaries = summaries.filter(
        (s) => s.sender && accountIdsEqual(s.sender, senderFilter)
      );
    }

    if (options?.excludeIds && options.excludeIds.length > 0) {
      const excludeSet = new Set(options.excludeIds);
      summaries = summaries.filter((s) => !excludeSet.has(s.id));
    }

    return summaries;
  }, [consumableNotes, getMetadata, options?.sender, options?.excludeIds]);

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

