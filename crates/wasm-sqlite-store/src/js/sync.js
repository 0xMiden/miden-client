/**
 * Sync operations for the WASM SQLite store.
 * Handles note tags, sync height, and state sync updates.
 */
import { getDatabase } from "./schema.js";
import { logError, uint8ArrayToBase64 } from "./utils.js";
import { upsertInputNote, upsertOutputNote } from "./notes.js";
import {
  upsertAccountCode,
  upsertAccountStorage,
  upsertStorageMapEntries,
  upsertVaultAssets,
  upsertAccountRecord,
  undoAccountStates,
  lockAccount,
} from "./accounts.js";
import { upsertTransactionRecord } from "./transactions.js";
import {
  insertBlockHeader,
  insertPartialBlockchainNodes,
} from "./chainData.js";
function toBase64(data) {
  if (!data) return "";
  const bytes = data instanceof Uint8Array ? data : new Uint8Array(data);
  return uint8ArrayToBase64(bytes);
}
export function getNoteTags(dbId) {
  try {
    const db = getDatabase(dbId);
    const rows = db.all("SELECT tag, source FROM tags");
    return rows.map((row) => ({
      tag: toBase64(row.tag),
      source: toBase64(row.source),
    }));
  } catch (error) {
    logError(error, "Error fetching note tags");
    return [];
  }
}
export function getSyncHeight(dbId) {
  try {
    const db = getDatabase(dbId);
    const row = db.get(
      "SELECT block_num FROM state_sync ORDER BY block_num DESC LIMIT 1"
    );
    return row ? { blockNum: row.block_num } : null;
  } catch (error) {
    logError(error, "Error fetching sync height");
    return null;
  }
}
export function addNoteTag(dbId, tag, source) {
  try {
    const db = getDatabase(dbId);
    // Check if this exact tag+source combination already exists
    const existing = db.get(
      "SELECT tag FROM tags WHERE tag = ? AND source = ?",
      [tag, source]
    );
    if (existing) {
      return false; // Tag already tracked
    }
    db.run("INSERT INTO tags (tag, source) VALUES (?, ?)", [tag, source]);
    return true;
  } catch (error) {
    logError(error, "Error adding note tag");
    return false;
  }
}
export function removeNoteTag(dbId, tag, source) {
  try {
    const db = getDatabase(dbId);
    const result = db.run("DELETE FROM tags WHERE tag = ? AND source = ?", [
      tag,
      source,
    ]);
    return result.changes;
  } catch (error) {
    logError(error, "Error removing note tag");
    return 0;
  }
}
/**
 * Applies a state sync update atomically.
 * This is the most complex operation as it updates multiple tables.
 */
export function applyStateSync(
  dbId,
  blockNum,
  // Block updates
  blockHeaders,
  nodeIds,
  nodeValues,
  // Note updates
  inputNotes,
  outputNotes,
  // Transaction updates
  transactionUpdates,
  // Account updates
  accountUpdates,
  // Tags to remove
  tagsToRemove,
  // Account states to undo
  accountStatesToUndo,
  // Accounts to lock
  accountsToLock
) {
  try {
    const db = getDatabase(dbId);
    db.transaction(() => {
      // 1. Update sync height
      db.run("UPDATE state_sync SET block_num = ? WHERE block_num < ?", [
        blockNum,
        blockNum,
      ]);
      // 2. Insert block headers
      for (const bh of blockHeaders) {
        insertBlockHeader(
          dbId,
          bh.blockNum,
          bh.header,
          bh.peaks,
          bh.hasRelevantNotes
        );
      }
      // 3. Insert authentication nodes
      if (nodeIds.length > 0) {
        insertPartialBlockchainNodes(dbId, nodeIds, nodeValues);
      }
      // 4. Upsert input notes
      for (const note of inputNotes) {
        upsertInputNote(
          dbId,
          note.noteId,
          note.assets,
          note.serialNumber,
          note.inputs,
          note.scriptRoot,
          note.serializedNoteScript,
          note.nullifier,
          note.createdAt,
          note.stateDiscriminant,
          note.state
        );
      }
      // 5. Upsert output notes
      for (const note of outputNotes) {
        upsertOutputNote(
          dbId,
          note.noteId,
          note.assets,
          note.recipientDigest,
          note.metadata,
          note.nullifier,
          note.expectedHeight,
          note.stateDiscriminant,
          note.state
        );
      }
      // 6. Update transactions
      for (const tx of transactionUpdates) {
        upsertTransactionRecord(
          dbId,
          tx.id,
          tx.details,
          tx.blockNum,
          tx.statusVariant,
          tx.status,
          tx.scriptRoot
        );
      }
      // 7. Undo account states
      if (accountStatesToUndo.length > 0) {
        undoAccountStates(dbId, accountStatesToUndo);
      }
      // 8. Update accounts
      for (const update of accountUpdates) {
        upsertAccountCode(dbId, update.codeCommitment, update.code);
        upsertAccountStorage(dbId, update.storageSlots);
        upsertStorageMapEntries(dbId, update.storageMapEntries);
        upsertVaultAssets(dbId, update.assets);
        upsertAccountRecord(
          dbId,
          update.accountId,
          update.codeCommitment,
          update.storageCommitment,
          update.vaultRoot,
          update.nonce,
          update.committed,
          update.accountCommitment,
          update.accountSeed
        );
      }
      // 9. Remove tags
      for (const tag of tagsToRemove) {
        db.run("DELETE FROM tags WHERE tag = ? AND source = ?", [
          tag.tag,
          tag.source,
        ]);
      }
      // 10. Lock accounts with mismatched digests
      for (const acctLock of accountsToLock) {
        lockAccount(dbId, acctLock.accountId);
      }
    });
  } catch (error) {
    logError(error, "Error applying state sync");
  }
}
export function discardTransactions(dbId, transactionIds) {
  try {
    if (transactionIds.length === 0) return;
    const db = getDatabase(dbId);
    const placeholders = transactionIds.map(() => "?").join(",");
    // Status variant 3 = Discarded
    db.run(
      `UPDATE transactions SET status_variant = 3 WHERE id IN (${placeholders})`,
      transactionIds
    );
  } catch (error) {
    logError(error, "Error discarding transactions");
  }
}
