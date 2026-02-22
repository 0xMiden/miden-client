import { useCallback, useRef, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import {
  FungibleAsset,
  Note,
  NoteAssets,
  NoteType,
  OutputNote,
  OutputNoteArray,
  TransactionRequestBuilder,
} from "@miden-sdk/miden-sdk";
import type {
  SendOptions,
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
  const isBusyRef = useRef(false);

  const [result, setResult] = useState<TransactionResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [stage, setStage] = useState<TransactionStage>("idle");
  const [error, setError] = useState<Error | null>(null);

  const send = useCallback(
    async (options: SendOptions): Promise<TransactionResult> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
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

        // Resolve amount — if sendAll, query the account balance
        let amount = options.amount;
        if (options.sendAll) {
          const resolvedAmount = await runExclusiveSafe(async () => {
            const fromId = parseAccountId(options.from);
            const account = await client.getAccount(fromId);
            if (!account) throw new Error("Account not found");
            const assetIdObj = parseAccountId(options.assetId);
            const balance = account.vault?.()?.getBalance?.(assetIdObj);
            if (balance === undefined || balance === null) {
              throw new Error("Could not query account balance");
            }
            const bal = BigInt(balance as number | bigint);
            if (bal === 0n) {
              throw new Error("Account has zero balance for this asset");
            }
            return bal;
          });
          amount = resolvedAmount;
        }

        if (amount === undefined || amount === null) {
          throw new Error("Amount is required (provide amount or sendAll)");
        }

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

        // Build transaction — use attachment path if attachment provided
        const hasAttachment =
          options.attachment !== undefined && options.attachment !== null;

        if (
          hasAttachment &&
          (options.recallHeight != null || options.timelockHeight != null)
        ) {
          throw new Error(
            "recallHeight and timelockHeight are not supported when attachment is provided"
          );
        }

        const txResult = await runExclusiveSafe(async () => {
          let txRequest;

          if (hasAttachment) {
            // Manual P2ID note construction with attachment
            const attachment = createNoteAttachment(options.attachment!);
            const assets = new NoteAssets([
              new FungibleAsset(assetIdObj, amount!),
            ]);
            const note = Note.createP2IDNote(
              fromAccountId,
              toAccountId,
              assets,
              noteType,
              attachment
            );
            txRequest = new TransactionRequestBuilder()
              .withOwnOutputNotes(new OutputNoteArray([OutputNote.full(note)]))
              .build();
          } else {
            txRequest = client.newSendTransactionRequest(
              fromAccountId,
              toAccountId,
              assetIdObj,
              noteType,
              amount!,
              options.recallHeight ?? null,
              options.timelockHeight ?? null
            );
          }

          return await client.executeTransaction(fromAccountId, txRequest);
        });

        setStage("proving");
        const provenTransaction = await runExclusiveSafe(() =>
          client.proveTransaction(txResult, prover ?? undefined)
        );

        setStage("submitting");
        const submissionHeight = await runExclusiveSafe(() =>
          client.submitProvenTransaction(provenTransaction, txResult)
        );

        // Save txId hex BEFORE applyTransaction, which consumes the WASM
        // pointer inside txResult (and any child objects like TransactionId).
        const txIdHex = txResult.id().toHex();
        const txIdString = txResult.id().toString();

        // For private notes, extract the full note BEFORE applyTransaction
        // consumes the WASM pointers.
        let fullNote: Note | null = null;
        if (noteType === NoteType.Private) {
          fullNote = extractFullNote(txResult);
        }

        await runExclusiveSafe(() =>
          client.applyTransaction(txResult, submissionHeight)
        );

        if (noteType === NoteType.Private) {
          if (!fullNote) {
            throw new Error("Missing full note for private send");
          }

          await waitForTransactionCommit(
            client as unknown as ClientWithTransactions,
            runExclusiveSafe,
            txIdHex
          );

          // Create a fresh AccountId — the original toAccountId may have been
          // consumed by Note.createP2IDNote or newSendTransactionRequest.
          const recipientAccountId = parseAccountId(options.to);
          const recipientAddress = parseAddress(options.to, recipientAccountId);
          await runExclusiveSafe(() =>
            client.sendPrivateNote(fullNote!, recipientAddress)
          );
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
