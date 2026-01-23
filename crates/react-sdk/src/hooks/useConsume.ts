import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import { AccountId } from "@miden-sdk/miden-sdk";
import type { ConsumeOptions, TransactionStage, TransactionResult } from "../types";

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
  const { client, isReady, sync } = useMiden();

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

        // Create the consume transaction request
        const txRequest = client.newConsumeTransactionRequest(options.noteIds);

        // Execute, prove, and submit in one call
        setStage("proving");
        const txId = await client.submitNewTransaction(accountIdObj, txRequest);

        setStage("complete");

        const txResult: TransactionResult = {
          transactionId: txId.toString(),
        };

        setResult(txResult);

        // Trigger sync to update state
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
    [client, isReady, sync]
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
