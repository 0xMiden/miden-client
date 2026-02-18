import { useCallback, useEffect } from "react";
import { useMiden } from "../context/MidenProvider";
import { useMidenStore, useAccountsStore } from "../store/MidenStore";
import type { AccountHeader } from "@miden-sdk/miden-sdk";
import type { AccountsResult } from "../types";
import { isFaucetId } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";

/**
 * Hook to list all accounts in the client.
 *
 * @example
 * ```tsx
 * function AccountList() {
 *   const { accounts, wallets, faucets, isLoading } = useAccounts();
 *
 *   if (isLoading) return <div>Loading...</div>;
 *
 *   return (
 *     <div>
 *       <h2>Wallets ({wallets.length})</h2>
 *       {wallets.map(w => <div key={w.id().toString()}>{w.id().toString()}</div>)}
 *
 *       <h2>Faucets ({faucets.length})</h2>
 *       {faucets.map(f => <div key={f.id().toString()}>{f.id().toString()}</div>)}
 *     </div>
 *   );
 * }
 * ```
 */
export function useAccounts(): AccountsResult {
  const { client, isReady, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;
  const accounts = useAccountsStore();
  const isLoadingAccounts = useMidenStore((state) => state.isLoadingAccounts);
  const setLoadingAccounts = useMidenStore((state) => state.setLoadingAccounts);
  const setAccounts = useMidenStore((state) => state.setAccounts);

  const refetch = useCallback(async () => {
    if (!client || !isReady) return;

    setLoadingAccounts(true);
    try {
      const fetchedAccounts = await runExclusiveSafe(() =>
        client.getAccounts()
      );
      setAccounts(fetchedAccounts);
    } catch (error) {
      console.error("Failed to fetch accounts:", error);
    } finally {
      setLoadingAccounts(false);
    }
  }, [client, isReady, runExclusive, setAccounts, setLoadingAccounts]);

  // Initial fetch
  useEffect(() => {
    if (isReady && accounts.length === 0) {
      refetch();
    }
  }, [isReady, accounts.length, refetch]);

  // Categorize accounts
  const wallets: AccountHeader[] = [];
  const faucets: AccountHeader[] = [];

  for (const account of accounts) {
    const accountId = account.id();
    // Check if account is a faucet based on account ID type
    // Faucet IDs have a specific bit pattern
    if (isFaucetId(accountId)) {
      faucets.push(account);
    } else {
      wallets.push(account);
    }
  }

  return {
    accounts,
    wallets,
    faucets,
    isLoading: isLoadingAccounts,
    error: null,
    refetch,
  };
}
