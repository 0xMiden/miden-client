import {
  NoteFilterTypes,
  NoteType,
  TransactionFilter,
} from "@miden-sdk/miden-sdk";

/**
 * Map a status string to the corresponding NoteFilterTypes enum value.
 * Shared between useNotes and useNoteStream.
 */
export function getNoteFilterType(
  status?: "all" | "consumed" | "committed" | "expected" | "processing"
): NoteFilterTypes {
  switch (status) {
    case "consumed":
      return NoteFilterTypes.Consumed;
    case "committed":
      return NoteFilterTypes.Committed;
    case "expected":
      return NoteFilterTypes.Expected;
    case "processing":
      return NoteFilterTypes.Processing;
    case "all":
    default:
      return NoteFilterTypes.All;
  }
}

/**
 * Map a note type string to the corresponding NoteType enum value.
 * Shared across hooks that create transactions (useSend, useMultiSend, etc.).
 */
export function getNoteType(type: "private" | "public"): NoteType {
  switch (type) {
    case "private":
      return NoteType.Private;
    case "public":
      return NoteType.Public;
    default:
      return NoteType.Private;
  }
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

/**
 * Poll until a transaction is committed or discarded.
 * Shared between useSend and useMultiSend.
 *
 * Accepts a hex string rather than a TransactionId WASM object because
 * applyTransaction may invalidate all child WASM pointers (including
 * TransactionId). Using a plain string avoids use-after-free.
 */
export async function waitForTransactionCommit(
  client: ClientWithTransactions,
  runExclusiveSafe: <T>(fn: () => Promise<T>) => Promise<T>,
  txIdHex: string,
  maxWaitMs = 10_000,
  delayMs = 1_000
) {
  const deadline = Date.now() + maxWaitMs;
  const targetHex = normalizeHex(txIdHex);

  while (Date.now() < deadline) {
    await runExclusiveSafe(() => client.syncState());
    const records = await runExclusiveSafe(() =>
      client.getTransactions(TransactionFilter.all())
    );
    const record = records.find(
      (r) => normalizeHex(r.id().toHex()) === targetHex
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
    await new Promise((resolve) => setTimeout(resolve, delayMs));
  }

  throw new Error("Timeout waiting for transaction commit");
}

function normalizeHex(value: string): string {
  const trimmed = value.trim();
  const normalized =
    trimmed.startsWith("0x") || trimmed.startsWith("0X")
      ? trimmed
      : `0x${trimmed}`;
  return normalized.toLowerCase();
}

export type { ClientWithTransactions };
