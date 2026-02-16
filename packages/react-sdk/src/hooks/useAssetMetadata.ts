import { useEffect, useMemo } from "react";
import {
  BasicFungibleFaucetComponent,
  Endpoint,
  RpcClient,
} from "@miden-sdk/miden-sdk";
import { useAssetMetadataStore, useMidenStore } from "../store/MidenStore";
import type { AssetMetadata } from "../types";
import { parseAccountId } from "../utils/accountParsing";

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
  rpcClient: RpcClient,
  assetId: string
): Promise<AssetMetadata | null> => {
  try {
    const accountId = parseAccountId(assetId);
    const fetched = await rpcClient.getAccountDetails(accountId);
    const account = fetched.account?.();

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
  const assetMetadata = useAssetMetadataStore();
  const setAssetMetadata = useMidenStore((state) => state.setAssetMetadata);
  const rpcUrl = useMidenStore((state) => state.config.rpcUrl);
  const rpcClient = useMemo(() => getRpcClient(rpcUrl), [rpcUrl]);

  const uniqueAssetIds = useMemo(
    () => Array.from(new Set(assetIds.filter(Boolean))),
    [assetIds]
  );

  useEffect(() => {
    if (!rpcClient || uniqueAssetIds.length === 0) return;

    uniqueAssetIds.forEach((assetId) => {
      const existing = assetMetadata.get(assetId);
      const hasMetadata =
        existing?.symbol !== undefined || existing?.decimals !== undefined;
      if (hasMetadata || inflight.has(assetId)) return;

      const promise = fetchAssetMetadata(rpcClient, assetId)
        .then((metadata) => {
          setAssetMetadata(assetId, metadata ?? { assetId });
        })
        .finally(() => {
          inflight.delete(assetId);
        });

      inflight.set(assetId, promise);
    });
  }, [uniqueAssetIds, assetMetadata, setAssetMetadata, rpcClient]);

  return { assetMetadata };
}
