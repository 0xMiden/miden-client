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
import { logDexieError, uint8ArrayToBase64 } from "./utils.js";
import { Transaction } from "dexie";

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
    logDexieError(error, "Error fetch tag record");
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
    logDexieError(error, "Error fetching sync height");
  }
}

export async function addNoteTag(
  tag: Uint8Array,
  sourceNoteId: string,
  sourceAccountId: string
) {
  try {
    let tagArray = new Uint8Array(tag);
    let tagBase64 = uint8ArrayToBase64(tagArray);
    await tags.add({
      tag: tagBase64,
      sourceNoteId: sourceNoteId ? sourceNoteId : "",
      sourceAccountId: sourceAccountId ? sourceAccountId : "",
    });
  } catch (error) {
    logDexieError(error, "Failed to add note tag");
  }
}

export async function removeNoteTag(
  tag: Uint8Array,
  sourceNoteId?: string,
  sourceAccountId?: string
) {
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
    logDexieError(error, "Failed to remove note tag");
  }
}

interface FlattenedU8Vec {
  data(): Uint8Array;
  lengths(): number[];
}

interface SerializedInputNoteData {
  noteId: string;
  noteAssets: any;
  serialNumber: string;
  inputs: any;
  noteScriptRoot: string;
  noteScript: any;
  nullifier: string;
  createdAt: number;
  stateDiscriminant: number;
  state: any;
}

interface SerializedOutputNoteData {
  noteId: string;
  noteAssets: any;
  recipientDigest: string;
  metadata: any;
  nullifier: string;
  expectedHeight: number;
  stateDiscriminant: number;
  state: any;
}

interface SerializedTransactionData {
  id: string;
  details: any;
  blockNum: string;
  scriptRoot: Uint8Array;
  commitHeight: string;
  discardCause?: Uint8Array;
  txScript?: Uint8Array;
}

interface JsAccountUpdate {
  storageRoot: string;
  storageSlots: Uint8Array;
  assetVaultRoot: string;
  assetBytes: Uint8Array;
  accountId: string;
  codeRoot: string;
  committed: boolean;
  nonce: string;
  accountCommitment: string;
  accountSeed?: Uint8Array;
}

interface JsStateSyncUpdate {
  blockNum: string;
  flattenedNewBlockHeaders: FlattenedU8Vec;
  flattenedPartialBlockChainPeaks: FlattenedU8Vec;
  newBlockNums: string[];
  blockHasRelevantNotes: Uint8Array;
  serializedNodeIds: string[];
  serializedNodes: string[];
  committedNoteIds: string[];
  serializedInputNotes: SerializedInputNoteData[];
  serializedOutputNotes: SerializedOutputNoteData[];
  accountUpdates: JsAccountUpdate[];
  transactionUpdates: SerializedTransactionData[];
}

interface NoteTag {
  tag: string;
  sourceNoteId: string | null;
  sourceAccountId: string | null;
}

interface SyncHeight {
  blockNum: number;
}

/*
 * Takes a `JsStateSyncUpdate` object and writes the state update into the store.
 * @param {JsStateSyncUpdate}
 */
export async function applyStateSync(stateUpdate: JsStateSyncUpdate) {
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
  });

  // See comment above, the same thing applies here, but for Output Notes.
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
  });

  // Fit insert operations into a single promise.
  let inputNotesWriteOp = Promise.all(serializedInputNotes);
  let outputNotesWriteOp = Promise.all(serializedOutputNotes);

  // Promises to insert each transaction update.
  transactionUpdates.flatMap((transactionRecord) => {
    [
      insertTransactionScript(
        transactionRecord.scriptRoot,
        transactionRecord.txScript
      ),
      upsertTransactionRecord(
        transactionRecord.id,
        transactionRecord.details,
        transactionRecord.blockNum,
        transactionRecord.scriptRoot,
        transactionRecord.commitHeight,
        transactionRecord.discardCause
      ),
    ];
  });

  // Fit the upsert transactions into a single promise
  let transactionWriteOp = Promise.all(transactionUpdates);

  // Promises to insert each account update.
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
  });

  let accountUpdatesWriteOp = Promise.all(accountUpdates);

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
  return await db.transaction(
    "rw",
    [
      stateSync,
      inputNotes,
      outputNotes,
      transactions,
      blockHeaders,
      partialBlockchainNodes,
      tags,
    ],
    async (tx) => {
      await Promise.all([
        inputNotesWriteOp,
        outputNotesWriteOp,
        transactionWriteOp,
        accountUpdatesWriteOp,
      ]);
      // Update to the new block number
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
      await updateCommittedNoteTags(tx, committedNoteIds);
    }
  );
}

async function updateSyncHeight(
  tx: Transaction & { stateSync: typeof stateSync },
  blockNum: string
) {
  try {
    await tx.stateSync.update(1, { blockNum: blockNum });
  } catch (error) {
    logDexieError(error, "Failed to update sync height");
  }
}

async function updateBlockHeader(
  tx: Transaction & { blockHeaders: typeof blockHeaders },
  blockNum: string,
  blockHeader: Uint8Array,
  partialBlockchainPeaks: Uint8Array,
  hasClientNotes: boolean
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
    logDexieError(err, "Failed to insert block header");
  }
}

async function updatePartialBlockchainNodes(
  tx: Transaction & { partialBlockchainNodes: typeof partialBlockchainNodes },
  nodeIndexes: string[],
  nodes: string[]
) {
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
    logDexieError(err, "Failed to update partial blockchain nodes");
  }
}

async function updateCommittedNoteTags(
  tx: Transaction & { tags: typeof tags },
  inputNoteIds: string[]
) {
  try {
    for (let i = 0; i < inputNoteIds.length; i++) {
      const noteId = inputNoteIds[i];
      // Remove note tags
      await tx.tags.where("source_note_id").equals(noteId).delete();
    }
  } catch (error) {
    logDexieError(error, "Failed to pudate committed note tags");
  }
}

// Helper function to reconstruct arrays from flattened data
function reconstructFlattenedVec(flattenedVec: FlattenedU8Vec) {
  const data = flattenedVec.data();
  const lengths = flattenedVec.lengths();

  let index = 0;
  const result: Uint8Array[] = [];
  lengths.forEach((length: number) => {
    result.push(data.slice(index, index + length));
    index += length;
  });
  return result;
}
