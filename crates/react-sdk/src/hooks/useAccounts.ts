import { useCallback, useEffect } from "react";
import { useMiden } from "../context/MidenProvider";
import { useMidenStore, useAccountsStore } from "../store/MidenStore";
import type { AccountHeader } from "@demox-labs/miden-sdk";
import type { AccountsResult } from "../types";

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
  const { client, isReady } = useMiden();
  const accounts = useAccountsStore();
  const isLoadingAccounts = useMidenStore((state) => state.isLoadingAccounts);
  const setLoadingAccounts = useMidenStore((state) => state.setLoadingAccounts);
  const setAccounts = useMidenStore((state) => state.setAccounts);

  const refetch = useCallback(async () => {
    if (!client || !isReady) return;

    setLoadingAccounts(true);
    try {
      const fetchedAccounts = await client.getAccounts();
      setAccounts(fetchedAccounts);
    } catch (error) {
      console.error("Failed to fetch accounts:", error);
    } finally {
      setLoadingAccounts(false);
    }
  }, [client, isReady, setAccounts, setLoadingAccounts]);

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

/**
 * Helper to check if an account ID represents a faucet.
 * Faucet IDs have bits 61..=60 == 0b10 (type = Fungible Faucet)
 */
function isFaucetId(accountId: unknown): boolean {
  try {
    // The account ID has a toHex() method, and faucet IDs start with specific prefixes
    const hex =
      typeof (accountId as { toHex?: () => string }).toHex === "function"
        ? (accountId as { toHex: () => string }).toHex()
        : String(accountId);

    // Parse the first byte to check account type bits
    // Account type is in bits 61..60 of the u64:
    // 0b00 = Regular account (off-chain)
    // 0b01 = Regular account (on-chain)
    // 0b10 = Fungible faucet
    // 0b11 = Non-fungible faucet
    const firstByte = parseInt(hex.slice(0, 2), 16);
    const accountType = (firstByte >> 4) & 0b11;

    return accountType === 0b10 || accountType === 0b11;
  } catch {
    return false;
  }
}
