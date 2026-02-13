import { useCallback } from "react";
import { useMiden } from "../context/MidenProvider";
import { TransactionFilter } from "@miden-sdk/miden-sdk";
import type { TransactionId } from "@miden-sdk/miden-sdk";
import type { WaitForCommitOptions } from "../types";

export interface UseWaitForCommitResult {
  /** Wait for a transaction to be committed on-chain */
  waitForCommit: (
    txId: string | TransactionId,
    options?: WaitForCommitOptions
  ) => Promise<void>;
}

type ClientWithTransactions = {
  syncState: () => Promise<unknown>;
  getTransactions: (filter: TransactionFilter) => Promise<
    Array<{
      id: () => { toHex: () => string };
      transactionStatus: () => {
        isPending: () => boolean;
        isCommitted: () => boolean;
        isDiscarded: () => boolean;
      };
    }>
  >;
};

export function useWaitForCommit(): UseWaitForCommitResult {
  const { client, isReady } = useMiden();

  const waitForCommit = useCallback(
    async (txId: string | TransactionId, options?: WaitForCommitOptions) => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      const timeoutMs = Math.max(0, options?.timeoutMs ?? 10_000);
      const intervalMs = Math.max(1, options?.intervalMs ?? 1_000);
      const targetHex = normalizeHex(
        typeof txId === "string" ? txId : txId.toHex()
      );

      let waited = 0;

      while (waited < timeoutMs) {
        await (client as ClientWithTransactions).syncState();

        const records = await (
          client as ClientWithTransactions
        ).getTransactions(
          typeof txId === "string"
            ? TransactionFilter.all()
            : TransactionFilter.ids([txId])
        );

        const record = records.find(
          (item) => normalizeHex(item.id().toHex()) === targetHex
        );

        if (record) {
          const status = record.transactionStatus();
          if (status.isCommitted()) {
            return;
          }
          if (status.isDiscarded()) {
            throw new Error("Transaction was discarded before commit");
          }
        }

        await new Promise((resolve) => setTimeout(resolve, intervalMs));
        waited += intervalMs;
      }

      throw new Error("Timeout waiting for transaction commit");
    },
    [client, isReady]
  );

  return { waitForCommit };
}

function normalizeHex(value: string): string {
  const trimmed = value.trim();
  const normalized =
    trimmed.startsWith("0x") || trimmed.startsWith("0X")
      ? trimmed
      : `0x${trimmed}`;
  return normalized.toLowerCase();
}
