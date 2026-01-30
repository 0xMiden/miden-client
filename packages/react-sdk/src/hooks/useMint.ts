import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import { NoteType, AccountId } from "@miden-sdk/miden-sdk";
import type {
  MintOptions,
  TransactionStage,
  TransactionResult,
} from "../types";
import { DEFAULTS } from "../types";
import { runExclusiveDirect } from "../utils/runExclusive";

export interface UseMintResult {
  /** Mint tokens from a faucet to a target account */
  mint: (options: MintOptions) => Promise<TransactionResult>;
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
 * Hook to mint tokens from a faucet.
 *
 * @example
 * ```tsx
 * function MintButton({ faucetId, targetAccountId }: Props) {
 *   const { mint, isLoading, stage, error } = useMint();
 *
 *   const handleMint = async () => {
 *     try {
 *       const result = await mint({
 *         faucetId,
 *         targetAccountId,
 *         amount: 1000n,
 *       });
 *       console.log('Minted! TX:', result.transactionId);
 *     } catch (err) {
 *       console.error('Mint failed:', err);
 *     }
 *   };
 *
 *   return (
 *     <button onClick={handleMint} disabled={isLoading}>
 *       {isLoading ? stage : 'Mint Tokens'}
 *     </button>
 *   );
 * }
 * ```
 */
export function useMint(): UseMintResult {
  const { client, isReady, sync, runExclusive, prover } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;

  const [result, setResult] = useState<TransactionResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [stage, setStage] = useState<TransactionStage>("idle");
  const [error, setError] = useState<Error | null>(null);

  const mint = useCallback(
    async (options: MintOptions): Promise<TransactionResult> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      setIsLoading(true);
      setStage("executing");
      setError(null);

      try {
        const noteType = getNoteType(options.noteType ?? DEFAULTS.NOTE_TYPE);

        // Convert string IDs to AccountId objects
        const targetAccountIdObj = AccountId.fromHex(options.targetAccountId);
        const faucetIdObj = AccountId.fromHex(options.faucetId);

        setStage("proving");
        const txResult = await runExclusiveSafe(async () => {
          const txRequest = client.newMintTransactionRequest(
            targetAccountIdObj,
            faucetIdObj,
            noteType,
            options.amount
          );

          const txId = prover
            ? await client.submitNewTransactionWithProver(
                faucetIdObj,
                txRequest,
                prover
              )
            : await client.submitNewTransaction(faucetIdObj, txRequest);

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
    mint,
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
