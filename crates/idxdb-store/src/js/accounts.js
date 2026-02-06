import { getDatabase, } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";
function isNewerNonce(candidate, current) {
    return BigInt(candidate) > BigInt(current);
}
function toHeaderObject(record) {
    let accountSeedBase64 = undefined;
    if (record.accountSeed && record.accountSeed.length > 0) {
        accountSeedBase64 = uint8ArrayToBase64(record.accountSeed);
    }
    return {
        id: record.id,
        nonce: record.nonce,
        vaultRoot: record.vaultRoot,
        storageRoot: record.storageRoot || "",
        codeRoot: record.codeRoot || "",
        accountSeed: accountSeedBase64,
        locked: record.locked,
        committed: record.committed,
        accountCommitment: record.accountCommitment || "",
    };
}
function pickLatestRecord(records) {
    if (records.length === 0) {
        return undefined;
    }
    return records.reduce((latest, record) => isNewerNonce(record.nonce, latest.nonce) ? record : latest);
}
export async function getAccountIds(dbId) {
    try {
        const db = getDatabase(dbId);
        const tracked = await db.trackedAccounts.toArray();
        return tracked.map((entry) => entry.id);
    }
    catch (error) {
        logWebStoreError(error, "Error while fetching account IDs");
    }
    return [];
}
export async function getAllAccountHeaders(dbId) {
    try {
        const db = getDatabase(dbId);
        const latestRecords = await db.accountsLatest.toArray();
        return latestRecords.map(toHeaderObject);
    }
    catch (error) {
        logWebStoreError(error, "Error while fetching account headers");
    }
}
export async function getAccountHeader(dbId, accountId) {
    try {
        const db = getDatabase(dbId);
        const latestRecord = await db.accountsLatest.get(accountId);
        if (!latestRecord) {
            console.log("No account header record found for given ID.");
            return null;
        }
        return toHeaderObject(latestRecord);
    }
    catch (error) {
        logWebStoreError(error, `Error while fetching account header for id: ${accountId}`);
    }
}
export async function getAccountHeaderByCommitment(dbId, accountCommitment) {
    try {
        const db = getDatabase(dbId);
        const matchingRecord = await db.accountsHistory
            .where("accountCommitment")
            .equals(accountCommitment)
            .first();
        if (!matchingRecord) {
            return undefined;
        }
        return toHeaderObject(matchingRecord);
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
export async function getAccountStorage(dbId, storageCommitment) {
    try {
        const db = getDatabase(dbId);
        const allMatchingRecords = await db.accountStorages
            .where("commitment")
            .equals(storageCommitment)
            .toArray();
        const slots = allMatchingRecords.map((record) => {
            return {
                slotName: record.slotName,
                slotValue: record.slotValue,
                slotType: record.slotType,
            };
        });
        return slots;
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account storage for commitment ${storageCommitment}`);
    }
}
export async function getAccountStorageMaps(dbId, roots) {
    try {
        const db = getDatabase(dbId);
        const allMatchingRecords = await db.storageMapEntries
            .where("root")
            .anyOf(roots)
            .toArray();
        return allMatchingRecords;
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account storage maps for roots ${roots.join(", ")}`);
    }
}
export async function getAccountVaultAssets(dbId, vaultRoot) {
    try {
        const db = getDatabase(dbId);
        const allMatchingRecords = await db.accountAssets
            .where("root")
            .equals(vaultRoot)
            .toArray();
        const assets = allMatchingRecords.map((record) => {
            return {
                asset: record.asset,
            };
        });
        return assets;
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account vault for root ${vaultRoot}`);
    }
}
export async function getAccountAuthByPubKeyCommitment(dbId, pubKeyCommitmentHex) {
    const db = getDatabase(dbId);
    const accountSecretKey = await db.accountAuths
        .where("pubKeyCommitmentHex")
        .equals(pubKeyCommitmentHex)
        .first();
    if (!accountSecretKey) {
        throw new Error("Account auth not found in cache.");
    }
    const data = {
        secretKey: accountSecretKey.secretKeyHex,
    };
    return data;
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
export async function upsertAccountStorage(dbId, storageSlots) {
    try {
        const db = getDatabase(dbId);
        let processedSlots = storageSlots.map((slot) => {
            return {
                commitment: slot.commitment,
                slotName: slot.slotName,
                slotValue: slot.slotValue,
                slotType: slot.slotType,
            };
        });
        await db.accountStorages.bulkPut(processedSlots);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting storage slots`);
    }
}
export async function upsertStorageMapEntries(dbId, entries) {
    try {
        const db = getDatabase(dbId);
        let processedEntries = entries.map((entry) => {
            return {
                root: entry.root,
                key: entry.key,
                value: entry.value,
            };
        });
        await db.storageMapEntries.bulkPut(processedEntries);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting storage map entries`);
    }
}
export async function upsertVaultAssets(dbId, assets) {
    try {
        const db = getDatabase(dbId);
        let processedAssets = assets.map((asset) => {
            return {
                root: asset.root,
                vaultKey: asset.vaultKey,
                faucetIdPrefix: asset.faucetIdPrefix,
                asset: asset.asset,
            };
        });
        await db.accountAssets.bulkPut(processedAssets);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting assets`);
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
        await db.accountsHistory.put(data);
        const currentLatest = await db.accountsLatest.get(accountId);
        if (!currentLatest || !isNewerNonce(currentLatest.nonce, nonce)) {
            await db.accountsLatest.put(data);
        }
        await db.trackedAccounts.put({ id: accountId });
    }
    catch (error) {
        logWebStoreError(error, `Error inserting account: ${accountId}`);
    }
}
export async function insertAccountAuth(dbId, pubKeyCommitmentHex, secretKey) {
    try {
        const db = getDatabase(dbId);
        const data = {
            pubKeyCommitmentHex,
            secretKeyHex: secretKey,
        };
        await db.accountAuths.add(data);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting account auth for pubKey: ${pubKeyCommitmentHex}`);
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
        await db.accountsLatest
            .where("id")
            .equals(accountId)
            .modify({ locked: true });
    }
    catch (error) {
        logWebStoreError(error, `Error locking account: ${accountId}`);
    }
}
export async function undoAccountStates(dbId, accountCommitments) {
    try {
        const db = getDatabase(dbId);
        await db.dexie.transaction("rw", db.accountsHistory, db.accountsLatest, async () => {
            const removedStates = await db.accountsHistory
                .where("accountCommitment")
                .anyOf(accountCommitments)
                .toArray();
            if (removedStates.length === 0) {
                return;
            }
            await db.accountsHistory
                .where("accountCommitment")
                .anyOf(accountCommitments)
                .delete();
            const affectedAccountIds = Array.from(new Set(removedStates.map((state) => state.id)));
            for (const accountId of affectedAccountIds) {
                const historyRows = await db.accountsHistory
                    .where("id")
                    .equals(accountId)
                    .toArray();
                const latest = pickLatestRecord(historyRows);
                if (latest) {
                    await db.accountsLatest.put(latest);
                }
                else {
                    await db.accountsLatest.delete(accountId);
                }
            }
        });
    }
    catch (error) {
        logWebStoreError(error, `Error undoing account states: ${accountCommitments.join(",")}`);
    }
}
