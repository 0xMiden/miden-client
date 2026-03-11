import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import { NoteFilter, NoteFilterTypes, NoteId } from "@miden-sdk/miden-sdk";
import type { Note, InputNoteRecord } from "@miden-sdk/miden-sdk";
import type {
  ConsumeOptions,
  TransactionStage,
  TransactionResult,
} from "../types";
import { parseAccountId } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";

export interface UseConsumeResult {
  /** Consume one or more notes */
  consume: (options: ConsumeOptions) => Promise<TransactionResult>;
  /** The transaction result */
  result: TransactionResult | null;
  /** Whether the transaction is in progress */
  isLoading: boolean;
  /** Current stage of the transaction */
  stage: TransactionStage;
  /** Error if transaction failed */
  error: Error | null;
  /** Reset the hook state */
  reset: () => void;
}

/**
 * Hook to consume notes and claim their assets.
 *
 * @example
 * ```tsx
 * function ConsumeNotesButton({ accountId, noteIds }: Props) {
 *   const { consume, isLoading, stage, error } = useConsume();
 *
 *   const handleConsume = async () => {
 *     try {
 *       const result = await consume({
 *         accountId,
 *         noteIds,
 *       });
 *       console.log('Consumed! TX:', result.transactionId);
 *     } catch (err) {
 *       console.error('Consume failed:', err);
 *     }
 *   };
 *
 *   return (
 *     <button onClick={handleConsume} disabled={isLoading}>
 *       {isLoading ? stage : 'Claim Notes'}
 *     </button>
 *   );
 * }
 * ```
 */
export function useConsume(): UseConsumeResult {
  const { client, isReady, sync, runExclusive, prover } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;

  const [result, setResult] = useState<TransactionResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [stage, setStage] = useState<TransactionStage>("idle");
  const [error, setError] = useState<Error | null>(null);

  const consume = useCallback(
    async (options: ConsumeOptions): Promise<TransactionResult> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      if (options.notes.length === 0) {
        throw new Error("No notes provided");
      }

      setIsLoading(true);
      setStage("executing");
      setError(null);

      try {
        // Convert account ID string to AccountId object
        const accountIdObj = parseAccountId(options.accountId);

        setStage("proving");
        const txResult = await runExclusiveSafe(async () => {
          // Resolve each input to a Note object, preserving original order:
          // - InputNoteRecord (has .toNote()) → unwrap via .toNote()
          // - Note (has .id() but not .toNote()) → use directly
          // - string → look up from store by hex ID
          // - NoteId → look up from store
          const resolved: Note[] = new Array(options.notes.length);
          const lookupIndices: number[] = [];
          const lookupIds: NoteId[] = [];

          for (let i = 0; i < options.notes.length; i++) {
            const item = options.notes[i];
            if (typeof item === "string") {
              lookupIndices.push(i);
              lookupIds.push(NoteId.fromHex(item));
            } else if (
              item !== null &&
              typeof item === "object" &&
              typeof (item as InputNoteRecord).toNote === "function"
            ) {
              resolved[i] = (item as InputNoteRecord).toNote();
            } else if (
              item !== null &&
              typeof item === "object" &&
              typeof (item as Note).id === "function"
            ) {
              resolved[i] = item as Note;
            } else {
              lookupIndices.push(i);
              lookupIds.push(item as NoteId);
            }
          }

          if (lookupIds.length > 0) {
            const filter = new NoteFilter(NoteFilterTypes.List, lookupIds);
            const noteRecords = await client.getInputNotes(filter);

            if (noteRecords.length !== lookupIds.length) {
              throw new Error("Some notes could not be found for provided IDs");
            }

            // Match returned records back to their original positions by ID
            const recordById = new Map(
              noteRecords.map((r) => [r.id().toString(), r])
            );
            for (let j = 0; j < lookupIndices.length; j++) {
              const record = recordById.get(lookupIds[j].toString());
              if (!record) {
                throw new Error(
                  "Some notes could not be found for provided IDs"
                );
              }
              resolved[lookupIndices[j]] = record.toNote();
            }
          }

          const notes = resolved;

          if (notes.length === 0) {
            throw new Error("No notes found for provided IDs");
          }

          if (notes.length !== options.notes.length) {
            throw new Error("Some notes could not be found for provided IDs");
          }

          const txRequest = client.newConsumeTransactionRequest(notes);
          const txId = prover
            ? await client.submitNewTransactionWithProver(
                accountIdObj,
                txRequest,
                prover
              )
            : await client.submitNewTransaction(accountIdObj, txRequest);
          return { transactionId: txId.toString() };
        });

        setStage("complete");
        setResult(txResult);

        await sync();

        return txResult;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        setStage("idle");
        throw error;
      } finally {
        setIsLoading(false);
      }
    },
    [client, isReady, prover, runExclusive, sync]
  );

  const reset = useCallback(() => {
    setResult(null);
    setIsLoading(false);
    setStage("idle");
    setError(null);
  }, []);

  return {
    consume,
    result,
    isLoading,
    stage,
    error,
    reset,
  };
}
