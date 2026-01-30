import { useEffect, useMemo } from "react";
import {
  AccountId,
  BasicFungibleFaucetComponent,
  Endpoint,
  RpcClient,
} from "@miden-sdk/miden-sdk";
import { useMiden } from "../context/MidenProvider";
import { useAssetMetadataStore, useMidenStore } from "../store/MidenStore";
import type { AssetMetadata } from "../types";
import { parseAccountId } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";

const inflight = new Map<string, Promise<void>>();
const rpcClients = new Map<string, RpcClient>();

const getRpcClient = (rpcUrl?: string): RpcClient | null => {
  const key = rpcUrl ?? "__default__";
  const existing = rpcClients.get(key);
  if (existing) return existing;

  try {
    const endpoint = rpcUrl ? new Endpoint(rpcUrl) : Endpoint.testnet();
    const client = new RpcClient(endpoint);
    rpcClients.set(key, client);
    return client;
  } catch {
    return null;
  }
};

const fetchAssetMetadata = async (
  client: { getAccount: (id: AccountId) => Promise<unknown> },
  rpcClient: RpcClient | null,
  assetId: string
): Promise<AssetMetadata | null> => {
  try {
    const accountId = parseAccountId(assetId);
    let account = (await client.getAccount(accountId)) as unknown;

    if (!account && rpcClient) {
      try {
        const fetched = await rpcClient.getAccountDetails(accountId);
        account = fetched.account?.();
      } catch {
        // Ignore RPC failures; fallback to null.
      }
    }

    if (!account) return null;

    const faucet = BasicFungibleFaucetComponent.fromAccount(account as never);
    const symbol = faucet.symbol().toString();
    const decimals = faucet.decimals();

    return { assetId, symbol, decimals };
  } catch {
    return null;
  }
};

export function useAssetMetadata(assetIds: string[] = []) {
  const { client, isReady, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;
  const assetMetadata = useAssetMetadataStore();
  const setAssetMetadata = useMidenStore((state) => state.setAssetMetadata);
  const rpcUrl = useMidenStore((state) => state.config.rpcUrl);
  const rpcClient = useMemo(() => getRpcClient(rpcUrl), [rpcUrl]);

  const uniqueAssetIds = useMemo(
    () => Array.from(new Set(assetIds.filter(Boolean))),
    [assetIds]
  );

  useEffect(() => {
    if (!client || !isReady || uniqueAssetIds.length === 0) return;

    uniqueAssetIds.forEach((assetId) => {
      const existing = assetMetadata.get(assetId);
      const hasMetadata =
        existing?.symbol !== undefined || existing?.decimals !== undefined;
      if (hasMetadata || inflight.has(assetId)) return;

      const promise = runExclusiveSafe(async () => {
        const metadata = await fetchAssetMetadata(
          client as never,
          rpcClient,
          assetId
        );
        setAssetMetadata(assetId, metadata ?? { assetId });
      }).finally(() => {
        inflight.delete(assetId);
      });

      inflight.set(assetId, promise);
    });
  }, [
    client,
    isReady,
    uniqueAssetIds,
    assetMetadata,
    runExclusiveSafe,
    setAssetMetadata,
    rpcClient,
  ]);

  return { assetMetadata };
}
