import { TransactionProver } from "@miden-sdk/miden-sdk";
import type { MidenConfig, ProverConfig } from "../types";

const DEFAULT_PROVER_URLS = {
  devnet: "https://tx-prover.devnet.miden.io",
  testnet: "https://tx-prover.testnet.miden.io",
};

type ProverConfigSubset = Pick<
  MidenConfig,
  "prover" | "proverUrls" | "proverTimeoutMs"
>;

export function resolveTransactionProver(
  config: ProverConfigSubset
): TransactionProver | null {
  const { prover } = config;
  if (!prover) {
    return null;
  }

  if (typeof prover === "string") {
    const normalized = prover.trim().toLowerCase();
    if (normalized === "local") {
      return TransactionProver.newLocalProver();
    }
    if (normalized === "devnet" || normalized === "testnet") {
      const url =
        config.proverUrls?.[normalized] ??
        DEFAULT_PROVER_URLS[normalized] ??
        null;
      if (!url) {
        throw new Error(`Missing ${normalized} prover URL`);
      }
      return TransactionProver.newRemoteProver(
        url,
        normalizeTimeout(config.proverTimeoutMs)
      );
    }
    return TransactionProver.newRemoteProver(
      prover,
      normalizeTimeout(config.proverTimeoutMs)
    );
  }

  return createRemoteProver(prover, config.proverTimeoutMs);
}

function createRemoteProver(
  config: Extract<ProverConfig, { url: string }>,
  fallbackTimeout?: number | bigint
): TransactionProver {
  const { url, timeoutMs } = config;
  if (!url) {
    throw new Error("Remote prover requires a URL");
  }
  return TransactionProver.newRemoteProver(
    url,
    normalizeTimeout(timeoutMs ?? fallbackTimeout)
  );
}

function normalizeTimeout(
  timeoutMs?: number | bigint
): bigint | null | undefined {
  if (timeoutMs === undefined) {
    return undefined;
  }
  if (timeoutMs === null) {
    return null;
  }
  return typeof timeoutMs === "bigint" ? timeoutMs : BigInt(timeoutMs);
}
