import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import {
  FungibleAsset,
  Note,
  NoteAndArgs,
  NoteAndArgsArray,
  NoteAssets,
  NoteAttachment,
  OutputNote,
  OutputNoteArray,
  TransactionRequestBuilder,
} from "@miden-sdk/miden-sdk";
import type {
  InternalTransferOptions,
  InternalTransferChainOptions,
  InternalTransferResult,
  TransactionStage,
} from "../types";
import { DEFAULTS } from "../types";
import { parseAccountId } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";
import { getNoteType } from "../utils/noteFilters";

export interface UseInternalTransferResult {
  /** Create a P2ID note and immediately consume it with another account */
  transfer: (
    options: InternalTransferOptions
  ) => Promise<InternalTransferResult>;
  /** Perform a chain of P2ID transfers across multiple accounts */
  transferChain: (
    options: InternalTransferChainOptions
  ) => Promise<InternalTransferResult[]>;
  /** The last transfer result(s) */
  result: InternalTransferResult | InternalTransferResult[] | null;
  /** Whether the transfer is in progress */
  isLoading: boolean;
  /** Current stage of the transfer */
  stage: TransactionStage;
  /** Error if transfer failed */
  error: Error | null;
  /** Reset the hook state */
  reset: () => void;
}

/**
 * Hook to create a P2ID note and immediately consume it.
 *
 * @example
 * ```tsx
 * function InternalTransferButton() {
 *   const { transfer, isLoading, stage } = useInternalTransfer();
 *
 *   const handleTransfer = async () => {
 *     await transfer({
 *       from: "mtst1...",
 *       to: "0x...",
 *       assetId: "0x...",
 *       amount: 50n,
 *       noteType: "public",
 *     });
 *   };
 *
 *   return (
 *     <button onClick={handleTransfer} disabled={isLoading}>
 *       {isLoading ? stage : "Transfer"}
 *     </button>
 *   );
 * }
 * ```
 */
export function useInternalTransfer(): UseInternalTransferResult {
  const { client, isReady, sync, runExclusive, prover } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;

  const [result, setResult] = useState<
    InternalTransferResult | InternalTransferResult[] | null
  >(null);
  const [isLoading, setIsLoading] = useState(false);
  const [stage, setStage] = useState<TransactionStage>("idle");
  const [error, setError] = useState<Error | null>(null);

  const transferOnce = useCallback(
    async (
      options: InternalTransferOptions
    ): Promise<InternalTransferResult> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      const noteType = getNoteType(options.noteType ?? DEFAULTS.NOTE_TYPE);
      const senderId = parseAccountId(options.from);
      const receiverId = parseAccountId(options.to);
      const assetId = parseAccountId(options.assetId);

      const assets = new NoteAssets([
        new FungibleAsset(assetId, options.amount),
      ]);
      const note = Note.createP2IDNote(
        senderId,
        receiverId,
        assets,
        noteType,
        new NoteAttachment()
      );
      const noteId = note.id().toString();

      const createRequest = new TransactionRequestBuilder()
        .withOwnOutputNotes(new OutputNoteArray([OutputNote.full(note)]))
        .build();

      const createTxId = await runExclusiveSafe(() =>
        prover
          ? client.submitNewTransactionWithProver(
              senderId,
              createRequest,
              prover
            )
          : client.submitNewTransaction(senderId, createRequest)
      );

      const consumeRequest = new TransactionRequestBuilder()
        .withInputNotes(new NoteAndArgsArray([new NoteAndArgs(note, null)]))
        .build();

      const consumeTxId = await runExclusiveSafe(() =>
        prover
          ? client.submitNewTransactionWithProver(
              receiverId,
              consumeRequest,
              prover
            )
          : client.submitNewTransaction(receiverId, consumeRequest)
      );

      return {
        createTransactionId: createTxId.toString(),
        consumeTransactionId: consumeTxId.toString(),
        noteId,
      };
    },
    [client, isReady, prover, runExclusiveSafe]
  );

  const transfer = useCallback(
    async (
      options: InternalTransferOptions
    ): Promise<InternalTransferResult> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      setIsLoading(true);
      setStage("executing");
      setError(null);

      try {
        setStage("proving");
        const txResult = await transferOnce(options);

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
    [client, isReady, sync, transferOnce]
  );

  const transferChain = useCallback(
    async (
      options: InternalTransferChainOptions
    ): Promise<InternalTransferResult[]> => {
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
        const results: InternalTransferResult[] = [];
        let currentSender = options.from;

        for (const recipient of options.recipients) {
          setStage("proving");
          const txResult = await transferOnce({
            from: currentSender,
            to: recipient,
            assetId: options.assetId,
            amount: options.amount,
            noteType: options.noteType,
          });
          results.push(txResult);
          currentSender = recipient;
        }

        setStage("complete");
        setResult(results);
        await sync();

        return results;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        setStage("idle");
        throw error;
      } finally {
        setIsLoading(false);
      }
    },
    [client, isReady, sync, transferOnce]
  );

  const reset = useCallback(() => {
    setResult(null);
    setIsLoading(false);
    setStage("idle");
    setError(null);
  }, []);

  return {
    transfer,
    transferChain,
    result,
    isLoading,
    stage,
    error,
    reset,
  };
}
