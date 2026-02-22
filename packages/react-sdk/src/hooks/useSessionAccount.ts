import { useCallback, useEffect, useRef, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import { useMidenStore } from "../store/MidenStore";
import { AccountStorageMode } from "@miden-sdk/miden-sdk";
import type {
  UseSessionAccountOptions,
  UseSessionAccountReturn,
  SessionAccountStep,
} from "../types";
import { DEFAULTS } from "../types";
import { parseAccountId } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";
import { ensureAccountBech32 } from "../utils/accountBech32";
import { MidenError } from "../utils/errors";

/**
 * Hook to manage a session wallet lifecycle: create -> fund -> consume.
 *
 * Replaces the common 300+ line pattern of creating a temporary wallet,
 * waiting for funding, and consuming the funding note.
 *
 * @example
 * ```tsx
 * function SessionWallet() {
 *   const { initialize, sessionAccountId, isReady, step, error, reset } =
 *     useSessionAccount({
 *       fund: async (sessionId) => {
 *         // Send tokens from main wallet to session wallet
 *         await send({ from: mainWalletId, to: sessionId, assetId, amount: 100n });
 *       },
 *       assetId: "0x...",
 *     });
 *
 *   if (error) return <div>Error: {error.message} <button onClick={reset}>Retry</button></div>;
 *   if (isReady) return <div>Session ready: {sessionAccountId}</div>;
 *
 *   return (
 *     <button onClick={initialize} disabled={step !== "idle"}>
 *       {step === "idle" ? "Start Session" : step}
 *     </button>
 *   );
 * }
 * ```
 */
export function useSessionAccount(
  options: UseSessionAccountOptions
): UseSessionAccountReturn {
  const { client, isReady, sync, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;
  const setAccounts = useMidenStore((state) => state.setAccounts);

  const [sessionAccountId, setSessionAccountId] = useState<string | null>(null);
  const [step, setStep] = useState<SessionAccountStep>("idle");
  const [error, setError] = useState<Error | null>(null);
  const cancelledRef = useRef(false);
  const isBusyRef = useRef(false);

  const storagePrefix = options.storagePrefix ?? "miden-session";
  const pollIntervalMs = options.pollIntervalMs ?? 3000;
  const maxWaitMs = options.maxWaitMs ?? 60_000;

  // Destructure walletOptions primitives so useCallback deps are stable
  const storageMode = options.walletOptions?.storageMode ?? "public";
  const mutable = options.walletOptions?.mutable ?? DEFAULTS.WALLET_MUTABLE;
  const authScheme = options.walletOptions?.authScheme ?? DEFAULTS.AUTH_SCHEME;

  // Store fund in a ref so the callback identity doesn't change when the
  // caller passes an inline function (the common case).
  const fundRef = useRef(options.fund);
  fundRef.current = options.fund;

  // Restore persisted session on mount
  useEffect(() => {
    try {
      const stored = localStorage.getItem(`${storagePrefix}:accountId`);
      const storedReady = localStorage.getItem(`${storagePrefix}:ready`);
      if (stored && stored.length > 0) {
        // Validate the stored ID is parseable before restoring
        const validationId = parseAccountId(stored);
        (validationId as { free?: () => void })?.free?.();
        setSessionAccountId(stored);
        if (storedReady === "true") {
          setStep("ready");
        }
      }
    } catch {
      // Stored data is invalid — clear it and start fresh
      localStorage.removeItem(`${storagePrefix}:accountId`);
      localStorage.removeItem(`${storagePrefix}:ready`);
    }
  }, [storagePrefix]);

  const initialize = useCallback(async () => {
    if (!client || !isReady) {
      throw new Error("Miden client is not ready");
    }

    if (isBusyRef.current) {
      throw new MidenError(
        "Session account initialization is already in progress.",
        { code: "OPERATION_BUSY" }
      );
    }

    isBusyRef.current = true;
    cancelledRef.current = false;
    setError(null);

    try {
      // Step 1: Create wallet (or use persisted one)
      let walletId = sessionAccountId;

      if (!walletId) {
        setStep("creating");

        const resolvedStorageMode = getStorageMode(storageMode);

        const wallet = await runExclusiveSafe(async () => {
          const w = await client.newWallet(
            resolvedStorageMode,
            mutable,
            authScheme
          );
          ensureAccountBech32(w);
          const accounts = await client.getAccounts();
          setAccounts(accounts);
          return w;
        });

        if (cancelledRef.current) return;

        walletId = wallet.id().toString();
        setSessionAccountId(walletId);
        localStorage.setItem(`${storagePrefix}:accountId`, walletId);
      }

      // Step 2: Fund the session wallet
      setStep("funding");
      await fundRef.current(walletId);

      if (cancelledRef.current) return;

      // Step 3: Wait for funding note and consume it
      setStep("consuming");
      await waitAndConsume(
        client as unknown as WaitAndConsumeClient,
        runExclusiveSafe,
        walletId,
        pollIntervalMs,
        maxWaitMs,
        cancelledRef
      );

      if (cancelledRef.current) return;

      // Done
      setStep("ready");
      localStorage.setItem(`${storagePrefix}:ready`, "true");

      await sync();
    } catch (err) {
      if (!cancelledRef.current) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        setStep("idle");
        throw error;
      }
    } finally {
      isBusyRef.current = false;
    }
  }, [
    client,
    isReady,
    sync,
    runExclusiveSafe,
    sessionAccountId,
    storageMode,
    mutable,
    authScheme,
    storagePrefix,
    pollIntervalMs,
    maxWaitMs,
    setAccounts,
  ]);

  const reset = useCallback(() => {
    cancelledRef.current = true;
    isBusyRef.current = false;
    setSessionAccountId(null);
    setStep("idle");
    setError(null);
    localStorage.removeItem(`${storagePrefix}:accountId`);
    localStorage.removeItem(`${storagePrefix}:ready`);
  }, [storagePrefix]);

  return {
    initialize,
    sessionAccountId,
    isReady: step === "ready",
    step,
    error,
    reset,
  };
}

function getStorageMode(
  mode: "private" | "public"
): ReturnType<typeof AccountStorageMode.private> {
  switch (mode) {
    case "private":
      return AccountStorageMode.private();
    case "public":
      return AccountStorageMode.public();
    default:
      return AccountStorageMode.public();
  }
}

type WaitAndConsumeClient = {
  syncState: () => Promise<unknown>;
  getConsumableNotes: (
    accountId?: unknown
  ) => Promise<Array<{ inputNoteRecord: () => { toNote: () => unknown } }>>;
  newConsumeTransactionRequest: (notes: unknown[]) => unknown;
  submitNewTransaction: (
    accountId: unknown,
    request: unknown
  ) => Promise<unknown>;
};

async function waitAndConsume(
  client: WaitAndConsumeClient,
  runExclusiveSafe: <T>(fn: () => Promise<T>) => Promise<T>,
  walletId: string,
  pollIntervalMs: number,
  maxWaitMs: number,
  cancelledRef: { current: boolean }
) {
  const deadline = Date.now() + maxWaitMs;

  while (Date.now() < deadline) {
    if (cancelledRef.current) return;

    await runExclusiveSafe(() =>
      (client as { syncState: () => Promise<unknown> }).syncState()
    );

    if (cancelledRef.current) return;

    // Fetch and consume in a single exclusive block to avoid WASM pointer
    // staleness between separate runExclusiveSafe calls.
    const consumed = await runExclusiveSafe(async () => {
      const accountIdObj = parseAccountId(walletId);
      const consumable = await client.getConsumableNotes(accountIdObj);
      if (consumable.length === 0) return false;
      const notes = consumable.map((c) => c.inputNoteRecord().toNote());
      const txRequest = client.newConsumeTransactionRequest(notes);
      const freshAccountId = parseAccountId(walletId);
      await client.submitNewTransaction(freshAccountId, txRequest);
      return true;
    });

    if (consumed) return;

    await new Promise((resolve) => setTimeout(resolve, pollIntervalMs));
  }

  throw new Error("Timeout waiting for session wallet funding");
}
