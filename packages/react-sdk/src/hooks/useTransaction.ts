import { useCallback, useRef, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import type {
  TransactionRequest,
  WasmWebClient as WebClient,
  AccountId as AccountIdType,
} from "@miden-sdk/miden-sdk";
import type {
  TransactionStage,
  TransactionResult,
  ExecuteTransactionOptions,
} from "../types";
import { parseAccountId } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";
import { MidenError } from "../utils/errors";

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

export function useTransaction(): UseTransactionResult {
  const { client, isReady, sync, runExclusive, prover } = useMiden();
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

        // Resolve account ID and request — this is the async work that makes
        // "executing" stage observable before transitioning to "proving"
        const accountIdObj = resolveAccountId(options.accountId);
        const txRequest = await resolveRequest(options.request, client);

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
    execute,
    result,
    isLoading,
    stage,
    error,
    reset,
  };
}

function resolveAccountId(accountId: string | AccountIdType): AccountIdType {
  return parseAccountId(accountId);
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
