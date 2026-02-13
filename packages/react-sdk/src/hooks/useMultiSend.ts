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
  TransactionFilter,
  TransactionRequestBuilder,
} from "@miden-sdk/miden-sdk";
import type { TransactionId } from "@miden-sdk/miden-sdk";
import type {
  MultiSendOptions,
  TransactionStage,
  TransactionResult,
} from "../types";
import { DEFAULTS } from "../types";
import { parseAccountId, parseAddress } from "../utils/accountParsing";

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

type ClientWithTransactions = {
  syncState: () => Promise<unknown>;
  getTransactions: (filter: TransactionFilter) => Promise<
    Array<{
      id: () => { toHex: () => string };
      transactionStatus: () => {
        isPending: () => boolean;
        isCommitted: () => boolean;
        isDiscarded: () => boolean;
      };
    }>
  >;
};

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
  const { client, isReady, sync, prover } = useMiden();

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

        const outputs = options.recipients.map(({ to, amount }) => {
          const receiverId = parseAccountId(to);
          const assets = new NoteAssets([new FungibleAsset(assetId, amount)]);
          const note = Note.createP2IDNote(
            senderId,
            receiverId,
            assets,
            noteType,
            new NoteAttachment()
          );
          const recipientAddress = parseAddress(to, receiverId);
          return {
            outputNote: OutputNote.full(note),
            note,
            recipientAddress,
          };
        });

        const txRequest = new TransactionRequestBuilder()
          .withOwnOutputNotes(
            new OutputNoteArray(outputs.map((o) => o.outputNote))
          )
          .build();

        const txResult = await client.executeTransaction(senderId, txRequest);

        setStage("proving");
        const provenTransaction = await client.proveTransaction(
          txResult,
          prover ?? undefined
        );

        setStage("submitting");
        const submissionHeight = await client.submitProvenTransaction(
          provenTransaction,
          txResult
        );

        // Save txId BEFORE applyTransaction, which consumes the WASM pointer
        // inside txResult.
        const txId = txResult.id();
        const txIdString = txId.toString();

        await client.applyTransaction(txResult, submissionHeight);

        if (noteType === NoteType.Private) {
          await waitForTransactionCommit(
            client as ClientWithTransactions,
            txId
          );

          for (const output of outputs) {
            await client.sendPrivateNote(output.note, output.recipientAddress);
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
      }
    },
    [client, isReady, prover, sync]
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

async function waitForTransactionCommit(
  client: ClientWithTransactions,
  txId: TransactionId,
  maxWaitMs = 10_000,
  delayMs = 1_000
) {
  let waited = 0;

  while (waited < maxWaitMs) {
    await client.syncState();
    const [record] = await client.getTransactions(
      TransactionFilter.ids([txId])
    );
    if (record) {
      const status = record.transactionStatus();
      if (status.isCommitted()) {
        return;
      }
      if (status.isDiscarded()) {
        throw new Error("Transaction was discarded before commit");
      }
    }
    await new Promise((resolve) => setTimeout(resolve, delayMs));
    waited += delayMs;
  }

  throw new Error("Timeout waiting for transaction commit");
}
