import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import type {
  TransactionRequest,
  WasmWebClient as WebClient,
} from "@miden-sdk/miden-sdk";
import type {
  TransactionStage,
  TransactionResult,
  ExecuteTransactionOptions,
} from "../types";
import { parseAccountId, parseAddress } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";
import { proveWithFallback } from "../utils/prover";
import { useMidenStore } from "../store/MidenStore";
import {
  waitForTransactionCommit,
  extractFullNotes,
} from "../utils/transactions";

export interface UseTransactionResult {
  /** Execute a transaction request end-to-end */
  execute: (options: ExecuteTransactionOptions) => Promise<TransactionResult>;
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

type TransactionRequestFactory = (
  client: WebClient
) => TransactionRequest | Promise<TransactionRequest>;

/**
 * Hook to execute arbitrary transaction requests.
 *
 * When `privateNoteRecipient` is set, uses the 4-step pipeline
 * (execute → prove → submit → apply) and delivers private output notes
 * to the recipient after commit.
 *
 * @example
 * ```tsx
 * function CustomTransactionButton({ accountId }: { accountId: string }) {
 *   const { execute, isLoading, stage } = useTransaction();
 *
 *   const handleClick = async () => {
 *     await execute({
 *       accountId,
 *       request: (client) =>
 *         client.newSwapTransactionRequest(
 *           AccountId.fromHex(accountId),
 *           AccountId.fromHex("0x..."),
 *           10n,
 *           AccountId.fromHex("0x..."),
 *           5n,
 *           NoteType.Private,
 *           NoteType.Private
 *         ),
 *       privateNoteRecipient: "0xrecipient...",
 *     });
 *   };
 *
 *   return (
 *     <button onClick={handleClick} disabled={isLoading}>
 *       {isLoading ? stage : "Run Transaction"}
 *     </button>
 *   );
 * }
 * ```
 */
export function useTransaction(): UseTransactionResult {
  const { client, isReady, sync, runExclusive, prover } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;

  const [result, setResult] = useState<TransactionResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [stage, setStage] = useState<TransactionStage>("idle");
  const [error, setError] = useState<Error | null>(null);

  const execute = useCallback(
    async (options: ExecuteTransactionOptions): Promise<TransactionResult> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      setIsLoading(true);
      setStage("executing");
      setError(null);

      try {
        const accountIdObj = parseAccountId(options.accountId);
        const txRequest = await resolveRequest(options.request, client);

        if (options.privateNoteRecipient != null) {
          // 4-step pipeline: execute → prove → submit → apply → deliver private notes
          const txResult = await runExclusiveSafe(() =>
            client.executeTransaction(accountIdObj, txRequest)
          );

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

          // Deliver private output notes to recipient
          const recipientAddress = parseAddress(options.privateNoteRecipient);
          const fullNotes = extractFullNotes(txResult);
          for (const note of fullNotes) {
            await runExclusiveSafe(() =>
              client.sendPrivateNote(note, recipientAddress)
            );
          }

          const txSummary = { transactionId: txId.toString() };

          setStage("complete");
          setResult(txSummary);
          await sync();
          return txSummary;
        }

        // Simple path: submitNewTransaction (no private note delivery needed)
        setStage("proving");
        const txResult = await runExclusiveSafe(async () => {
          const txId = prover
            ? await client.submitNewTransactionWithProver(
                accountIdObj,
                txRequest,
                prover
              )
            : await client.submitNewTransaction(accountIdObj, txRequest);
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
    execute,
    result,
    isLoading,
    stage,
    error,
    reset,
  };
}

async function resolveRequest(
  request: TransactionRequest | TransactionRequestFactory,
  client: WebClient
): Promise<TransactionRequest> {
  if (typeof request === "function") {
    return await request(client);
  }
  return request;
}
