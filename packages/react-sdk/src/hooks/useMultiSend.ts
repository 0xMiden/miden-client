import { useCallback, useRef, useState } from "react";
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
import { parseAccountId, parseAddress } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";
import { createNoteAttachment } from "../utils/noteAttachment";
import { MidenError } from "../utils/errors";
import { waitForTransactionCommit } from "../utils/noteFilters";
import type { ClientWithTransactions } from "../utils/noteFilters";

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
  const isBusyRef = useRef(false);

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

      if (isBusyRef.current) {
        throw new MidenError(
          "A send is already in progress. Await the previous send before starting another.",
          { code: "SEND_BUSY" }
        );
      }

      isBusyRef.current = true;
      setIsLoading(true);
      setStage("executing");
      setError(null);

      try {
        // Auto-sync before send unless opted out
        if (!options.skipSync) {
          await sync();
        }

        const noteType = getNoteType(options.noteType ?? DEFAULTS.NOTE_TYPE);
        const senderId = parseAccountId(options.from);
        const assetId = parseAccountId(options.assetId);

        const outputs = options.recipients.map(
          ({ to, amount, attachment, noteType: recipientNoteType }) => {
            const receiverId = parseAccountId(to);
            const assets = new NoteAssets([new FungibleAsset(assetId, amount)]);
            const resolvedNoteType = recipientNoteType
              ? getNoteType(recipientNoteType)
              : noteType;
            const noteAttachment =
              attachment !== undefined && attachment !== null
                ? createNoteAttachment(attachment)
                : new NoteAttachment();
            const note = Note.createP2IDNote(
              senderId,
              receiverId,
              assets,
              resolvedNoteType,
              noteAttachment
            );
            const recipientAddress = parseAddress(to, receiverId);
            return {
              outputNote: OutputNote.full(note),
              note,
              recipientAddress,
              noteType: resolvedNoteType,
            };
          }
        );

        const txRequest = new TransactionRequestBuilder()
          .withOwnOutputNotes(
            new OutputNoteArray(outputs.map((o) => o.outputNote))
          )
          .build();

        const txResult = await runExclusiveSafe(() =>
          client.executeTransaction(senderId, txRequest)
        );

        setStage("proving");
        const provenTransaction = await runExclusiveSafe(() =>
          client.proveTransaction(txResult, prover ?? undefined)
        );

        setStage("submitting");
        const submissionHeight = await runExclusiveSafe(() =>
          client.submitProvenTransaction(provenTransaction, txResult)
        );

        // Save txId BEFORE applyTransaction, which consumes the WASM pointer
        const txId = txResult.id();
        const txIdString = txId.toString();

        await runExclusiveSafe(() =>
          client.applyTransaction(txResult, submissionHeight)
        );

        // Send private notes after commit
        const hasPrivate = outputs.some((o) => o.noteType === NoteType.Private);
        if (hasPrivate) {
          await waitForTransactionCommit(
            client as unknown as ClientWithTransactions,
            runExclusiveSafe,
            txId
          );

          for (const output of outputs) {
            if (output.noteType === NoteType.Private) {
              await runExclusiveSafe(() =>
                client.sendPrivateNote(output.note, output.recipientAddress)
              );
            }
          }
        }

        const txSummary = { transactionId: txIdString };

        setStage("complete");
        setResult(txSummary);

        await sync();

        return txSummary;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        setStage("idle");
        throw error;
      } finally {
        setIsLoading(false);
        isBusyRef.current = false;
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

function getNoteType(type: "private" | "public"): NoteType {
  switch (type) {
    case "private":
      return NoteType.Private;
    case "public":
      return NoteType.Public;
    default:
      return NoteType.Private;
  }
}
