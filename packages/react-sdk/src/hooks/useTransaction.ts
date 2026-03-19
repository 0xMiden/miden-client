import { useCallback, useRef, useState } from "react";
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
import { MidenError } from "../utils/errors";
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
 * Always uses the 4-step pipeline (execute → prove → submit → apply)
 * with prover fallback support. When `privateNoteTarget` is set,
 * additionally waits for commit and delivers private output notes.
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
 *       privateNoteTarget: "0xrecipient...",
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
  const { client, isReady, sync, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;
  const isBusyRef = useRef(false);

  const [result, setResult] = useState<TransactionResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [stage, setStage] = useState<TransactionStage>("idle");
  const [error, setError] = useState<Error | null>(null);

  const execute = useCallback(
    async (options: ExecuteTransactionOptions): Promise<TransactionResult> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      if (isBusyRef.current) {
        throw new MidenError(
          "A transaction is already in progress. Await the previous transaction before starting another.",
          { code: "SEND_BUSY" }
        );
      }

      isBusyRef.current = true;
      setIsLoading(true);
      setStage("executing");
      setError(null);

      try {
        // Auto-sync before transaction unless opted out
        if (!options.skipSync) {
          await sync();
        }

        // Resolve request outside runExclusiveSafe so the "executing" stage
        // is observable before transitioning to "proving"
        const txRequest = await resolveRequest(options.request, client);

        // Step 1: Execute
        const txResult = await runExclusiveSafe(() => {
          const accountIdObj = parseAccountId(options.accountId);
          return client.executeTransaction(accountIdObj, txRequest);
        });

        // Step 2: Prove (with fallback)
        setStage("proving");
        const proverConfig = useMidenStore.getState().config;
        const provenTransaction = await proveWithFallback(
          (resolvedProver) =>
            runExclusiveSafe(() =>
              client.proveTransaction(txResult, resolvedProver)
            ),
          proverConfig
        );

        // Step 3: Submit
        setStage("submitting");
        const submissionHeight = await runExclusiveSafe(() =>
          client.submitProvenTransaction(provenTransaction, txResult)
        );

        // Step 4: Apply
        await runExclusiveSafe(() =>
          client.applyTransaction(txResult, submissionHeight)
        );

        // Deliver private notes if requested
        const txId = txResult.id();
        if (options.privateNoteTarget != null) {
          await waitForTransactionCommit(client, runExclusiveSafe, txId);

          const targetAddress = parseAddress(options.privateNoteTarget);
          const fullNotes = extractFullNotes(txResult);
          for (const note of fullNotes) {
            await runExclusiveSafe(() =>
              client.sendPrivateNote(note, targetAddress)
            );
          }
        }

        const txSummary = { transactionId: txId.toString() };
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
    [client, isReady, runExclusive, sync]
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
