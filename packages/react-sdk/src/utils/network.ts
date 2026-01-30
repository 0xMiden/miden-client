import type { RpcUrlConfig } from "../types";

const RPC_URLS = {
  testnet: "https://rpc.testnet.miden.io",
  devnet: "https://rpc.devnet.miden.io",
  localhost: "http://localhost:57291",
};

export function resolveRpcUrl(rpcUrl?: RpcUrlConfig): string | undefined {
  if (!rpcUrl) {
    return undefined;
  }

  const normalized = rpcUrl.trim().toLowerCase();
  if (normalized === "testnet") {
    return RPC_URLS.testnet;
  }
  if (normalized === "devnet") {
    return RPC_URLS.devnet;
  }
  if (normalized === "localhost" || normalized === "local") {
    return RPC_URLS.localhost;
  }

  return rpcUrl;
}
