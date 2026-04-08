import { useCallback, useRef, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import {
  FungibleAsset,
  Note,
  NoteAssets,
  NoteAttachment,
  NoteType,
  NoteArray,
  TransactionRequestBuilder,
} from "@miden-sdk/miden-sdk";
import type { SendOptions, SendResult, TransactionStage } from "../types";
import { DEFAULTS } from "../types";
import { parseAccountId, parseAddress } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";
import { createNoteAttachment } from "../utils/noteAttachment";
import { MidenError } from "../utils/errors";
import { getNoteType, waitForTransactionCommit } from "../utils/noteFilters";
import type { ClientWithTransactions } from "../utils/noteFilters";
import { proveWithFallback } from "../utils/prover";
import { useMidenStore } from "../store/MidenStore";

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
  const isBusyRef = useRef(false);

  const [result, setResult] = useState<SendResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [stage, setStage] = useState<TransactionStage>("idle");
  const [error, setError] = useState<Error | null>(null);

  const send = useCallback(
    async (options: SendOptions): Promise<SendResult> => {
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
        amount = BigInt(amount);

        const assetId =
          options.assetId ??
          (options as { faucetId?: string }).faucetId ??
          null;
        if (!assetId) {
          throw new Error("Asset ID is required");
        }

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

        // returnNote path: build note in JS, submit as output note, return Note object
        if (options.returnNote === true) {
          const returnResult = await runExclusiveSafe(async () => {
            const fromId = parseAccountId(options.from);
            const toId = parseAccountId(options.to);
            const assetObj = parseAccountId(assetId);

            const assets = new NoteAssets([
              new FungibleAsset(assetObj, BigInt(amount!)),
            ]);
            const p2idNote = Note.createP2IDNote(
              fromId,
              toId,
              assets,
              noteType,
              new NoteAttachment()
            );

            const txRequest = new TransactionRequestBuilder()
              .withOwnOutputNotes(new NoteArray([p2idNote]))
              .build();

            const execFromId = parseAccountId(options.from);
            const txId = prover
              ? await client.submitNewTransactionWithProver(
                  execFromId,
                  txRequest,
                  prover
                )
              : await client.submitNewTransaction(execFromId, txRequest);

            return { txId: txId.toString(), note: p2idNote } as SendResult;
          });

          setStage("complete");
          setResult(returnResult);
          await sync();

          return returnResult;
        }

        // On-chain path (default)
        const txResult = await runExclusiveSafe(async () => {
          // Create all WASM AccountId objects inside runExclusiveSafe to
          // avoid stale pointers if another exclusive operation runs between
          // creation and consumption.
          const fromAccountId = parseAccountId(options.from);
          const toAccountId = parseAccountId(options.to);
          const assetIdObj = parseAccountId(assetId);

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
              .withOwnOutputNotes(new NoteArray([note]))
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

          // Fresh AccountId — the originals may have been consumed by
          // createP2IDNote or newSendTransactionRequest above.
          const execAccountId = parseAccountId(options.from);
          return await client.executeTransaction(execAccountId, txRequest);
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

        const sendResult: SendResult = {
          txId: txIdString,
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
