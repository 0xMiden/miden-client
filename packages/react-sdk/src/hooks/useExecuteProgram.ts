import { useCallback, useRef, useState } from "react";
import {
  AdviceInputs,
  ForeignAccount,
  ForeignAccountArray,
  AccountStorageRequirements,
} from "@miden-sdk/miden-sdk";
import { useMiden } from "../context/MidenProvider";
import type { ExecuteProgramOptions, ExecuteProgramResult } from "../types";
import { parseAccountId } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";
import { MidenError } from "../utils/errors";

export interface UseExecuteProgramResult {
  /** Execute a program (view call) and return the stack output */
  execute: (options: ExecuteProgramOptions) => Promise<ExecuteProgramResult>;
  /** The most recent result */
  result: ExecuteProgramResult | null;
  /** Whether execution is in progress */
  isLoading: boolean;
  /** Error if execution failed */
  error: Error | null;
  /** Reset the hook state */
  reset: () => void;
}

/**
 * Hook to execute a program (view call) against an account and read the
 * resulting 16-element stack output. This runs locally and does not submit
 * anything to the network.
 *
 * @example
 * ```tsx
 * const { execute, result, isLoading } = useExecuteProgram();
 *
 * const handleExecute = async () => {
 *   const { stack } = await execute({
 *     accountId: "0x...",
 *     script: compiledTxScript,
 *   });
 *   console.log("Stack output:", stack);
 * };
 * ```
 */
type ForeignAccountWrapper = {
  id: string;
  storage?: AccountStorageRequirements;
};

function isForeignAccountWrapper(fa: unknown): fa is ForeignAccountWrapper {
  return (
    fa !== null &&
    typeof fa === "object" &&
    "id" in fa &&
    typeof (fa as { id: unknown }).id !== "function"
  );
}

export function useExecuteProgram(): UseExecuteProgramResult {
  const { client, isReady, sync, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;
  const isBusyRef = useRef(false);

  const [result, setResult] = useState<ExecuteProgramResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const execute = useCallback(
    async (options: ExecuteProgramOptions): Promise<ExecuteProgramResult> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      if (isBusyRef.current) {
        throw new MidenError(
          "A program execution is already in progress. Await the previous call before starting another.",
          { code: "OPERATION_BUSY" }
        );
      }

      isBusyRef.current = true;
      setIsLoading(true);
      setError(null);

      try {
        if (!options.skipSync) {
          await sync();
        }

        const programResult = await runExclusiveSafe(async () => {
          const accountIdObj = parseAccountId(options.accountId);

          const adviceInputs = options.adviceInputs ?? new AdviceInputs();

          let foreignAccountsArray: ForeignAccountArray;
          if (options.foreignAccounts?.length) {
            const accounts = options.foreignAccounts.map((fa) => {
              const wrapper = isForeignAccountWrapper(fa);
              const id = parseAccountId(wrapper ? fa.id : (fa as string));
              const storage =
                wrapper && fa.storage
                  ? fa.storage
                  : new AccountStorageRequirements();
              return ForeignAccount.public(id, storage);
            });
            foreignAccountsArray = new ForeignAccountArray(accounts);
          } else {
            foreignAccountsArray = new ForeignAccountArray();
          }

          const feltArray = await client.executeProgram(
            accountIdObj,
            options.script,
            adviceInputs,
            foreignAccountsArray
          );

          const stack: bigint[] = [];
          const len = feltArray.length();
          for (let i = 0; i < len; i++) {
            stack.push(feltArray.get(i).asInt());
          }

          return { stack };
        });

        setResult(programResult);
        return programResult;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
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
    setError(null);
  }, []);

  return {
    execute,
    result,
    isLoading,
    error,
    reset,
  };
}
