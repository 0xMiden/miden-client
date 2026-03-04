import { useCallback } from "react";
import { useMiden } from "../context/MidenProvider";
import type { ConsumableNoteRecord } from "@miden-sdk/miden-sdk";
import type { WaitForNotesOptions } from "../types";
import { parseAccountId } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";

export interface UseWaitForNotesResult {
  /** Wait until an account has consumable notes */
  waitForConsumableNotes: (
    options: WaitForNotesOptions
  ) => Promise<ConsumableNoteRecord[]>;
}

type ClientWithNotes = {
  syncState: () => Promise<unknown>;
  getConsumableNotes: (accountId?: unknown) => Promise<ConsumableNoteRecord[]>;
};

export function useWaitForNotes(): UseWaitForNotesResult {
  const { client, isReady, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;

  const waitForConsumableNotes = useCallback(
    async (options: WaitForNotesOptions): Promise<ConsumableNoteRecord[]> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      const timeoutMs = Math.max(0, options.timeoutMs ?? 10_000);
      const intervalMs = Math.max(1, options.intervalMs ?? 1_000);
      const minCount = Math.max(1, options.minCount ?? 1);
      const accountId = parseAccountId(options.accountId);

      let waited = 0;

      while (waited < timeoutMs) {
        await runExclusiveSafe(() => (client as ClientWithNotes).syncState());
        const notes = await runExclusiveSafe(() =>
          (client as ClientWithNotes).getConsumableNotes(accountId)
        );
        if (notes.length >= minCount) {
          return notes;
        }

        await new Promise((resolve) => setTimeout(resolve, intervalMs));
        waited += intervalMs;
      }

      throw new Error("Timeout waiting for consumable notes");
    },
    [client, isReady, runExclusiveSafe]
  );

  return { waitForConsumableNotes };
}
