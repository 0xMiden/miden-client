import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import {
  AccountId,
  NoteFilter,
  NoteFilterTypes,
  NoteId,
} from "@miden-sdk/miden-sdk";
import type {
  ConsumeOptions,
  TransactionStage,
  TransactionResult,
} from "../types";
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

      if (options.noteIds.length === 0) {
        throw new Error("No note IDs provided");
      }

      setIsLoading(true);
      setStage("executing");
      setError(null);

      try {
        // Convert account ID string to AccountId object
        const accountIdObj = AccountId.fromHex(options.accountId);

        setStage("proving");
        const txResult = await runExclusiveSafe(async () => {
          const noteIds = options.noteIds.map((noteId) =>
            NoteId.fromHex(noteId)
          );
          const filter = new NoteFilter(NoteFilterTypes.List, noteIds);
          const noteRecords = await client.getInputNotes(filter);
          const notes = noteRecords.map((record) => record.toNote());

          if (notes.length === 0) {
            throw new Error("No notes found for provided IDs");
          }

          if (notes.length !== options.noteIds.length) {
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
