import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import { NoteType, AccountId } from "@demox-labs/miden-sdk";
import type { SendOptions, TransactionStage, TransactionResult } from "../types";
import { DEFAULTS } from "../types";

export interface UseSendResult {
  /** Send tokens from one account to another */
  send: (options: SendOptions) => Promise<TransactionResult>;
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
 * Hook to send tokens between accounts.
 *
 * @example
 * ```tsx
 * function SendButton({ from, to, faucetId }: Props) {
 *   const { send, isLoading, stage, error } = useSend();
 *
 *   const handleSend = async () => {
 *     try {
 *       const result = await send({
 *         from,
 *         to,
 *         faucetId,
 *         amount: 100n,
 *       });
 *       console.log('Transaction ID:', result.transactionId);
 *     } catch (err) {
 *       console.error('Send failed:', err);
 *     }
 *   };
 *
 *   return (
 *     <button onClick={handleSend} disabled={isLoading}>
 *       {isLoading ? stage : 'Send'}
 *     </button>
 *   );
 * }
 * ```
 */
export function useSend(): UseSendResult {
  const { client, isReady, sync } = useMiden();

  const [result, setResult] = useState<TransactionResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [stage, setStage] = useState<TransactionStage>("idle");
  const [error, setError] = useState<Error | null>(null);

  const send = useCallback(
    async (options: SendOptions): Promise<TransactionResult> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      setIsLoading(true);
      setStage("executing");
      setError(null);

      try {
        const noteType = getNoteType(options.noteType ?? DEFAULTS.NOTE_TYPE);

        // Convert string IDs to AccountId objects
        const fromAccountId = AccountId.fromHex(options.from);
        const toAccountId = AccountId.fromHex(options.to);
        const faucetIdObj = AccountId.fromHex(options.faucetId);

        // Create the send transaction request
        const txRequest = client.newSendTransactionRequest(
          fromAccountId,
          toAccountId,
          faucetIdObj,
          noteType,
          options.amount,
          options.recallHeight ?? null,
          options.timelockHeight ?? null
        );

        // Execute, prove, and submit in one call
        setStage("proving");
        const txId = await client.submitNewTransaction(fromAccountId, txRequest);

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
    send,
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
