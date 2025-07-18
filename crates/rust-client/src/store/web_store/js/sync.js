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

import { upsertTransactionRecord } from "./transactions.js";

import {
  insertAccountStorage,
  insertAccountAssetVault,
  insertAccountRecord,
} from "./accounts.js";

export async function getNoteTags() {
  try {
    let records = await tags.toArray();

    let processedRecords = records.map((record) => {
      record.sourceNoteId =
        record.sourceNoteId == "" ? null : record.sourceNoteId;
      record.sourceAccountId =
        record.sourceAccountId == "" ? null : record.sourceAccountId;
      return record;
    });

    return processedRecords;
  } catch (error) {
    console.error("Error fetching tag record:", error.toString());
    throw error;
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
    console.error("Error fetching sync height:", error.toString());
    throw error;
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
  } catch (err) {
    console.error("Failed to add note tag: ", err.toString());
    throw err;
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
  } catch (err) {
    console.log("Failed to remove note tag: ", err.toString());
    throw err;
  }
}

export async function receiveStateSync(stateSync) {}

export async function applyStateSync(stateUpdate) {
  const {
    blockNum,
    flattenedNewBlockHeaders,
    newBlockNums,
    blockHasRelevantNotes,
    serializedNodeIds,
    serializedNodes,
    noteTagsToRemove,
    serializedInputNotes,
    serializedOutputNotes,
    accountUpdates,
    transactionUpdates,
  } = stateUpdate;
  const newBlockHeaders = reconstructFlattenedVec(flattenedNewBlockHeaders);
  const partialBlockchainPeaks = reconstructFlattenedVec(
    flattenedPartialBlockchainPeaks
  );
  let inputNotesUpsertOp = serializedInputNotes.map((note) => {
    upsertInputNote(
      note.noteId,
      note.assets,
      note.serialNumber,
      note.inputs,
      note.noteScriptRoot,
      note.serializedNoteScript,
      note.nullifier,
      note.serializedCreatedAt,
      note.stateDiscriminant,
      note.state
    );
  });

  let outputNotesUpsertOp = serializedOutputNotes.map((note) => {
    outputNotesUpsertOp(
      note.noteId,
      note.assets,
      note.recipientDigest,
      note.metaData,
      note.nullifier,
      note.expectedHeight,
      note.stateDiscriminant,
      note.state
    );
  });

  let transactionWriteOp = transactionUpdates.map((transactionRecord) => {
    Promise.all([
      insertTransactionScript(
        transactionRecord.scriptRoot,
        transactionRecord.txScript
      ),
      upsertTransactionRecord(
        transactionRecord.Id,
        transactionRecord.details,
        transactionRecord.scriptRoot,
        transactionRecord.blockNum,
        transactionRecord.commitHeight,
        transactionRecord.discardCause
      ),
    ]);
  });

  let accountUpdatesWriteOp = accountUpdates.map((accountUpdate) => {
    Promise.all([
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
        account.codeRoot,
        account.assetVaultRoot,
        account.nonce,
        account.commited,
        account.accountSeed,
        account.account_commitment
      ),
    ]);
  });

  return await db.transaction(
    "rw",
    stateSync,
    inputNotes,
    outputNotes,
    transactions,
    blockHeaders,
    partialBlockchainNodes,
    tags,
    async (tx) => {
      await inputNotesUpsertOp;
      await outputNotesUpsertOp;
      await transactionWriteOp;
      await accountUpdatesWriteOp;
      await updateSyncHeight(tx, blockNum);
      for (let i = 0; i < newBlockHeaders.length; i++) {
        await updateBlockHeader(
          tx,
          newBlockNums[i],
          newBlockHeaders[i],
          partialBlockchainPeaks[i],
          blockHasRelevantNotes[i] == 1 // blockHasRelevantNotes is a u8 array, so we convert it to boolean
        );
      }
      await updatePartialBlockchainNodes(
        tx,
        serializedNodeIds,
        serializedNodes
      );
      await updateCommittedNoteTags(tx, noteTagsToRemove);
    }
  );
}

async function updateSyncHeight(tx, blockNum) {
  try {
    await tx.stateSync.update(1, { blockNum: blockNum });
  } catch (error) {
    console.error("Failed to update sync height: ", error.toString());
    throw error;
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
    console.error("Failed to insert block header: ", err.toString());
    throw err;
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
    console.error(
      "Failed to update partial blockchain nodes: ",
      err.toString()
    );
    throw err;
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
    console.error("Error updating committed notes:", error.toString());
    throw error;
  }
}

function uint8ArrayToBase64(bytes) {
  const binary = bytes.reduce(
    (acc, byte) => acc + String.fromCharCode(byte),
    ""
  );
  return btoa(binary);
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
