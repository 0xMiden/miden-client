import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import { useMidenStore } from "../store/MidenStore";
import { AccountStorageMode } from "@miden-sdk/miden-sdk";
import type { Account } from "@miden-sdk/miden-sdk";
import type { CreateWalletOptions } from "../types";
import { DEFAULTS } from "../types";
import { runExclusiveDirect } from "../utils/runExclusive";
import { ensureAccountBech32 } from "../utils/accountBech32";

export interface UseCreateWalletResult {
  /** Create a new wallet with optional configuration */
  createWallet: (options?: CreateWalletOptions) => Promise<Account>;
  /** The created wallet account */
  wallet: Account | null;
  /** Whether wallet creation is in progress */
  isCreating: boolean;
  /** Error if creation failed */
  error: Error | null;
  /** Reset the hook state */
  reset: () => void;
}

/**
 * Hook to create a new wallet account.
 *
 * @example
 * ```tsx
 * function CreateWalletButton() {
 *   const { createWallet, wallet, isCreating, error } = useCreateWallet();
 *
 *   const handleCreate = async () => {
 *     const newWallet = await createWallet({
 *       storageMode: 'private',
 *       mutable: true,
 *     });
 *     console.log('Created wallet:', newWallet.id().toString());
 *   };
 *
 *   return (
 *     <div>
 *       <button onClick={handleCreate} disabled={isCreating}>
 *         {isCreating ? 'Creating...' : 'Create Wallet'}
 *       </button>
 *       {wallet && <p>Created: {wallet.id().toString()}</p>}
 *       {error && <p>Error: {error.message}</p>}
 *     </div>
 *   );
 * }
 * ```
 */
export function useCreateWallet(): UseCreateWalletResult {
  const { client, isReady, sync, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;
  const setAccounts = useMidenStore((state) => state.setAccounts);

  const [wallet, setWallet] = useState<Account | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const createWallet = useCallback(
    async (options: CreateWalletOptions = {}): Promise<Account> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      await sync();

      setIsCreating(true);
      setError(null);

      try {
        const storageMode = getStorageMode(
          options.storageMode ?? DEFAULTS.STORAGE_MODE
        );
        const mutable = options.mutable ?? DEFAULTS.WALLET_MUTABLE;
        const authScheme = options.authScheme ?? DEFAULTS.AUTH_SCHEME;

        const newWallet = await runExclusiveSafe(async () => {
          const createdWallet = await client.newWallet(
            storageMode,
            mutable,
            authScheme,
            options.initSeed
          );
          ensureAccountBech32(createdWallet);
          const accounts = await client.getAccounts();
          setAccounts(accounts);
          return createdWallet;
        });

        setWallet(newWallet);

        return newWallet;
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
    setWallet(null);
    setIsCreating(false);
    setError(null);
  }, []);

  return {
    createWallet,
    wallet,
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
