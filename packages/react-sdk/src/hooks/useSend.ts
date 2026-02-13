import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import { NoteType, TransactionFilter } from "@miden-sdk/miden-sdk";
import type { Note, TransactionId } from "@miden-sdk/miden-sdk";
import type {
  SendOptions,
  TransactionStage,
  TransactionResult,
} from "../types";
import { DEFAULTS } from "../types";
import { parseAccountId, parseAddress } from "../utils/accountParsing";

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
 * Hook to send tokens between accounts.
 *
 * @example
 * ```tsx
 * function SendButton({ from, to, assetId }: Props) {
 *   const { send, isLoading, stage, error } = useSend();
 *
 *   const handleSend = async () => {
 *     try {
 *       const result = await send({
 *         from,
 *         to,
 *         assetId,
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
  const { client, isReady, sync, prover } = useMiden();

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
        const fromAccountId = parseAccountId(options.from);
        const toAccountId = parseAccountId(options.to);
        const assetId =
          options.assetId ??
          (options as { faucetId?: string }).faucetId ??
          null;
        if (!assetId) {
          throw new Error("Asset ID is required");
        }
        const assetIdObj = parseAccountId(assetId);

        const txRequest = client.newSendTransactionRequest(
          fromAccountId,
          toAccountId,
          assetIdObj,
          noteType,
          options.amount,
          options.recallHeight ?? null,
          options.timelockHeight ?? null
        );

        const txResult = await client.executeTransaction(
          fromAccountId,
          txRequest
        );

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

        // Save txIdString BEFORE applyTransaction, which consumes the WASM
        // pointer inside txResult (and any child objects like TransactionId).
        const txIdString = txResult.id().toString();

        // For private notes we need to wait for commit before sending the
        // note via the transport. Extract the full note and the txId for
        // the commit-wait loop BEFORE applyTransaction invalidates them.
        let fullNote: Note | null = null;
        let txIdForWait: TransactionId | undefined;
        if (noteType === NoteType.Private) {
          fullNote = extractFullNote(txResult);
          txIdForWait = txResult.id();
        }

        await client.applyTransaction(txResult, submissionHeight);

        if (noteType === NoteType.Private) {
          if (!fullNote || !txIdForWait) {
            throw new Error("Missing full note for private send");
          }

          await waitForTransactionCommit(
            client as ClientWithTransactions,
            txIdForWait
          );

          const recipientAddress = parseAddress(options.to, toAccountId);
          await client.sendPrivateNote(fullNote, recipientAddress);
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

function extractFullNote(txResult: unknown): Note | null {
  try {
    const executedTx = (
      txResult as { executedTransaction?: () => unknown }
    ).executedTransaction?.() as {
      outputNotes?: () => {
        notes?: () => Array<{ intoFull?: () => Note | null }>;
      };
    };
    const notes = executedTx?.outputNotes?.().notes?.() ?? [];
    const note = notes[0];
    return note?.intoFull?.() ?? null;
  } catch {
    return null;
  }
}
