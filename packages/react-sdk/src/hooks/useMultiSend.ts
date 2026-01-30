import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import {
  FungibleAsset,
  Note,
  NoteAssets,
  NoteAttachment,
  NoteType,
  OutputNote,
  OutputNoteArray,
  TransactionRequestBuilder,
} from "@miden-sdk/miden-sdk";
import type {
  MultiSendOptions,
  TransactionStage,
  TransactionResult,
} from "../types";
import { DEFAULTS } from "../types";
import { parseAccountId } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";

export interface UseMultiSendResult {
  /** Create multiple P2ID output notes in one transaction */
  sendMany: (options: MultiSendOptions) => Promise<TransactionResult>;
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
 * Hook to create a multi-send transaction (multiple P2ID notes).
 *
 * @example
 * ```tsx
 * function MultiSendButton() {
 *   const { sendMany, isLoading, stage } = useMultiSend();
 *
 *   const handleSend = async () => {
 *     await sendMany({
 *       from: "mtst1...",
 *       assetId: "0x...",
 *       recipients: [
 *         { to: "mtst1...", amount: 100n },
 *         { to: "0x...", amount: 250n },
 *       ],
 *       noteType: "public",
 *     });
 *   };
 *
 *   return (
 *     <button onClick={handleSend} disabled={isLoading}>
 *       {isLoading ? stage : "Multi-send"}
 *     </button>
 *   );
 * }
 * ```
 */
export function useMultiSend(): UseMultiSendResult {
  const { client, isReady, sync, runExclusive, prover } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;

  const [result, setResult] = useState<TransactionResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [stage, setStage] = useState<TransactionStage>("idle");
  const [error, setError] = useState<Error | null>(null);

  const sendMany = useCallback(
    async (options: MultiSendOptions): Promise<TransactionResult> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      if (options.recipients.length === 0) {
        throw new Error("No recipients provided");
      }

      setIsLoading(true);
      setStage("executing");
      setError(null);

      try {
        const noteType = getNoteType(options.noteType ?? DEFAULTS.NOTE_TYPE);
        const senderId = parseAccountId(options.from);
        const assetId = parseAccountId(options.assetId);

        const outputNotes = options.recipients.map(({ to, amount }) => {
          const receiverId = parseAccountId(to);
          const assets = new NoteAssets([new FungibleAsset(assetId, amount)]);
          const note = Note.createP2IDNote(
            senderId,
            receiverId,
            assets,
            noteType,
            new NoteAttachment()
          );
          return OutputNote.full(note);
        });

        setStage("proving");
        const txResult = await runExclusiveSafe(async () => {
          const txRequest = new TransactionRequestBuilder()
            .withOwnOutputNotes(new OutputNoteArray(outputNotes))
            .build();

          const txId = prover
            ? await client.submitNewTransactionWithProver(
                senderId,
                txRequest,
                prover
              )
            : await client.submitNewTransaction(senderId, txRequest);

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
    sendMany,
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
