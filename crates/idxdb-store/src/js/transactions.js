import { getDatabase, } from "./schema.js";
import { logWebStoreError, mapOption, uint8ArrayToBase64 } from "./utils.js";
import { applyFullAccountState, applyTransactionDelta } from "./accounts.js";
import { upsertInputNote, upsertOutputNote } from "./notes.js";
const IDS_FILTER_PREFIX = "Ids:";
const EXPIRED_BEFORE_FILTER_PREFIX = "ExpiredPending:";
const STATUS_PENDING_VARIANT = 0;
const STATUS_COMMITTED_VARIANT = 1;
const STATUS_DISCARDED_VARIANT = 2;
export async function getTransactions(dbId, filter) {
    let transactionRecords = [];
    try {
        const db = getDatabase(dbId);
        if (filter === "Uncommitted") {
            transactionRecords = await db.transactions
                .filter((tx) => tx.statusVariant === STATUS_PENDING_VARIANT)
                .toArray();
        }
        else if (filter.startsWith(IDS_FILTER_PREFIX)) {
            const idsString = filter.substring(IDS_FILTER_PREFIX.length);
            const ids = idsString.split(",");
            if (ids.length > 0) {
                transactionRecords = await db.transactions
                    .where("id")
                    .anyOf(ids)
                    .toArray();
            }
            else {
                transactionRecords = [];
            }
        }
        else if (filter.startsWith(EXPIRED_BEFORE_FILTER_PREFIX)) {
            const blockNumString = filter.substring(EXPIRED_BEFORE_FILTER_PREFIX.length);
            const blockNum = parseInt(blockNumString);
            transactionRecords = await db.transactions
                .filter((tx) => tx.blockNum < blockNum &&
                tx.statusVariant !== STATUS_COMMITTED_VARIANT &&
                tx.statusVariant !== STATUS_DISCARDED_VARIANT)
                .toArray();
        }
        else {
            transactionRecords = await db.transactions.toArray();
        }
        if (transactionRecords.length === 0) {
            return [];
        }
        const scriptRoots = transactionRecords
            .map((transactionRecord) => {
            return transactionRecord.scriptRoot;
        })
            .filter((scriptRoot) => scriptRoot != undefined);
        const scripts = await db.transactionScripts
            .where("scriptRoot")
            .anyOf(scriptRoots)
            .toArray();
        const scriptMap = new Map();
        scripts.forEach((script) => {
            if (script.txScript) {
                scriptMap.set(script.scriptRoot, script.txScript);
            }
        });
        const processedTransactions = await Promise.all(transactionRecords.map((transactionRecord) => {
            let txScriptBase64 = undefined;
            if (transactionRecord.scriptRoot) {
                const txScript = scriptMap.get(transactionRecord.scriptRoot);
                if (txScript) {
                    txScriptBase64 = uint8ArrayToBase64(txScript);
                }
            }
            const detailsBase64 = uint8ArrayToBase64(transactionRecord.details);
            const statusBase64 = uint8ArrayToBase64(transactionRecord.status);
            const data = {
                id: transactionRecord.id,
                details: detailsBase64,
                scriptRoot: transactionRecord.scriptRoot,
                txScript: txScriptBase64,
                blockNum: transactionRecord.blockNum,
                statusVariant: transactionRecord.statusVariant,
                status: statusBase64,
            };
            return data;
        }));
        return processedTransactions;
    }
    catch (err) {
        logWebStoreError(err, "Failed to get transactions");
    }
}
export async function insertTransactionScript(dbId, scriptRoot, txScript, tx) {
    try {
        const db = getDatabase(dbId);
        const scriptRootArray = new Uint8Array(scriptRoot);
        const scriptRootBase64 = uint8ArrayToBase64(scriptRootArray);
        const data = {
            scriptRoot: scriptRootBase64,
            txScript: mapOption(txScript, (txScript) => new Uint8Array(txScript)),
        };
        await (tx || db).transactionScripts.put(data);
    }
    catch (error) {
        logWebStoreError(error, "Failed to insert transaction script");
        throw error;
    }
}
export async function upsertTransactionRecord(dbId, transactionId, details, blockNum, statusVariant, status, scriptRoot, tx) {
    try {
        const db = getDatabase(dbId);
        const data = {
            id: transactionId,
            details,
            scriptRoot: mapOption(scriptRoot, (root) => uint8ArrayToBase64(root)),
            blockNum,
            statusVariant,
            status,
        };
        await (tx || db).transactions.put(data);
    }
    catch (err) {
        logWebStoreError(err, "Failed to insert proven transaction data");
        throw err;
    }
}
/**
 * Applies a batch of transaction updates atomically inside a single Dexie transaction.
 *
 * All sub-operations that internally call `db.dexie.transaction()` are auto-joined by Dexie
 * as nested sub-transactions when run inside this parent transaction, provided the parent
 * scope is a superset of every sub-transaction scope.
 */
export async function applyTransactionBatch(dbId, payloads) {
    const db = getDatabase(dbId);
    await db.dexie.transaction("rw", [
        db.transactions,
        db.transactionScripts,
        db.latestAccountStorages,
        db.historicalAccountStorages,
        db.latestStorageMapEntries,
        db.historicalStorageMapEntries,
        db.latestAccountAssets,
        db.historicalAccountAssets,
        db.latestAccountHeaders,
        db.historicalAccountHeaders,
        db.inputNotes,
        db.outputNotes,
        db.notesScripts,
        db.tags,
    ], async () => {
        for (const payload of payloads) {
            // 1. Insert the transaction record (script first, then record)
            const rec = payload.transactionRecord;
            if (rec.scriptRoot && rec.txScript) {
                await insertTransactionScript(dbId, rec.scriptRoot, rec.txScript);
            }
            await upsertTransactionRecord(dbId, rec.id, rec.details, rec.blockNum, rec.statusVariant, rec.status, rec.scriptRoot);
            // 2. Apply account state (full or delta)
            const acct = payload.accountState;
            if (acct.kind === "full") {
                await applyFullAccountState(dbId, acct.account);
            }
            else {
                await applyTransactionDelta(dbId, acct.accountId, acct.nonce, acct.updatedSlots, acct.changedMapEntries, acct.changedAssets, acct.codeRoot, acct.storageRoot, acct.vaultRoot, acct.committed, acct.commitment);
            }
            // 3. Upsert input and output notes
            for (const note of payload.inputNotes) {
                await upsertInputNote(dbId, note.noteId, note.noteAssets, note.serialNumber, note.inputs, note.noteScriptRoot, note.noteScript, note.nullifier, note.createdAt, note.stateDiscriminant, note.state, note.consumedBlockHeight ?? null, note.consumedTxOrder ?? null, note.consumerAccountId ?? null);
            }
            for (const note of payload.outputNotes) {
                await upsertOutputNote(dbId, note.noteId, note.noteAssets, note.recipientDigest, note.metadata, note.nullifier, note.expectedHeight, note.stateDiscriminant, note.state);
            }
            // 4. Add note tags (deduplicated within the transaction)
            for (const tagEntry of payload.tags) {
                const tagArray = new Uint8Array(tagEntry.tag);
                const tagBase64 = uint8ArrayToBase64(tagArray);
                const sourceNoteId = tagEntry.sourceNoteId ?? "";
                const sourceAccountId = tagEntry.sourceAccountId ?? "";
                // Check for existing tag to avoid duplicates (mirrors the Rust add_note_tag logic)
                const existing = await db.tags
                    .where({ tag: tagBase64, sourceNoteId, sourceAccountId })
                    .first();
                if (!existing) {
                    await db.tags.add({
                        tag: tagBase64,
                        sourceNoteId,
                        sourceAccountId,
                    });
                }
            }
        }
    });
}
