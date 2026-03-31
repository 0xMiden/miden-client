import { getDatabase, } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";
function seedToBase64(seed) {
    return seed ? uint8ArrayToBase64(seed) : undefined;
}
export async function getAccountIds(dbId) {
    try {
        const db = getDatabase(dbId);
        const records = await db.latestAccountHeaders.toArray();
        return records.map((entry) => entry.id);
    }
    catch (error) {
        logWebStoreError(error, "Error while fetching account IDs");
    }
    return [];
}
export async function getAllAccountHeaders(dbId) {
    try {
        const db = getDatabase(dbId);
        const records = await db.latestAccountHeaders.toArray();
        const resultObject = records.map((record) => ({
            id: record.id,
            nonce: record.nonce,
            vaultRoot: record.vaultRoot,
            storageRoot: record.storageRoot || "",
            codeRoot: record.codeRoot || "",
            accountSeed: seedToBase64(record.accountSeed),
            locked: record.locked,
            committed: record.committed,
            accountCommitment: record.accountCommitment || "",
        }));
        return resultObject;
    }
    catch (error) {
        logWebStoreError(error, "Error while fetching account headers");
    }
}
export async function getAccountHeader(dbId, accountId) {
    try {
        const db = getDatabase(dbId);
        const record = await db.latestAccountHeaders
            .where("id")
            .equals(accountId)
            .first();
        if (!record) {
            console.log("No account header record found for given ID.");
            return null;
        }
        return {
            id: record.id,
            nonce: record.nonce,
            vaultRoot: record.vaultRoot,
            storageRoot: record.storageRoot,
            codeRoot: record.codeRoot,
            accountSeed: seedToBase64(record.accountSeed),
            locked: record.locked,
        };
    }
    catch (error) {
        logWebStoreError(error, `Error while fetching account header for id: ${accountId}`);
    }
}
export async function getAccountHeaderByCommitment(dbId, accountCommitment) {
    try {
        const db = getDatabase(dbId);
        const record = await db.historicalAccountHeaders
            .where("accountCommitment")
            .equals(accountCommitment)
            .first();
        if (!record) {
            return undefined;
        }
        return {
            id: record.id,
            nonce: record.nonce,
            vaultRoot: record.vaultRoot,
            storageRoot: record.storageRoot,
            codeRoot: record.codeRoot,
            accountSeed: seedToBase64(record.accountSeed),
            locked: record.locked,
        };
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account header for commitment ${accountCommitment}`);
    }
}
export async function getAccountCode(dbId, codeRoot) {
    try {
        const db = getDatabase(dbId);
        const allMatchingRecords = await db.accountCodes
            .where("root")
            .equals(codeRoot)
            .toArray();
        const codeRecord = allMatchingRecords[0];
        if (codeRecord === undefined) {
            console.log("No records found for given code root.");
            return null;
        }
        const codeBase64 = uint8ArrayToBase64(codeRecord.code);
        return {
            root: codeRecord.root,
            code: codeBase64,
        };
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account code for root ${codeRoot}`);
    }
}
export async function getAccountStorage(dbId, accountId, slotNames) {
    try {
        const db = getDatabase(dbId);
        let query = db.latestAccountStorages.where("accountId").equals(accountId);
        let allMatchingRecords;
        if (slotNames.length) {
            const nameSet = new Set(slotNames);
            allMatchingRecords = await query
                .and((record) => nameSet.has(record.slotName))
                .toArray();
        }
        else {
            allMatchingRecords = await query.toArray();
        }
        return allMatchingRecords.map((record) => ({
            slotName: record.slotName,
            slotValue: record.slotValue,
            slotType: record.slotType,
        }));
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account storage for account ${accountId}`);
    }
}
export async function getAccountStorageMaps(dbId, accountId) {
    try {
        const db = getDatabase(dbId);
        const allMatchingRecords = await db.latestStorageMapEntries
            .where("accountId")
            .equals(accountId)
            .toArray();
        return allMatchingRecords;
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account storage maps for account ${accountId}`);
    }
}
export async function getAccountVaultAssets(dbId, accountId, faucetIdPrefixes) {
    try {
        const db = getDatabase(dbId);
        let query = db.latestAccountAssets.where("accountId").equals(accountId);
        let records;
        if (faucetIdPrefixes.length) {
            const prefixSet = new Set(faucetIdPrefixes);
            records = await query
                .and((record) => prefixSet.has(record.faucetIdPrefix))
                .toArray();
        }
        else {
            records = await query.toArray();
        }
        return records.map((record) => ({
            asset: record.asset,
        }));
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account vault for account ${accountId}`);
    }
}
export async function getAccountAddresses(dbId, accountId) {
    try {
        const db = getDatabase(dbId);
        const allMatchingRecords = await db.addresses
            .where("id")
            .equals(accountId)
            .toArray();
        if (allMatchingRecords.length === 0) {
            console.log("No address records found for given account ID.");
            return [];
        }
        return allMatchingRecords;
    }
    catch (error) {
        logWebStoreError(error, `Error while fetching account addresses for id: ${accountId}`);
    }
}
export async function upsertAccountCode(dbId, codeRoot, code) {
    try {
        const db = getDatabase(dbId);
        const data = {
            root: codeRoot,
            code,
        };
        await db.accountCodes.put(data);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting code with root: ${codeRoot}`);
    }
}
export async function upsertAccountStorage(dbId, accountId, storageSlots) {
    try {
        const db = getDatabase(dbId);
        await db.latestAccountStorages
            .where("accountId")
            .equals(accountId)
            .delete();
        if (storageSlots.length === 0)
            return;
        const latestEntries = storageSlots.map((slot) => ({
            accountId,
            slotName: slot.slotName,
            slotValue: slot.slotValue,
            slotType: slot.slotType,
        }));
        await db.latestAccountStorages.bulkPut(latestEntries);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting storage slots`);
    }
}
export async function upsertStorageMapEntries(dbId, accountId, entries) {
    try {
        const db = getDatabase(dbId);
        await db.latestStorageMapEntries
            .where("accountId")
            .equals(accountId)
            .delete();
        if (entries.length === 0)
            return;
        const latestEntries = entries.map((entry) => ({
            accountId,
            slotName: entry.slotName,
            key: entry.key,
            value: entry.value,
        }));
        await db.latestStorageMapEntries.bulkPut(latestEntries);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting storage map entries`);
    }
}
export async function upsertVaultAssets(dbId, accountId, assets) {
    try {
        const db = getDatabase(dbId);
        await db.latestAccountAssets.where("accountId").equals(accountId).delete();
        if (assets.length === 0)
            return;
        const latestEntries = assets.map((asset) => ({
            accountId,
            vaultKey: asset.vaultKey,
            faucetIdPrefix: asset.faucetIdPrefix,
            asset: asset.asset,
        }));
        await db.latestAccountAssets.bulkPut(latestEntries);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting assets`);
    }
}
export async function applyTransactionDelta(dbId, accountId, nonce, updatedSlots, changedMapEntries, changedAssets, codeRoot, storageRoot, vaultRoot, committed, commitment) {
    try {
        const db = getDatabase(dbId);
        await db.dexie.transaction("rw", [
            db.latestAccountStorages,
            db.historicalAccountStorages,
            db.latestStorageMapEntries,
            db.historicalStorageMapEntries,
            db.latestAccountAssets,
            db.historicalAccountAssets,
            db.latestAccountHeaders,
            db.historicalAccountHeaders,
        ], async () => {
            // Apply storage delta: read old → archive → write new
            for (const slot of updatedSlots) {
                const oldSlot = await db.latestAccountStorages
                    .where("[accountId+slotName]")
                    .equals([accountId, slot.slotName])
                    .first();
                await db.historicalAccountStorages.put({
                    accountId,
                    replacedAtNonce: nonce,
                    slotName: slot.slotName,
                    oldSlotValue: oldSlot?.slotValue ?? null,
                    slotType: slot.slotType,
                });
                await db.latestAccountStorages.put({
                    accountId,
                    slotName: slot.slotName,
                    slotValue: slot.slotValue,
                    slotType: slot.slotType,
                });
            }
            // Process map entries: read old → archive → update latest
            for (const entry of changedMapEntries) {
                const oldEntry = await db.latestStorageMapEntries
                    .where("[accountId+slotName+key]")
                    .equals([accountId, entry.slotName, entry.key])
                    .first();
                await db.historicalStorageMapEntries.put({
                    accountId,
                    replacedAtNonce: nonce,
                    slotName: entry.slotName,
                    key: entry.key,
                    oldValue: oldEntry?.value ?? null,
                });
                // "" means removal
                if (entry.value === "") {
                    await db.latestStorageMapEntries
                        .where("[accountId+slotName+key]")
                        .equals([accountId, entry.slotName, entry.key])
                        .delete();
                }
                else {
                    await db.latestStorageMapEntries.put({
                        accountId,
                        slotName: entry.slotName,
                        key: entry.key,
                        value: entry.value,
                    });
                }
            }
            // Apply vault delta: read old → archive → update latest
            for (const entry of changedAssets) {
                const oldAsset = await db.latestAccountAssets
                    .where("[accountId+vaultKey]")
                    .equals([accountId, entry.vaultKey])
                    .first();
                await db.historicalAccountAssets.put({
                    accountId,
                    replacedAtNonce: nonce,
                    vaultKey: entry.vaultKey,
                    faucetIdPrefix: entry.faucetIdPrefix,
                    oldAsset: oldAsset?.asset ?? null,
                });
                // "" means removal
                if (entry.asset === "") {
                    await db.latestAccountAssets
                        .where("[accountId+vaultKey]")
                        .equals([accountId, entry.vaultKey])
                        .delete();
                }
                else {
                    await db.latestAccountAssets.put({
                        accountId,
                        vaultKey: entry.vaultKey,
                        faucetIdPrefix: entry.faucetIdPrefix,
                        asset: entry.asset,
                    });
                }
            }
            // Archive old header and write new header
            const oldHeader = await db.latestAccountHeaders
                .where("id")
                .equals(accountId)
                .first();
            if (oldHeader) {
                await db.historicalAccountHeaders.put({
                    id: accountId,
                    replacedAtNonce: nonce,
                    codeRoot: oldHeader.codeRoot,
                    storageRoot: oldHeader.storageRoot,
                    vaultRoot: oldHeader.vaultRoot,
                    nonce: oldHeader.nonce,
                    committed: oldHeader.committed,
                    accountSeed: oldHeader.accountSeed,
                    accountCommitment: oldHeader.accountCommitment,
                    locked: oldHeader.locked,
                });
            }
            await db.latestAccountHeaders.put({
                id: accountId,
                codeRoot,
                storageRoot,
                vaultRoot,
                nonce,
                committed,
                accountSeed: undefined,
                accountCommitment: commitment,
                locked: false,
            });
        });
    }
    catch (error) {
        logWebStoreError(error, `Error applying transaction delta`);
    }
}
async function archiveAndReplaceStorageSlots(db, accountId, nonce, newSlots) {
    const oldSlots = await db.latestAccountStorages
        .where("accountId")
        .equals(accountId)
        .toArray();
    // Archive every old slot
    for (const slot of oldSlots) {
        await db.historicalAccountStorages.put({
            accountId,
            replacedAtNonce: nonce,
            slotName: slot.slotName,
            oldSlotValue: slot.slotValue,
            slotType: slot.slotType,
        });
    }
    // Write NULL markers for genuinely new slots (no old value to archive)
    const oldSlotNames = new Set(oldSlots.map((s) => s.slotName));
    for (const slot of newSlots) {
        if (!oldSlotNames.has(slot.slotName)) {
            await db.historicalAccountStorages.put({
                accountId,
                replacedAtNonce: nonce,
                slotName: slot.slotName,
                oldSlotValue: null,
                slotType: slot.slotType,
            });
        }
    }
    // Replace latest
    await db.latestAccountStorages.where("accountId").equals(accountId).delete();
    if (newSlots.length > 0) {
        await db.latestAccountStorages.bulkPut(newSlots.map((slot) => ({
            accountId,
            slotName: slot.slotName,
            slotValue: slot.slotValue,
            slotType: slot.slotType,
        })));
    }
}
async function archiveAndReplaceMapEntries(db, accountId, nonce, newEntries) {
    const oldEntries = await db.latestStorageMapEntries
        .where("accountId")
        .equals(accountId)
        .toArray();
    for (const entry of oldEntries) {
        await db.historicalStorageMapEntries.put({
            accountId,
            replacedAtNonce: nonce,
            slotName: entry.slotName,
            key: entry.key,
            oldValue: entry.value,
        });
    }
    const oldKeys = new Set(oldEntries.map((e) => `${e.slotName}\0${e.key}`));
    for (const entry of newEntries) {
        if (!oldKeys.has(`${entry.slotName}\0${entry.key}`)) {
            await db.historicalStorageMapEntries.put({
                accountId,
                replacedAtNonce: nonce,
                slotName: entry.slotName,
                key: entry.key,
                oldValue: null,
            });
        }
    }
    await db.latestStorageMapEntries
        .where("accountId")
        .equals(accountId)
        .delete();
    if (newEntries.length > 0) {
        await db.latestStorageMapEntries.bulkPut(newEntries.map((entry) => ({
            accountId,
            slotName: entry.slotName,
            key: entry.key,
            value: entry.value,
        })));
    }
}
async function archiveAndReplaceVaultAssets(db, accountId, nonce, newAssets) {
    const oldAssets = await db.latestAccountAssets
        .where("accountId")
        .equals(accountId)
        .toArray();
    for (const asset of oldAssets) {
        await db.historicalAccountAssets.put({
            accountId,
            replacedAtNonce: nonce,
            vaultKey: asset.vaultKey,
            faucetIdPrefix: asset.faucetIdPrefix,
            oldAsset: asset.asset,
        });
    }
    const oldKeys = new Set(oldAssets.map((a) => a.vaultKey));
    for (const asset of newAssets) {
        if (!oldKeys.has(asset.vaultKey)) {
            await db.historicalAccountAssets.put({
                accountId,
                replacedAtNonce: nonce,
                vaultKey: asset.vaultKey,
                faucetIdPrefix: asset.faucetIdPrefix,
                oldAsset: null,
            });
        }
    }
    await db.latestAccountAssets.where("accountId").equals(accountId).delete();
    if (newAssets.length > 0) {
        await db.latestAccountAssets.bulkPut(newAssets.map((asset) => ({
            accountId,
            vaultKey: asset.vaultKey,
            faucetIdPrefix: asset.faucetIdPrefix,
            asset: asset.asset,
        })));
    }
}
async function restoreSlotsFromHistorical(db, accountId, nonce) {
    const oldSlots = await db.historicalAccountStorages
        .where("[accountId+replacedAtNonce]")
        .equals([accountId, nonce])
        .toArray();
    for (const slot of oldSlots) {
        if (slot.oldSlotValue !== null) {
            await db.latestAccountStorages.put({
                accountId: slot.accountId,
                slotName: slot.slotName,
                slotValue: slot.oldSlotValue,
                slotType: slot.slotType,
            });
        }
        else {
            await db.latestAccountStorages
                .where("[accountId+slotName]")
                .equals([accountId, slot.slotName])
                .delete();
        }
    }
}
async function restoreMapEntriesFromHistorical(db, accountId, nonce) {
    const oldEntries = await db.historicalStorageMapEntries
        .where("[accountId+replacedAtNonce]")
        .equals([accountId, nonce])
        .toArray();
    for (const entry of oldEntries) {
        if (entry.oldValue !== null) {
            await db.latestStorageMapEntries.put({
                accountId: entry.accountId,
                slotName: entry.slotName,
                key: entry.key,
                value: entry.oldValue,
            });
        }
        else {
            await db.latestStorageMapEntries
                .where("[accountId+slotName+key]")
                .equals([accountId, entry.slotName, entry.key])
                .delete();
        }
    }
}
async function restoreAssetsFromHistorical(db, accountId, nonce) {
    const oldAssets = await db.historicalAccountAssets
        .where("[accountId+replacedAtNonce]")
        .equals([accountId, nonce])
        .toArray();
    for (const asset of oldAssets) {
        if (asset.oldAsset !== null) {
            await db.latestAccountAssets.put({
                accountId: asset.accountId,
                vaultKey: asset.vaultKey,
                faucetIdPrefix: asset.faucetIdPrefix,
                asset: asset.oldAsset,
            });
        }
        else {
            await db.latestAccountAssets
                .where("[accountId+vaultKey]")
                .equals([accountId, asset.vaultKey])
                .delete();
        }
    }
}
/**
 * Replaces an account's full state (storage, map entries, vault assets, header)
 * with a new snapshot. Before overwriting, all current latest values are archived
 * to historical.
 */
export async function applyFullAccountState(dbId, accountState) {
    try {
        const db = getDatabase(dbId);
        const { accountId, nonce, storageSlots, storageMapEntries, assets, codeRoot, storageRoot, vaultRoot, committed, accountCommitment, accountSeed, } = accountState;
        await db.dexie.transaction("rw", [
            db.latestAccountStorages,
            db.historicalAccountStorages,
            db.latestStorageMapEntries,
            db.historicalStorageMapEntries,
            db.latestAccountAssets,
            db.historicalAccountAssets,
            db.latestAccountHeaders,
            db.historicalAccountHeaders,
        ], async () => {
            // Archive: save current latest values to historical (so they can be
            // restored on undo), then replace latest with the new state.
            await archiveAndReplaceStorageSlots(db, accountId, nonce, storageSlots);
            await archiveAndReplaceMapEntries(db, accountId, nonce, storageMapEntries);
            await archiveAndReplaceVaultAssets(db, accountId, nonce, assets);
            // Archive old header and write new header
            const oldHeader = await db.latestAccountHeaders
                .where("id")
                .equals(accountId)
                .first();
            if (oldHeader) {
                await db.historicalAccountHeaders.put({
                    id: accountId,
                    replacedAtNonce: nonce,
                    codeRoot: oldHeader.codeRoot,
                    storageRoot: oldHeader.storageRoot,
                    vaultRoot: oldHeader.vaultRoot,
                    nonce: oldHeader.nonce,
                    committed: oldHeader.committed,
                    accountSeed: oldHeader.accountSeed,
                    accountCommitment: oldHeader.accountCommitment,
                    locked: oldHeader.locked,
                });
            }
            await db.latestAccountHeaders.put({
                id: accountId,
                codeRoot,
                storageRoot,
                vaultRoot,
                nonce,
                committed,
                accountSeed,
                accountCommitment,
                locked: false,
            });
        });
    }
    catch (error) {
        logWebStoreError(error, `Error applying full account state`);
    }
}
export async function upsertAccountRecord(dbId, accountId, codeRoot, storageRoot, vaultRoot, nonce, committed, commitment, accountSeed) {
    try {
        const db = getDatabase(dbId);
        const data = {
            id: accountId,
            codeRoot,
            storageRoot,
            vaultRoot,
            nonce,
            committed,
            accountSeed,
            accountCommitment: commitment,
            locked: false,
        };
        await db.latestAccountHeaders.put(data);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting account: ${accountId}`);
    }
}
export async function insertAccountAddress(dbId, accountId, address) {
    try {
        const db = getDatabase(dbId);
        const data = {
            id: accountId,
            address,
        };
        await db.addresses.put(data);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting address with value: ${String(address)} for the account ID ${accountId}`);
    }
}
export async function removeAccountAddress(dbId, address) {
    try {
        const db = getDatabase(dbId);
        await db.addresses.where("address").equals(address).delete();
    }
    catch (error) {
        logWebStoreError(error, `Error removing address with value: ${String(address)}`);
    }
}
export async function upsertForeignAccountCode(dbId, accountId, code, codeRoot) {
    try {
        const db = getDatabase(dbId);
        await upsertAccountCode(dbId, codeRoot, code);
        const data = {
            accountId,
            codeRoot,
        };
        await db.foreignAccountCode.put(data);
    }
    catch (error) {
        logWebStoreError(error, `Error upserting foreign account code for account: ${accountId}`);
    }
}
export async function getForeignAccountCode(dbId, accountIds) {
    try {
        const db = getDatabase(dbId);
        const foreignAccounts = await db.foreignAccountCode
            .where("accountId")
            .anyOf(accountIds)
            .toArray();
        if (foreignAccounts.length === 0) {
            console.log("No records found for the given account IDs.");
            return null;
        }
        const codeRoots = foreignAccounts.map((account) => account.codeRoot);
        const accountCode = await db.accountCodes
            .where("root")
            .anyOf(codeRoots)
            .toArray();
        const processedCode = foreignAccounts
            .map((foreignAccount) => {
            const matchingCode = accountCode.find((code) => code.root === foreignAccount.codeRoot);
            if (matchingCode === undefined) {
                return undefined;
            }
            const codeBase64 = uint8ArrayToBase64(matchingCode.code);
            return {
                accountId: foreignAccount.accountId,
                code: codeBase64,
            };
        })
            .filter((matchingCode) => matchingCode !== undefined);
        return processedCode;
    }
    catch (error) {
        logWebStoreError(error, "Error fetching foreign account code");
    }
}
export async function lockAccount(dbId, accountId) {
    try {
        const db = getDatabase(dbId);
        await db.latestAccountHeaders
            .where("id")
            .equals(accountId)
            .modify({ locked: true });
        // Also lock historical rows so that undo/rebuild preserves the lock.
        await db.historicalAccountHeaders
            .where("id")
            .equals(accountId)
            .modify({ locked: true });
    }
    catch (error) {
        logWebStoreError(error, `Error locking account: ${accountId}`);
    }
}
/**
 * Rebuilds latest storage slots from historical data.
 * Groups by slotName, takes the entry with MAX(nonce) per slot.
 * Slots cannot be removed, so no tombstone filtering needed.
 */
export async function rebuildLatestStorageSlots(db, accountId) {
    await db.latestAccountStorages.where("accountId").equals(accountId).delete();
    const allHist = await db.historicalAccountStorages
        .where("accountId")
        .equals(accountId)
        .toArray();
    // Group by slotName, take MAX(replacedAtNonce) per slot
    const bySlot = new Map();
    for (const entry of allHist) {
        const existing = bySlot.get(entry.slotName);
        if (!existing ||
            BigInt(entry.replacedAtNonce) > BigInt(existing.replacedAtNonce)) {
            bySlot.set(entry.slotName, entry);
        }
    }
    if (bySlot.size > 0) {
        const entries = [...bySlot.values()].map(
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        ({ replacedAtNonce, oldSlotValue, ...rest }) => ({
            ...rest,
            slotValue: oldSlotValue ?? "",
        }));
        await db.latestAccountStorages.bulkPut(entries);
    }
}
/**
 * Rebuilds latest storage map entries from historical data.
 * Groups by (slotName, key), takes the entry with MAX(nonce) per key.
 * Filters out tombstones (value === null).
 */
export async function rebuildLatestStorageMapEntries(db, accountId) {
    await db.latestStorageMapEntries
        .where("accountId")
        .equals(accountId)
        .delete();
    const allHist = await db.historicalStorageMapEntries
        .where("accountId")
        .equals(accountId)
        .toArray();
    // Group by (slotName, key), take MAX(replacedAtNonce) per key
    const byKey = new Map();
    for (const entry of allHist) {
        const compositeKey = `${entry.slotName}\0${entry.key}`;
        const existing = byKey.get(compositeKey);
        if (!existing ||
            BigInt(entry.replacedAtNonce) > BigInt(existing.replacedAtNonce)) {
            byKey.set(compositeKey, entry);
        }
    }
    // Filter out tombstones and strip replacedAtNonce
    const entries = [...byKey.values()]
        .filter((e) => e.oldValue !== null)
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        .map(({ replacedAtNonce, oldValue, ...rest }) => ({
        ...rest,
        value: oldValue,
    }));
    if (entries.length > 0) {
        await db.latestStorageMapEntries.bulkPut(entries);
    }
}
/**
 * Rebuilds latest vault assets from historical data.
 * Groups by vaultKey, takes the entry with MAX(nonce) per key.
 * Filters out tombstones (asset === null).
 */
export async function rebuildLatestVaultAssets(db, accountId) {
    await db.latestAccountAssets.where("accountId").equals(accountId).delete();
    const allHist = await db.historicalAccountAssets
        .where("accountId")
        .equals(accountId)
        .toArray();
    // Group by vaultKey, take MAX(replacedAtNonce) per key
    const byKey = new Map();
    for (const entry of allHist) {
        const existing = byKey.get(entry.vaultKey);
        if (!existing ||
            BigInt(entry.replacedAtNonce) > BigInt(existing.replacedAtNonce)) {
            byKey.set(entry.vaultKey, entry);
        }
    }
    // Filter out tombstones and strip replacedAtNonce
    const entries = [...byKey.values()]
        .filter((e) => e.oldAsset !== null)
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        .map(({ replacedAtNonce, oldAsset, ...rest }) => ({
        ...rest,
        asset: oldAsset,
    }));
    if (entries.length > 0) {
        await db.latestAccountAssets.bulkPut(entries);
    }
}
/**
 * Prunes old committed historical states for one or all accounts.
 */
export async function pruneAccountHistory(dbId, accountId, pendingNoncesJson) {
    try {
        const db = getDatabase(dbId);
        let totalDeleted = 0;
        // Parse the pending-nonces map supplied by the Rust caller.
        const pendingMap = JSON.parse(pendingNoncesJson);
        await db.dexie.transaction("rw", [
            db.historicalAccountHeaders,
            db.historicalAccountStorages,
            db.historicalStorageMapEntries,
            db.historicalAccountAssets,
            db.accountCodes,
            db.latestAccountHeaders,
            db.foreignAccountCode,
        ], async () => {
            // Determine which accounts to prune
            let accountIds;
            if (accountId != null) {
                accountIds = [accountId];
            }
            else {
                const allHeaders = await db.historicalAccountHeaders.toArray();
                accountIds = [...new Set(allHeaders.map((h) => h.id))];
            }
            for (const aid of accountIds) {
                const headers = await db.historicalAccountHeaders
                    .where("id")
                    .equals(aid)
                    .toArray();
                if (headers.length <= 1)
                    continue;
                // Build set of pending nonces for this account.
                const pending = new Set((pendingMap[aid] ?? []).map((n) => BigInt(n)));
                // Collect all nonces, sorted descending.
                const nonces = headers
                    .map((h) => BigInt(h.nonce))
                    .sort((a, b) => (a > b ? -1 : a < b ? 1 : 0));
                // The boundary is the highest nonce that is NOT pending.
                const boundaryNonce = nonces.find((n) => !pending.has(n));
                if (boundaryNonce === undefined)
                    continue;
                // Everything below the boundary should be deleted.
                // Since pending nonces always form a contiguous suffix above the boundary,
                // there can't be any pending nonces below it.
                const toDeleteNonces = nonces.filter((n) => n < boundaryNonce);
                if (toDeleteNonces.length === 0)
                    continue;
                // Build a fast lookup from nonce → header for deletion.
                const headerByNonce = new Map(headers.map((h) => [BigInt(h.nonce), h]));
                for (const nonce of toDeleteNonces) {
                    const old = headerByNonce.get(nonce);
                    const replacedAtNonce = old.replacedAtNonce;
                    await db.historicalAccountHeaders
                        .where("accountCommitment")
                        .equals(old.accountCommitment)
                        .delete();
                    const storageDeleted = await db.historicalAccountStorages
                        .where("[accountId+replacedAtNonce]")
                        .equals([aid, replacedAtNonce])
                        .delete();
                    const mapDeleted = await db.historicalStorageMapEntries
                        .where("[accountId+replacedAtNonce]")
                        .equals([aid, replacedAtNonce])
                        .delete();
                    const assetDeleted = await db.historicalAccountAssets
                        .where("[accountId+replacedAtNonce]")
                        .equals([aid, replacedAtNonce])
                        .delete();
                    totalDeleted += 1 + storageDeleted + mapDeleted + assetDeleted;
                }
            }
            // Prune orphaned account code
            const latestHeaders = await db.latestAccountHeaders.toArray();
            const historicalHeaders = await db.historicalAccountHeaders.toArray();
            const foreignCodes = await db.foreignAccountCode.toArray();
            const referencedCodeRoots = new Set();
            for (const h of latestHeaders)
                referencedCodeRoots.add(h.codeRoot);
            for (const h of historicalHeaders)
                referencedCodeRoots.add(h.codeRoot);
            for (const f of foreignCodes)
                referencedCodeRoots.add(f.codeRoot);
            const allCodes = await db.accountCodes.toArray();
            for (const code of allCodes) {
                if (!referencedCodeRoots.has(code.root)) {
                    await db.accountCodes.where("root").equals(code.root).delete();
                    totalDeleted += 1;
                }
            }
        });
        return totalDeleted;
    }
    catch (error) {
        logWebStoreError(error, `Error pruning account history for ${accountId ?? "all accounts"}`);
        throw error;
    }
}
/**
 * Undoes discarded account states by restoring old values from historical
 * back to latest. Non-null old values overwrite latest; null old values
 * (entries that didn't exist before that nonce) cause deletion from latest.
 */
export async function undoAccountStates(dbId, accountCommitments) {
    try {
        const db = getDatabase(dbId);
        await db.dexie.transaction("rw", [
            db.latestAccountStorages,
            db.historicalAccountStorages,
            db.latestStorageMapEntries,
            db.historicalStorageMapEntries,
            db.latestAccountAssets,
            db.historicalAccountAssets,
            db.latestAccountHeaders,
            db.historicalAccountHeaders,
        ], async () => {
            // Step 1: Resolve nonces from both latest and historical headers
            const accountNonces = new Map();
            for (const commitment of accountCommitments) {
                const latestRecord = await db.latestAccountHeaders
                    .where("accountCommitment")
                    .equals(commitment)
                    .first();
                if (latestRecord) {
                    if (!accountNonces.has(latestRecord.id)) {
                        accountNonces.set(latestRecord.id, new Set());
                    }
                    accountNonces.get(latestRecord.id).add(latestRecord.nonce);
                    continue;
                }
                const histRecord = await db.historicalAccountHeaders
                    .where("accountCommitment")
                    .equals(commitment)
                    .first();
                if (histRecord) {
                    if (!accountNonces.has(histRecord.id)) {
                        accountNonces.set(histRecord.id, new Set());
                    }
                    accountNonces.get(histRecord.id).add(histRecord.nonce);
                }
            }
            // Step 2: Group nonces by account, sort descending (undo most recent first).
            // Descending order is needed because each nonce's old value is the state before
            // that nonce — processing most recent first lets earlier nonces overwrite with
            // the correct final value.
            for (const [accountId, nonces] of accountNonces) {
                const sortedNonces = [...nonces].sort((a, b) => Number(BigInt(b) - BigInt(a)));
                // Step 3: Restore old values from historical back to latest, undoing
                // each nonce in descending order. Non-null old values overwrite latest;
                // null old values (entries that didn't exist before) cause deletion.
                for (const nonce of sortedNonces) {
                    await restoreSlotsFromHistorical(db, accountId, nonce);
                    await restoreMapEntriesFromHistorical(db, accountId, nonce);
                    await restoreAssetsFromHistorical(db, accountId, nonce);
                }
                // Step 4: Restore old header from the earliest discarded nonce
                const minNonce = sortedNonces[sortedNonces.length - 1];
                const oldHeader = await db.historicalAccountHeaders
                    .where("[id+replacedAtNonce]")
                    .equals([accountId, minNonce])
                    .first();
                if (oldHeader) {
                    await db.latestAccountHeaders.put({
                        id: oldHeader.id,
                        codeRoot: oldHeader.codeRoot,
                        storageRoot: oldHeader.storageRoot,
                        vaultRoot: oldHeader.vaultRoot,
                        nonce: oldHeader.nonce,
                        committed: oldHeader.committed,
                        accountSeed: oldHeader.accountSeed,
                        accountCommitment: oldHeader.accountCommitment,
                        locked: oldHeader.locked,
                    });
                }
                else {
                    // No previous state — delete the account entirely
                    await db.latestAccountHeaders
                        .where("id")
                        .equals(accountId)
                        .delete();
                    await db.latestAccountStorages
                        .where("accountId")
                        .equals(accountId)
                        .delete();
                    await db.latestStorageMapEntries
                        .where("accountId")
                        .equals(accountId)
                        .delete();
                    await db.latestAccountAssets
                        .where("accountId")
                        .equals(accountId)
                        .delete();
                }
                // Step 5: Delete consumed historical entries at discarded nonces
                for (const nonce of sortedNonces) {
                    await db.historicalAccountStorages
                        .where("[accountId+replacedAtNonce]")
                        .equals([accountId, nonce])
                        .delete();
                    await db.historicalStorageMapEntries
                        .where("[accountId+replacedAtNonce]")
                        .equals([accountId, nonce])
                        .delete();
                    await db.historicalAccountAssets
                        .where("[accountId+replacedAtNonce]")
                        .equals([accountId, nonce])
                        .delete();
                    await db.historicalAccountHeaders
                        .where("[id+replacedAtNonce]")
                        .equals([accountId, nonce])
                        .delete();
                }
            }
        });
    }
    catch (error) {
        logWebStoreError(error, `Error undoing account states: ${accountCommitments.join(",")}`);
        throw error;
    }
}
