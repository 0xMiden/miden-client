import { getDatabase, } from "./schema.js";
import { upsertTransactionRecord, insertTransactionScript, } from "./transactions.js";
import { upsertInputNote, upsertOutputNote } from "./notes.js";
import { upsertAccountStorage, upsertAccountRecord, upsertVaultAssets, upsertStorageMapEntries, } from "./accounts.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";
export async function getNoteTags(dbId) {
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
    }
    catch (error) {
        logWebStoreError(error, "Error fetch tag record");
    }
}
export async function getSyncHeight(dbId) {
    try {
        const db = getDatabase(dbId);
        const record = await db.stateSync.get(1);
        if (record) {
            let data = {
                blockNum: record.blockNum,
            };
            return data;
        }
        else {
            return null;
        }
    }
    catch (error) {
        logWebStoreError(error, "Error fetching sync height");
    }
}
export async function addNoteTag(dbId, tag, sourceNoteId, sourceAccountId) {
    try {
        const db = getDatabase(dbId);
        let tagArray = new Uint8Array(tag);
        let tagBase64 = uint8ArrayToBase64(tagArray);
        await db.tags.add({
            tag: tagBase64,
            sourceNoteId: sourceNoteId ? sourceNoteId : "",
            sourceAccountId: sourceAccountId ? sourceAccountId : "",
        });
    }
    catch (error) {
        logWebStoreError(error, "Failed to add note tag");
    }
}
export async function removeNoteTag(dbId, tag, sourceNoteId, sourceAccountId) {
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
    }
    catch (error) {
        logWebStoreError(error, "Failed to remove note tag");
    }
}
export async function applyStateSync(dbId, stateUpdate) {
    const db = getDatabase(dbId);
    const { blockNum, flattenedNewBlockHeaders, flattenedPartialBlockChainPeaks, newBlockNums, blockHasRelevantNotes, serializedNodeIds, serializedNodes, committedNoteIds, serializedInputNotes, serializedOutputNotes, accountUpdates, transactionUpdates, } = stateUpdate;
    const newBlockHeaders = reconstructFlattenedVec(flattenedNewBlockHeaders);
    const partialBlockchainPeaks = reconstructFlattenedVec(flattenedPartialBlockChainPeaks);
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
            Promise.all(serializedInputNotes.map((note) => {
                return upsertInputNote(dbId, note.noteId, note.noteAssets, note.serialNumber, note.inputs, note.noteScriptRoot, note.noteScript, note.nullifier, note.createdAt, note.stateDiscriminant, note.state);
            })),
            Promise.all(serializedOutputNotes.map((note) => {
                return upsertOutputNote(dbId, note.noteId, note.noteAssets, note.recipientDigest, note.metadata, note.nullifier, note.expectedHeight, note.stateDiscriminant, note.state);
            })),
            Promise.all(transactionUpdates.map((transactionRecord) => {
                let promises = [
                    upsertTransactionRecord(dbId, transactionRecord.id, transactionRecord.details, transactionRecord.blockNum, transactionRecord.statusVariant, transactionRecord.status, transactionRecord.scriptRoot),
                ];
                if (transactionRecord.scriptRoot && transactionRecord.txScript) {
                    promises.push(insertTransactionScript(dbId, transactionRecord.scriptRoot, transactionRecord.txScript));
                }
                return Promise.all(promises);
            })),
            Promise.all(accountUpdates.flatMap((accountUpdate) => {
                return [
                    upsertAccountStorage(dbId, accountUpdate.accountId, accountUpdate.nonce, accountUpdate.storageSlots),
                    upsertStorageMapEntries(dbId, accountUpdate.accountId, accountUpdate.nonce, accountUpdate.storageMapEntries),
                    upsertVaultAssets(dbId, accountUpdate.accountId, accountUpdate.nonce, accountUpdate.assets),
                    upsertAccountRecord(dbId, accountUpdate.accountId, accountUpdate.codeRoot, accountUpdate.storageRoot, accountUpdate.vaultRoot, accountUpdate.nonce, accountUpdate.committed, accountUpdate.accountCommitment, accountUpdate.accountSeed),
                ];
            })),
            updateSyncHeight(tx, blockNum),
            updatePartialBlockchainNodes(tx, serializedNodeIds, serializedNodes),
            updateCommittedNoteTags(tx, committedNoteIds),
            Promise.all(newBlockHeaders.map((newBlockHeader, i) => {
                return updateBlockHeader(tx, newBlockNums[i], newBlockHeader, partialBlockchainPeaks[i], blockHasRelevantNotes[i] == 1);
            })),
        ]);
    });
}
async function updateSyncHeight(tx, blockNum) {
    try {
        // Only update if moving forward to prevent race conditions
        const current = await tx.stateSync.get(1);
        if (!current || current.blockNum < blockNum) {
            await tx.stateSync.update(1, {
                blockNum: blockNum,
            });
        }
    }
    catch (error) {
        logWebStoreError(error, "Failed to update sync height");
    }
}
async function updateBlockHeader(tx, blockNum, blockHeader, partialBlockchainPeaks, hasClientNotes) {
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
    }
    catch (err) {
        logWebStoreError(err, "Failed to insert block header");
    }
}
async function updatePartialBlockchainNodes(tx, nodeIndexes, nodes) {
    try {
        if (nodeIndexes.length !== nodes.length) {
            throw new Error("nodeIndexes and nodes arrays must be of the same length");
        }
        if (nodeIndexes.length === 0) {
            return;
        }
        const data = nodes.map((node, index) => ({
            id: Number(nodeIndexes[index]),
            node: node,
        }));
        // Use bulkPut to add/overwrite the entries
        await tx.partialBlockchainNodes.bulkPut(data);
    }
    catch (err) {
        logWebStoreError(err, "Failed to update partial blockchain nodes");
    }
}
async function updateCommittedNoteTags(tx, inputNoteIds) {
    try {
        for (let i = 0; i < inputNoteIds.length; i++) {
            const noteId = inputNoteIds[i];
            await tx.tags
                .where("sourceNoteId")
                .equals(noteId)
                .delete();
        }
    }
    catch (error) {
        logWebStoreError(error, "Failed to update committed note tags");
    }
}
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
export async function discardTransactions(dbId, transactions) {
    try {
        const db = getDatabase(dbId);
        await db.transactions.where("id").anyOf(transactions).delete();
    }
    catch (err) {
        logWebStoreError(err, "Failed to discard transactions");
    }
}
