import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import { NoteType, AccountId } from "@miden-sdk/miden-sdk";
import type {
  SwapOptions,
  TransactionStage,
  TransactionResult,
} from "../types";
import { DEFAULTS } from "../types";
import { runExclusiveDirect } from "../utils/runExclusive";

export interface UseSwapResult {
  /** Create an atomic swap offer */
  swap: (options: SwapOptions) => Promise<TransactionResult>;
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
 * Hook to create atomic swap transactions.
 *
 * @example
 * ```tsx
 * function SwapButton({ accountId }: { accountId: string }) {
 *   const { swap, isLoading, stage, error } = useSwap();
 *
 *   const handleSwap = async () => {
 *     try {
 *       const result = await swap({
 *         accountId,
 *         offeredFaucetId: '0x...', // Token A
 *         offeredAmount: 100n,
 *         requestedFaucetId: '0x...', // Token B
 *         requestedAmount: 50n,
 *       });
 *       console.log('Swap created! TX:', result.transactionId);
 *     } catch (err) {
 *       console.error('Swap failed:', err);
 *     }
 *   };
 *
 *   return (
 *     <button onClick={handleSwap} disabled={isLoading}>
 *       {isLoading ? stage : 'Create Swap'}
 *     </button>
 *   );
 * }
 * ```
 */
export function useSwap(): UseSwapResult {
  const { client, isReady, sync, runExclusive, prover } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;

  const [result, setResult] = useState<TransactionResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [stage, setStage] = useState<TransactionStage>("idle");
  const [error, setError] = useState<Error | null>(null);

  const swap = useCallback(
    async (options: SwapOptions): Promise<TransactionResult> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      setIsLoading(true);
      setStage("executing");
      setError(null);

      try {
        const noteType = getNoteType(options.noteType ?? DEFAULTS.NOTE_TYPE);
        const paybackNoteType = getNoteType(
          options.paybackNoteType ?? DEFAULTS.NOTE_TYPE
        );

        // Convert string IDs to AccountId objects
        const accountIdObj = AccountId.fromHex(options.accountId);
        const offeredFaucetIdObj = AccountId.fromHex(options.offeredFaucetId);
        const requestedFaucetIdObj = AccountId.fromHex(
          options.requestedFaucetId
        );

        setStage("proving");
        const txResult = await runExclusiveSafe(async () => {
          const txRequest = client.newSwapTransactionRequest(
            accountIdObj,
            offeredFaucetIdObj,
            options.offeredAmount,
            requestedFaucetIdObj,
            options.requestedAmount,
            noteType,
            paybackNoteType
          );

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
    swap,
    result,
    isLoading,
    stage,
    error,
    reset,
  };
}

function getNoteType(type: "private" | "public" | "encrypted"): NoteType {
  switch (type) {
    case "private":
      return NoteType.Private;
    case "public":
      return NoteType.Public;
    case "encrypted":
      return NoteType.Encrypted;
    default:
      return NoteType.Private;
  }
}
