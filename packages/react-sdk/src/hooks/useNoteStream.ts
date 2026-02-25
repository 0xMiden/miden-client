import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import {
  useMidenStore,
  useNotesStore,
  useSyncStateStore,
  useNoteFirstSeenStore,
} from "../store/MidenStore";
import { NoteFilter } from "@miden-sdk/miden-sdk";
import type { InputNoteRecord } from "@miden-sdk/miden-sdk";
import type {
  StreamedNote,
  UseNoteStreamOptions,
  UseNoteStreamReturn,
  NoteAsset,
} from "../types";
import { readNoteAttachment } from "../utils/noteAttachment";
import { normalizeAccountId } from "../utils/accountId";
import { toBech32AccountId } from "../utils/accountBech32";
import { getNoteFilterType } from "../utils/noteFilters";

/**
 * Hook for temporal note tracking with a unified model.
 *
 * Replaces the common pattern of `handledNoteIds` refs, deferred baselines,
 * and dual-track note decoding. Returns `StreamedNote` objects that merge
 * summary data with the underlying record and pre-decode attachments.
 *
 * @example
 * ```tsx
 * function IncomingNotes({ opponentId }: { opponentId: string }) {
 *   const { notes, latest, markHandled, snapshot } = useNoteStream({
 *     sender: opponentId,
 *     status: "committed",
 *   });
 *
 *   useEffect(() => {
 *     if (latest) {
 *       console.log("New note!", latest.attachment);
 *       markHandled(latest.id);
 *     }
 *   }, [latest, markHandled]);
 *
 *   return <div>{notes.length} unhandled notes</div>;
 * }
 * ```
 */
export function useNoteStream(
  options: UseNoteStreamOptions = {}
): UseNoteStreamReturn {
  const { client, isReady } = useMiden();

  const allNotes = useNotesStore();
  const noteFirstSeen = useNoteFirstSeenStore();
  const { lastSyncTime } = useSyncStateStore();
  const setNotesIfChanged = useMidenStore((state) => state.setNotesIfChanged);

  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const handledIdsRef = useRef<Set<string>>(new Set());
  const [handledVersion, setHandledVersion] = useState(0);

  // Resolve options
  const status = options.status ?? "committed";
  const sender = options.sender ?? null;
  const since = options.since;

  // Store amountFilter in a ref so the streamedNotes useMemo doesn't depend
  // on the function reference (callers typically pass inline lambdas).
  const amountFilterRef = useRef(options.amountFilter);
  amountFilterRef.current = options.amountFilter;

  // Serialize excludeIds to a stable key so array literals don't defeat memoization.
  // Uses \0 separator (note IDs are hex strings that never contain null bytes).
  const excludeIdsKey = useMemo(() => {
    if (!options.excludeIds) return "";
    if (options.excludeIds instanceof Set)
      return Array.from(options.excludeIds).sort().join("\0");
    return [...options.excludeIds].sort().join("\0");
  }, [options.excludeIds]);

  const excludeIdSet = useMemo(() => {
    if (!excludeIdsKey) return null;
    return new Set(excludeIdsKey.split("\0"));
  }, [excludeIdsKey]);

  // Fetch notes from client
  const refetch = useCallback(async () => {
    if (!client || !isReady) return;

    setIsLoading(true);
    setError(null);

    try {
      const filterType = getNoteFilterType(status);
      const filter = new NoteFilter(filterType);
      const fetched = await client.getInputNotes(filter);
      setNotesIfChanged(fetched);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setIsLoading(false);
    }
  }, [client, isReady, status, setNotesIfChanged]);

  // Fetch on mount and after each sync
  useEffect(() => {
    if (isReady) {
      refetch();
    }
  }, [isReady, lastSyncTime, refetch]);

  // Build StreamedNote array with all filters applied
  const streamedNotes = useMemo((): StreamedNote[] => {
    // Force recalculation when handled IDs change
    void handledVersion;

    const result: StreamedNote[] = [];

    // Normalize sender once outside the loop to avoid creating WASM objects per note
    let normalizedSender: string | null = null;
    if (sender) {
      try {
        normalizedSender = normalizeAccountId(sender);
      } catch {
        normalizedSender = sender;
      }
    }

    for (const record of allNotes) {
      const note = buildStreamedNote(record, noteFirstSeen);
      if (!note) continue;

      // Filter: handled IDs
      if (handledIdsRef.current.has(note.id)) continue;

      // Filter: exclude IDs
      if (excludeIdSet && excludeIdSet.has(note.id)) continue;

      // Filter: sender (compare normalized strings directly)
      if (normalizedSender && note.sender !== normalizedSender) continue;

      // Filter: since timestamp
      if (since !== undefined && note.firstSeenAt < since) continue;

      // Filter: amount (read from ref to avoid unstable function dep)
      if (amountFilterRef.current && !amountFilterRef.current(note.amount))
        continue;

      result.push(note);
    }

    // Sort by firstSeenAt ascending (oldest first)
    result.sort((a, b) => a.firstSeenAt - b.firstSeenAt);

    return result;
  }, [allNotes, noteFirstSeen, excludeIdSet, sender, since, handledVersion]);

  const latest = useMemo(
    () =>
      streamedNotes.length > 0 ? streamedNotes[streamedNotes.length - 1] : null,
    [streamedNotes]
  );

  const markHandled = useCallback((noteId: string) => {
    handledIdsRef.current = new Set(handledIdsRef.current).add(noteId);
    setHandledVersion((v) => v + 1);
  }, []);

  const markAllHandled = useCallback(() => {
    const newSet = new Set(handledIdsRef.current);
    for (const note of streamedNotes) {
      newSet.add(note.id);
    }
    handledIdsRef.current = newSet;
    setHandledVersion((v) => v + 1);
  }, [streamedNotes]);

  const snapshot = useCallback(() => {
    const ids = new Set<string>();
    for (const note of streamedNotes) {
      ids.add(note.id);
    }
    return { ids, timestamp: Date.now() };
  }, [streamedNotes]);

  return {
    notes: streamedNotes,
    latest,
    markHandled,
    markAllHandled,
    snapshot,
    isLoading,
    error,
  };
}

function buildStreamedNote(
  record: InputNoteRecord,
  noteFirstSeen: Map<string, number>
): StreamedNote | null {
  try {
    const id = record.id().toString();

    // Extract sender
    const metadata = record.metadata?.();
    const senderHex = metadata?.sender?.()?.toString?.();
    const sender = senderHex ? toBech32AccountId(senderHex) : "";

    // Extract assets
    const assets: NoteAsset[] = [];
    let primaryAmount = 0n;
    try {
      const details = record.details();
      const assetsList = details?.assets?.().fungibleAssets?.() ?? [];
      for (const asset of assetsList) {
        const assetId = asset.faucetId().toString();
        const amount = BigInt(asset.amount() as number | bigint);
        assets.push({ assetId, amount });
        if (primaryAmount === 0n) {
          primaryAmount = amount;
        }
      }
    } catch {
      // Keep assets empty
    }

    // Decode attachment
    const attachmentData = readNoteAttachment(record);
    const attachment = attachmentData ? attachmentData.values : null;

    // First seen timestamp
    const firstSeenAt = noteFirstSeen.get(id) ?? Date.now();

    return {
      id,
      sender,
      amount: primaryAmount,
      assets,
      record,
      firstSeenAt,
      attachment,
    };
  } catch {
    return null;
  }
}
