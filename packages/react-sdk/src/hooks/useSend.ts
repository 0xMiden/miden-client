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
import type { SendOptions, SendResult, TransactionStage } from "../types";
import { DEFAULTS } from "../types";
import { parseAccountId, parseAddress } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";
import { proveWithFallback } from "../utils/prover";
import { useMidenStore } from "../store/MidenStore";
import {
  waitForTransactionCommit,
  extractFullNote,
} from "../utils/transactions";

export interface UseSendResult {
  /** Send tokens from one account to another */
  send: (options: SendOptions) => Promise<SendResult>;
  /** The transaction result */
  result: SendResult | null;
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
  const { client, isReady, sync, runExclusive, prover } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;

  const [result, setResult] = useState<SendResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [stage, setStage] = useState<TransactionStage>("idle");
  const [error, setError] = useState<Error | null>(null);

  const send = useCallback(
    async (options: SendOptions): Promise<SendResult> => {
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

        // returnNote path: build note in JS, submit as output note, return Note object
        if (options.returnNote === true) {
          const assets = new NoteAssets([
            new FungibleAsset(assetIdObj, options.amount),
          ]);
          const p2idNote = Note.createP2IDNote(
            fromAccountId,
            toAccountId,
            assets,
            noteType,
            new NoteAttachment()
          );

          const txRequest = new TransactionRequestBuilder()
            .withOwnOutputNotes(
              new OutputNoteArray([OutputNote.full(p2idNote)])
            )
            .build();

          setStage("proving");
          const txId = await runExclusiveSafe(() =>
            prover
              ? client.submitNewTransactionWithProver(
                  fromAccountId,
                  txRequest,
                  prover
                )
              : client.submitNewTransaction(fromAccountId, txRequest)
          );

          const sendResult: SendResult = {
            txId: txId.toString(),
            note: p2idNote,
          };

          setStage("complete");
          setResult(sendResult);
          await sync();

          return sendResult;
        }

        // On-chain path (default)
        const txResult = await runExclusiveSafe(async () => {
          const txRequest = client.newSendTransactionRequest(
            fromAccountId,
            toAccountId,
            assetIdObj,
            noteType,
            options.amount,
            options.recallHeight ?? null,
            options.timelockHeight ?? null
          );

          return await client.executeTransaction(fromAccountId, txRequest);
        });

        setStage("proving");
        const proverConfig = useMidenStore.getState().config;
        const provenTransaction = await proveWithFallback(
          (resolvedProver) =>
            runExclusiveSafe(() =>
              client.proveTransaction(txResult, resolvedProver)
            ),
          proverConfig
        );

        setStage("submitting");
        const submissionHeight = await runExclusiveSafe(() =>
          client.submitProvenTransaction(provenTransaction, txResult)
        );

        await runExclusiveSafe(() =>
          client.applyTransaction(txResult, submissionHeight)
        );

        const txId = txResult.id();
        await waitForTransactionCommit(client, runExclusiveSafe, txId);

        if (noteType === NoteType.Private) {
          const fullNote = extractFullNote(txResult);
          if (!fullNote) {
            throw new Error("Missing full note for private send");
          }

          const recipientAddress = parseAddress(options.to, toAccountId);
          await runExclusiveSafe(() =>
            client.sendPrivateNote(fullNote, recipientAddress)
          );
        }

        const sendResult: SendResult = {
          txId: txResult.id().toString(),
          note: null,
        };

        setStage("complete");
        setResult(sendResult);

        await sync();

        return sendResult;
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
    send,
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
