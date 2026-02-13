import {
  getDatabase,
  JsVaultAsset,
  JsStorageSlot,
  JsStorageMapEntry,
  IBlockHeader,
  IStateSync,
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
import Dexie from "dexie";

export async function getNoteTags(dbId: string) {
  try {
    const db = getDatabase(dbId);
    let records = await db.tags.toArray();

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

export async function getSyncHeight(dbId: string) {
  try {
    const db = getDatabase(dbId);
    const record = await db.stateSync.get(1);
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
  dbId: string,
  tag: Uint8Array,
  sourceNoteId: string,
  sourceAccountId: string
) {
  try {
    const db = getDatabase(dbId);
    let tagArray = new Uint8Array(tag);
    let tagBase64 = uint8ArrayToBase64(tagArray);
    await db.tags.add({
      tag: tagBase64,
      sourceNoteId: sourceNoteId ? sourceNoteId : "",
      sourceAccountId: sourceAccountId ? sourceAccountId : "",
    });
  } catch (error) {
    logWebStoreError(error, "Failed to add note tag");
  }
}

export async function removeNoteTag(
  dbId: string,
  tag: Uint8Array,
  sourceNoteId?: string,
  sourceAccountId?: string
) {
  try {
    const db = getDatabase(dbId);
    let tagArray = new Uint8Array(tag);
    let tagBase64 = uint8ArrayToBase64(tagArray);

    return await db.tags
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
  blockNum: number;
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
  blockNum: number;
  flattenedNewBlockHeaders: FlattenedU8Vec;
  flattenedPartialBlockChainPeaks: FlattenedU8Vec;
  newBlockNums: number[];
  blockHasRelevantNotes: Uint8Array;
  serializedNodeIds: string[];
  serializedNodes: string[];
  committedNoteIds: string[];
  serializedInputNotes: SerializedInputNoteData[];
  serializedOutputNotes: SerializedOutputNoteData[];
  accountUpdates: JsAccountUpdate[];
  transactionUpdates: SerializedTransactionData[];
}

export async function applyStateSync(
  dbId: string,
  stateUpdate: JsStateSyncUpdate
) {
  const db = getDatabase(dbId);
  const {
    blockNum,
    flattenedNewBlockHeaders,
    flattenedPartialBlockChainPeaks,
    newBlockNums,
    blockHasRelevantNotes,
    serializedNodeIds,
    serializedNodes,
    committedNoteIds,
    serializedInputNotes,
    serializedOutputNotes,
    accountUpdates,
    transactionUpdates,
  } = stateUpdate;

  const newBlockHeaders = reconstructFlattenedVec(flattenedNewBlockHeaders);
  const partialBlockchainPeaks = reconstructFlattenedVec(
    flattenedPartialBlockChainPeaks
  );

  const tablesToAccess = [
    db.stateSync,
    db.inputNotes,
    db.outputNotes,
    db.notesScripts,
    db.transactions,
    db.transactionScripts,
    db.blockHeaders,
    db.partialBlockchainNodes,
    db.tags,
    db.latestAccountHeaders,
    db.historicalAccountHeaders,
    db.latestAccountStorages,
    db.historicalAccountStorages,
    db.latestStorageMapEntries,
    db.historicalStorageMapEntries,
    db.latestAccountAssets,
    db.historicalAccountAssets,
  ];

  return await db.dexie.transaction("rw", tablesToAccess, async (tx) => {
    await Promise.all([
      Promise.all(
        serializedInputNotes.map((note) => {
          return upsertInputNote(
            dbId,
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
      ),
      Promise.all(
        serializedOutputNotes.map((note) => {
          return upsertOutputNote(
            dbId,
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
      ),
      Promise.all(
        transactionUpdates.map((transactionRecord) => {
          let promises = [
            upsertTransactionRecord(
              dbId,
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
                dbId,
                transactionRecord.scriptRoot,
                transactionRecord.txScript
              )
            );
          }

          return Promise.all(promises);
        })
      ),
      Promise.all(
        accountUpdates.flatMap((accountUpdate) => {
          return [
            upsertAccountStorage(
              dbId,
              accountUpdate.accountId,
              accountUpdate.storageSlots
            ),
            upsertStorageMapEntries(
              dbId,
              accountUpdate.accountId,
              accountUpdate.storageMapEntries
            ),
            upsertVaultAssets(
              dbId,
              accountUpdate.accountId,
              accountUpdate.assets
            ),
            upsertAccountRecord(
              dbId,
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
      ),
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

async function updateSyncHeight(tx: Transaction, blockNum: number) {
  try {
    // Only update if moving forward to prevent race conditions
    const current = await (
      tx as Transaction & { stateSync: Dexie.Table<IStateSync, number> }
    ).stateSync.get(1);
    if (!current || current.blockNum < blockNum) {
      await (
        tx as Transaction & { stateSync: Dexie.Table<IStateSync, number> }
      ).stateSync.update(1, {
        blockNum: blockNum,
      });
    }
  } catch (error) {
    logWebStoreError(error, "Failed to update sync height");
  }
}

async function updateBlockHeader(
  tx: Transaction,
  blockNum: number,
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

    const existingBlockHeader = await (
      tx as Transaction & { blockHeaders: Dexie.Table<IBlockHeader, number> }
    ).blockHeaders.get(blockNum);

    if (!existingBlockHeader) {
      await (
        tx as Transaction & { blockHeaders: Dexie.Table }
      ).blockHeaders.add(data);
    }
  } catch (err) {
    logWebStoreError(err, "Failed to insert block header");
  }
}

async function updatePartialBlockchainNodes(
  tx: Transaction,
  nodeIndexes: string[],
  nodes: string[]
) {
  try {
    if (nodeIndexes.length !== nodes.length) {
      throw new Error(
        "nodeIndexes and nodes arrays must be of the same length"
      );
    }

    if (nodeIndexes.length === 0) {
      return;
    }

    const data = nodes.map((node, index) => ({
      id: Number(nodeIndexes[index]),
      node: node,
    }));
    // Use bulkPut to add/overwrite the entries
    await (
      tx as Transaction & { partialBlockchainNodes: Dexie.Table }
    ).partialBlockchainNodes.bulkPut(data);
  } catch (err) {
    logWebStoreError(err, "Failed to update partial blockchain nodes");
  }
}

async function updateCommittedNoteTags(
  tx: Transaction,
  inputNoteIds: string[]
) {
  try {
    for (let i = 0; i < inputNoteIds.length; i++) {
      const noteId = inputNoteIds[i];
      await (tx as Transaction & { tags: Dexie.Table }).tags
        .where("sourceNoteId")
        .equals(noteId)
        .delete();
    }
  } catch (error) {
    logWebStoreError(error, "Failed to update committed note tags");
  }
}

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

export async function discardTransactions(
  dbId: string,
  transactions: string[]
) {
  try {
    const db = getDatabase(dbId);
    await db.transactions.where("id").anyOf(transactions).delete();
  } catch (err) {
    logWebStoreError(err, "Failed to discard transactions");
  }
}
