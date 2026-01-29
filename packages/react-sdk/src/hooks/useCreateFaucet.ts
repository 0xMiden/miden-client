import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import { useMidenStore } from "../store/MidenStore";
import { AccountStorageMode } from "@miden-sdk/miden-sdk";
import type { Account } from "@miden-sdk/miden-sdk";
import type { CreateFaucetOptions } from "../types";
import { DEFAULTS } from "../types";
import { runExclusiveDirect } from "../utils/runExclusive";

export interface UseCreateFaucetResult {
  /** Create a new faucet with the specified options */
  createFaucet: (options: CreateFaucetOptions) => Promise<Account>;
  /** The created faucet account */
  faucet: Account | null;
  /** Whether faucet creation is in progress */
  isCreating: boolean;
  /** Error if creation failed */
  error: Error | null;
  /** Reset the hook state */
  reset: () => void;
}

/**
 * Hook to create a new faucet account.
 *
 * @example
 * ```tsx
 * function CreateFaucetButton() {
 *   const { createFaucet, faucet, isCreating, error } = useCreateFaucet();
 *
 *   const handleCreate = async () => {
 *     const newFaucet = await createFaucet({
 *       tokenSymbol: 'TEST',
 *       decimals: 8,
 *       maxSupply: 1000000n * 10n ** 8n, // 1M tokens
 *     });
 *     console.log('Created faucet:', newFaucet.id().toString());
 *   };
 *
 *   return (
 *     <div>
 *       <button onClick={handleCreate} disabled={isCreating}>
 *         {isCreating ? 'Creating...' : 'Create Faucet'}
 *       </button>
 *       {faucet && <p>Created: {faucet.id().toString()}</p>}
 *       {error && <p>Error: {error.message}</p>}
 *     </div>
 *   );
 * }
 * ```
 */
export function useCreateFaucet(): UseCreateFaucetResult {
  const { client, isReady, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;
  const setAccounts = useMidenStore((state) => state.setAccounts);

  const [faucet, setFaucet] = useState<Account | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const createFaucet = useCallback(
    async (options: CreateFaucetOptions): Promise<Account> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      setIsCreating(true);
      setError(null);

      try {
        const storageMode = getStorageMode(
          options.storageMode ?? DEFAULTS.STORAGE_MODE
        );
        const decimals = options.decimals ?? DEFAULTS.FAUCET_DECIMALS;
        const authScheme = options.authScheme ?? DEFAULTS.AUTH_SCHEME;

        const newFaucet = await runExclusiveSafe(async () => {
          const createdFaucet = await client.newFaucet(
            storageMode,
            false, // nonFungible - currently only fungible faucets supported
            options.tokenSymbol,
            decimals,
            options.maxSupply,
            authScheme
          );
          const accounts = await client.getAccounts();
          setAccounts(accounts);
          return createdFaucet;
        });

        setFaucet(newFaucet);

        return newFaucet;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      } finally {
        setIsCreating(false);
      }
    },
    [client, isReady, runExclusive, setAccounts]
  );

  const reset = useCallback(() => {
    setFaucet(null);
    setIsCreating(false);
    setError(null);
  }, []);

  return {
    createFaucet,
    faucet,
    isCreating,
    error,
    reset,
  };
}

function getStorageMode(
  mode: "private" | "public" | "network"
): ReturnType<typeof AccountStorageMode.private> {
  switch (mode) {
    case "private":
      return AccountStorageMode.private();
    case "public":
      return AccountStorageMode.public();
    case "network":
      return AccountStorageMode.network();
    default:
      return AccountStorageMode.private();
  }
}
