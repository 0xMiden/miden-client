import { useCallback, useEffect, useState, useMemo } from "react";
import { useMiden } from "../context/MidenProvider";
import { useMidenStore } from "../store/MidenStore";
import { AccountId } from "@miden-sdk/miden-sdk";
import type { AccountResult, AssetBalance } from "../types";
import { runExclusiveDirect } from "../utils/runExclusive";

/**
 * Hook to get details for a single account.
 *
 * @param accountId - The account ID string or AccountId object
 *
 * @example
 * ```tsx
 * function AccountDetails({ accountId }: { accountId: string }) {
 *   const { account, assets, getBalance, isLoading } = useAccount(accountId);
 *
 *   if (isLoading) return <div>Loading...</div>;
 *   if (!account) return <div>Account not found</div>;
 *
 *   return (
 *     <div>
 *       <h2>Account: {account.id().toString()}</h2>
 *       <p>Nonce: {account.nonce().toString()}</p>
 *       <h3>Assets</h3>
 *       {assets.map(a => (
 *         <div key={a.faucetId}>
 *           {a.faucetId}: {a.amount.toString()}
 *         </div>
 *       ))}
 *       <p>USDC Balance: {getBalance('0x...').toString()}</p>
 *     </div>
 *   );
 * }
 * ```
 */
export function useAccount(
  accountId: string | AccountId | undefined
): AccountResult {
  const { client, isReady, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;
  const accountDetails = useMidenStore((state) => state.accountDetails);
  const setAccountDetails = useMidenStore((state) => state.setAccountDetails);

  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  // Normalize accountId to string
  const accountIdStr = useMemo(() => {
    if (!accountId) return undefined;
    if (typeof accountId === "string") return accountId;
    // AccountId object - convert to string
    if (typeof (accountId as AccountId).toString === "function") {
      return (accountId as AccountId).toString();
    }
    return String(accountId);
  }, [accountId]);

  // Get cached account
  const account = accountIdStr
    ? (accountDetails.get(accountIdStr) ?? null)
    : null;

  const refetch = useCallback(async () => {
    if (!client || !isReady || !accountIdStr) return;

    setIsLoading(true);
    setError(null);

    try {
      const accountIdObj = AccountId.fromHex(accountIdStr);
      const fetchedAccount = await runExclusiveSafe(() =>
        client.getAccount(accountIdObj)
      );
      if (fetchedAccount) {
        setAccountDetails(accountIdStr, fetchedAccount);
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setIsLoading(false);
    }
  }, [client, isReady, runExclusive, accountIdStr, setAccountDetails]);

  // Initial fetch
  useEffect(() => {
    if (isReady && accountIdStr && !account) {
      refetch();
    }
  }, [isReady, accountIdStr, account, refetch]);

  // Extract assets from account vault
  const assets = useMemo((): AssetBalance[] => {
    if (!account) return [];

    try {
      const vault = account.vault();
      const assetsList: AssetBalance[] = [];

      // Get fungible assets from vault
      const vaultAssets = vault.fungibleAssets();
      for (const asset of vaultAssets) {
        assetsList.push({
          faucetId: asset.faucetId().toString(),
          amount: asset.amount(),
        });
      }

      return assetsList;
    } catch {
      return [];
    }
  }, [account]);

  // Helper to get balance for a specific faucet
  const getBalance = useCallback(
    (faucetId: string): bigint => {
      const asset = assets.find((a) => a.faucetId === faucetId);
      return asset?.amount ?? 0n;
    },
    [assets]
  );

  return {
    account,
    assets,
    isLoading,
    error,
    refetch,
    getBalance,
  };
}
