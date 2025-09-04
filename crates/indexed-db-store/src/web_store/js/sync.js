import {
  db,
  stateSync,
  inputNotes,
  outputNotes,
  transactions,
  blockHeaders,
  partialBlockchainNodes,
  tags,
} from "./schema.js";
import {
  upsertTransactionRecord,
  insertTransactionScript,
} from "./transactions.js";
import { upsertInputNote, upsertOutputNote } from "./notes.js";
import {
  insertAccountStorage,
  insertAccountAssetVault,
  insertAccountRecord,
} from "./accounts.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";
export async function getNoteTags() {
  try {
    let records = await tags.toArray();
    let processedRecords = records.map((record) => {
      record.sourceNoteId =
        record.sourceNoteId == "" ? undefined : record.sourceNoteId;
      record.sourceAccountId =
        record.sourceAccountId == "" ? undefined : record.sourceAccountId;
      return record;
    });
    return processedRecords;
  } catch (error) {
    logWebStoreError(error, "Error fetch tag record");
  }
}
export async function getSyncHeight() {
  try {
    const record = await stateSync.get(1); // Since id is the primary key and always 1
    if (record) {
      let data = {
        blockNum: record.blockNum,
      };
      return data;
    } else {
      return null;
    }
  } catch (error) {
    logWebStoreError(error, "Error fetching sync height");
  }
}
export async function addNoteTag(tag, sourceNoteId, sourceAccountId) {
  try {
    let tagArray = new Uint8Array(tag);
    let tagBase64 = uint8ArrayToBase64(tagArray);
    await tags.add({
      tag: tagBase64,
      sourceNoteId: sourceNoteId ? sourceNoteId : "",
      sourceAccountId: sourceAccountId ? sourceAccountId : "",
    });
  } catch (error) {
    logWebStoreError(error, "Failed to add note tag");
  }
}
export async function removeNoteTag(tag, sourceNoteId, sourceAccountId) {
  try {
    let tagArray = new Uint8Array(tag);
    let tagBase64 = uint8ArrayToBase64(tagArray);
    return await tags
      .where({
        tag: tagBase64,
        sourceNoteId: sourceNoteId ? sourceNoteId : "",
        sourceAccountId: sourceAccountId ? sourceAccountId : "",
      })
      .delete();
  } catch (error) {
    logWebStoreError(error, "Failed to remove note tag");
  }
}
/*
 * Takes a `JsStateSyncUpdate` object and writes the state update into the store.
 * @param {JsStateSyncUpdate}
 */
export async function applyStateSync(stateUpdate) {
  const {
    blockNum, // Target block number for this sync
    flattenedNewBlockHeaders, // Serialized block headers to be reconstructed
    flattenedPartialBlockChainPeaks, // Serialized blockchain peaks for verification
    newBlockNums, // Block numbers corresponding to new headers
    blockHasRelevantNotes, // Flags indicating which blocks have relevant notes
    serializedNodeIds, // IDs for new authentication nodes
    serializedNodes, // Authentication node data for merkle proofs
    committedNoteIds, // Note tags to be cleaned up/removed
    serializedInputNotes, // Input notes consumed in transactions
    serializedOutputNotes, // Output notes created in transactions
    accountUpdates, // Account state changes
    transactionUpdates, // Transaction records and scripts
  } = stateUpdate;
  // Block headers and Blockchain peaks are flattened before calling
  // this function, here we rebuild them.
  const newBlockHeaders = reconstructFlattenedVec(flattenedNewBlockHeaders);
  const partialBlockchainPeaks = reconstructFlattenedVec(
    flattenedPartialBlockChainPeaks
  );
  // Create promises to insert each input note. Each note will have its own transaction,
  // and therefore, nested inside the final transaction inside this function.
  let inputNotesWriteOp = Promise.all(
    serializedInputNotes.map((note) => {
      return upsertInputNote(
        note.noteId,
        note.noteAssets,
        note.serialNumber,
        note.inputs,
        note.noteScriptRoot,
        note.noteScript,
        note.nullifier,
        note.createdAt,
        note.stateDiscriminant,
        note.state
      );
    })
  );
  // See comment above, the same thing applies here, but for Output Notes.
  let outputNotesWriteOp = Promise.all(
    serializedOutputNotes.map((note) => {
      return upsertOutputNote(
        note.noteId,
        note.noteAssets,
        note.recipientDigest,
        note.metadata,
        note.nullifier,
        note.expectedHeight,
        note.stateDiscriminant,
        note.state
      );
    })
  );
  // Promises to insert each transaction update.
  let transactionWriteOp = Promise.all(
    transactionUpdates.map((transactionRecord) => {
      let promises = [
        upsertTransactionRecord(
          transactionRecord.id,
          transactionRecord.details,
          transactionRecord.blockNum,
          transactionRecord.statusVariant,
          transactionRecord.status,
          transactionRecord.scriptRoot
        ),
      ];
      if (transactionRecord.scriptRoot && transactionRecord.txScript) {
        promises.push(
          insertTransactionScript(
            transactionRecord.scriptRoot,
            transactionRecord.txScript
          )
        );
      }
      return Promise.all(promises);
    })
  );
  // Promises to insert each account update.
  let accountUpdatesWriteOp = Promise.all(
    accountUpdates.flatMap((accountUpdate) => {
      return [
        insertAccountStorage(
          accountUpdate.storageRoot,
          accountUpdate.storageSlots
        ),
        insertAccountAssetVault(
          accountUpdate.assetVaultRoot,
          accountUpdate.assetBytes
        ),
        insertAccountRecord(
          accountUpdate.accountId,
          accountUpdate.codeRoot,
          accountUpdate.storageRoot,
          accountUpdate.assetVaultRoot,
          accountUpdate.nonce,
          accountUpdate.committed,
          accountUpdate.accountCommitment,
          accountUpdate.accountSeed
        ),
      ];
    })
  );
  const tablesToAccess = [
    stateSync,
    inputNotes,
    outputNotes,
    transactions,
    blockHeaders,
    partialBlockchainNodes,
    tags,
  ];
  // Write everything in a single transaction, this transaction will atomically do the operations
  // below, since every operation here (or at least, most of them), is done in a nested transaction.
  // For more information on this, check: https://dexie.org/docs/Dexie/Dexie.transaction()
  return await db.transaction("rw", tablesToAccess, async (tx) => {
    // Everything is under a single promise since otherwise the tx expires.
    await Promise.all([
      inputNotesWriteOp,
      outputNotesWriteOp,
      transactionWriteOp,
      accountUpdatesWriteOp,
      updateSyncHeight(tx, blockNum),
      updatePartialBlockchainNodes(tx, serializedNodeIds, serializedNodes),
      updateCommittedNoteTags(tx, committedNoteIds),
      Promise.all(
        newBlockHeaders.map((newBlockHeader, i) => {
          return updateBlockHeader(
            tx,
            newBlockNums[i],
            newBlockHeader,
            partialBlockchainPeaks[i],
            blockHasRelevantNotes[i] == 1
          );
        })
      ),
    ]);
  });
}
async function updateSyncHeight(tx, blockNum) {
  try {
    await tx.stateSync.update(1, { blockNum: blockNum });
  } catch (error) {
    logWebStoreError(error, "Failed to update sync height");
  }
}
async function updateBlockHeader(
  tx,
  blockNum,
  blockHeader,
  partialBlockchainPeaks,
  hasClientNotes
) {
  try {
    const headerBlob = new Blob([new Uint8Array(blockHeader)]);
    const partialBlockchainPeaksBlob = new Blob([
      new Uint8Array(partialBlockchainPeaks),
    ]);
    const data = {
      blockNum: blockNum,
      header: headerBlob,
      partialBlockchainPeaks: partialBlockchainPeaksBlob,
      hasClientNotes: hasClientNotes.toString(),
    };
    const existingBlockHeader = await tx.blockHeaders.get(blockNum);
    if (!existingBlockHeader) {
      await tx.blockHeaders.add(data);
    }
  } catch (err) {
    logWebStoreError(err, "Failed to insert block header");
  }
}
async function updatePartialBlockchainNodes(tx, nodeIndexes, nodes) {
  try {
    // Check if the arrays are not of the same length
    if (nodeIndexes.length !== nodes.length) {
      throw new Error(
        "nodeIndexes and nodes arrays must be of the same length"
      );
    }
    if (nodeIndexes.length === 0) {
      return;
    }
    // Create array of objects with id and node
    const data = nodes.map((node, index) => ({
      id: nodeIndexes[index],
      node: node,
    }));
    // Use bulkPut to add/overwrite the entries
    await tx.partialBlockchainNodes.bulkPut(data);
  } catch (err) {
    logWebStoreError(err, "Failed to update partial blockchain nodes");
  }
}
async function updateCommittedNoteTags(tx, inputNoteIds) {
  try {
    for (let i = 0; i < inputNoteIds.length; i++) {
      const noteId = inputNoteIds[i];
      // Remove note tags
      await tx.tags.where("source_note_id").equals(noteId).delete();
    }
  } catch (error) {
    logWebStoreError(error, "Failed to pudate committed note tags");
  }
}
// Helper function to reconstruct arrays from flattened data
function reconstructFlattenedVec(flattenedVec) {
  const data = flattenedVec.data();
  const lengths = flattenedVec.lengths();
  let index = 0;
  const result = [];
  lengths.forEach((length) => {
    result.push(data.slice(index, index + length));
    index += length;
  });
  return result;
}
//# sourceMappingURL=sync.js.map
