import { TransactionProver } from "@miden-sdk/miden-sdk";
import type { MidenConfig, ProverConfig, ProverTarget } from "../types";

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

  // Fallback config object — resolve the primary target
  if (isFallbackConfig(prover)) {
    return resolveProverTarget(prover.primary, config);
  }

  return resolveProverTarget(prover, config);
}

/**
 * Prove a transaction with automatic fallback.
 *
 * 1. Try the primary prover
 * 2. If it fails and a fallback is configured (and not disabled), retry with fallback
 * 3. Calls `onFallback` when falling back so the UI can notify the user
 */
export async function proveWithFallback<T>(
  proveFn: (prover: TransactionProver | undefined) => Promise<T>,
  config: ProverConfigSubset
): Promise<T> {
  const primaryProver = resolveTransactionProver(config);

  try {
    return await proveFn(primaryProver ?? undefined);
  } catch (primaryError) {
    const { prover } = config;
    if (!prover || !isFallbackConfig(prover) || !prover.fallback) {
      throw primaryError;
    }

    if (prover.disableFallback?.()) {
      throw primaryError;
    }

    const fallbackProver = resolveProverTarget(prover.fallback, config);
    prover.onFallback?.();

    return await proveFn(fallbackProver ?? undefined);
  }
}

function isFallbackConfig(
  prover: ProverConfig
): prover is Extract<ProverConfig, { primary: unknown }> {
  return typeof prover === "object" && "primary" in prover;
}

function resolveProverTarget(
  target: ProverTarget,
  config: ProverConfigSubset
): TransactionProver | null {
  if (typeof target === "string") {
    const normalized = target.trim().toLowerCase();
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
      target,
      normalizeTimeout(config.proverTimeoutMs)
    );
  }

  return createRemoteProver(target, config.proverTimeoutMs);
}

function createRemoteProver(
  config: { url: string; timeoutMs?: number | bigint },
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
