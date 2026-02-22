import { NoteFilterTypes, TransactionFilter } from "@miden-sdk/miden-sdk";
import type { TransactionId } from "@miden-sdk/miden-sdk";

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
 */
export async function waitForTransactionCommit(
  client: ClientWithTransactions,
  runExclusiveSafe: <T>(fn: () => Promise<T>) => Promise<T>,
  txId: TransactionId,
  maxWaitMs = 10_000,
  delayMs = 1_000
) {
  let waited = 0;

  while (waited < maxWaitMs) {
    await runExclusiveSafe(() => client.syncState());
    const [record] = await runExclusiveSafe(() =>
      client.getTransactions(TransactionFilter.ids([txId]))
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
    waited += delayMs;
  }

  throw new Error("Timeout waiting for transaction commit");
}

export type { ClientWithTransactions };
