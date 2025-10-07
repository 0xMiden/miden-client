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
  upsertAccountStorage,
  upsertAccountRecord,
  upsertVaultAssets,
  upsertStorageMapEntries,
} from "./accounts.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";
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
    logWebStoreError(error, "Failed to add note tag");
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
    logWebStoreError(error, "Failed to remove note tag");
  }
}

// TODO: The interfaces below are already defined in Rust, we should look into something that keeps
// types in sync between Rust and Typescript (#1083).
interface FlattenedU8Vec {
  data(): Uint8Array;
  lengths(): number[];
}

interface SerializedInputNoteData {
  noteId: string;
  noteAssets: Uint8Array;
  serialNumber: Uint8Array;
  inputs: Uint8Array;
  noteScriptRoot: string;
  noteScript: Uint8Array;
  nullifier: string;
  createdAt: string;
  stateDiscriminant: number;
  state: Uint8Array;
}

interface SerializedOutputNoteData {
  noteId: string;
  noteAssets: Uint8Array;
  recipientDigest: string;
  metadata: Uint8Array;
  nullifier?: string;
  expectedHeight: number;
  stateDiscriminant: number;
  state: Uint8Array;
}

interface SerializedTransactionData {
  id: string;
  details: Uint8Array;
  blockNum: string;
  scriptRoot?: Uint8Array;
  statusVariant: number;
  status: Uint8Array;
  txScript?: Uint8Array;
}

interface JsAccountUpdate {
  storageRoot: string;
  storageSlots: JsStorageSlot[];
  storageMapEntries: JsStorageMapEntry[];
  assetVaultRoot: string;
  assets: JsVaultAsset[];
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

export interface JsVaultAsset {
  root: string;
  vaultKey: string;
  faucetIdPrefix: string;
  asset: string;
}

export interface JsStorageSlot {
  commitment: string;
  slotIndex: number;
  slotValue: string;
  slotType: number;
}

export interface JsStorageMapEntry {
  root: string;
  key: string;
  value: string;
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
        upsertAccountStorage(accountUpdate.storageSlots),
        upsertStorageMapEntries(accountUpdate.storageMapEntries),
        upsertVaultAssets(accountUpdate.assets),
        upsertAccountRecord(
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

async function updateSyncHeight(
  tx: Transaction & { stateSync: typeof stateSync },
  blockNum: string
) {
  try {
    await tx.stateSync.update(1, { blockNum: blockNum });
  } catch (error) {
    logWebStoreError(error, "Failed to update sync height");
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
    const data = {
      blockNum: blockNum,
      header: blockHeader,
      partialBlockchainPeaks,
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
    logWebStoreError(err, "Failed to update partial blockchain nodes");
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
    logWebStoreError(error, "Failed to pudate committed note tags");
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
