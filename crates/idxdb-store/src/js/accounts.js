import { getDatabase, } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";
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
        const resultObject = records.map((record) => {
            let accountSeedBase64 = undefined;
            if (record.accountSeed) {
                const seedAsBytes = new Uint8Array(record.accountSeed);
                if (seedAsBytes.length > 0) {
                    accountSeedBase64 = uint8ArrayToBase64(seedAsBytes);
                }
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
        });
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
        let accountSeedBase64 = undefined;
        if (record.accountSeed) {
            if (record.accountSeed.length > 0) {
                accountSeedBase64 = uint8ArrayToBase64(record.accountSeed);
            }
        }
        return {
            id: record.id,
            nonce: record.nonce,
            vaultRoot: record.vaultRoot,
            storageRoot: record.storageRoot,
            codeRoot: record.codeRoot,
            accountSeed: accountSeedBase64,
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
        let accountSeedBase64 = undefined;
        if (record.accountSeed) {
            accountSeedBase64 = uint8ArrayToBase64(record.accountSeed);
        }
        return {
            id: record.id,
            nonce: record.nonce,
            vaultRoot: record.vaultRoot,
            storageRoot: record.storageRoot,
            codeRoot: record.codeRoot,
            accountSeed: accountSeedBase64,
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
export async function getAccountStorage(dbId, accountId) {
    try {
        const db = getDatabase(dbId);
        const allMatchingRecords = await db.latestAccountStorages
            .where("accountId")
            .equals(accountId)
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
export async function getAccountVaultAssets(dbId, accountId) {
    try {
        const db = getDatabase(dbId);
        const allMatchingRecords = await db.latestAccountAssets
            .where("accountId")
            .equals(accountId)
            .toArray();
        const assets = allMatchingRecords.map((record) => {
            return {
                asset: record.asset,
            };
        });
        return assets;
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account vault for account ${accountId}`);
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
        if (storageSlots.length === 0)
            return;
        const db = getDatabase(dbId);
        const accountId = storageSlots[0].accountId;
        const latestEntries = storageSlots.map((slot) => {
            return {
                accountId: slot.accountId,
                slotName: slot.slotName,
                slotValue: slot.slotValue,
                slotType: slot.slotType,
            };
        });
        const historicalEntries = storageSlots.map((slot) => {
            return {
                accountId: slot.accountId,
                nonce: slot.nonce,
                slotName: slot.slotName,
                slotValue: slot.slotValue,
                slotType: slot.slotType,
            };
        });
        await db.latestAccountStorages
            .where("accountId")
            .equals(accountId)
            .delete();
        await db.latestAccountStorages.bulkPut(latestEntries);
        await db.historicalAccountStorages.bulkPut(historicalEntries);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting storage slots`);
    }
}
export async function upsertStorageMapEntries(dbId, entries) {
    try {
        if (entries.length === 0)
            return;
        const db = getDatabase(dbId);
        const accountId = entries[0].accountId;
        const latestEntries = entries.map((entry) => {
            return {
                accountId: entry.accountId,
                slotName: entry.slotName,
                key: entry.key,
                value: entry.value,
            };
        });
        const historicalEntries = entries.map((entry) => {
            return {
                accountId: entry.accountId,
                nonce: entry.nonce,
                slotName: entry.slotName,
                key: entry.key,
                value: entry.value,
            };
        });
        await db.latestStorageMapEntries
            .where("accountId")
            .equals(accountId)
            .delete();
        await db.latestStorageMapEntries.bulkPut(latestEntries);
        await db.historicalStorageMapEntries.bulkPut(historicalEntries);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting storage map entries`);
    }
}
export async function upsertVaultAssets(dbId, assets) {
    try {
        if (assets.length === 0)
            return;
        const db = getDatabase(dbId);
        const accountId = assets[0].accountId;
        const latestEntries = assets.map((asset) => {
            return {
                accountId: asset.accountId,
                vaultKey: asset.vaultKey,
                faucetIdPrefix: asset.faucetIdPrefix,
                asset: asset.asset,
            };
        });
        const historicalEntries = assets.map((asset) => {
            return {
                accountId: asset.accountId,
                nonce: asset.nonce,
                vaultKey: asset.vaultKey,
                faucetIdPrefix: asset.faucetIdPrefix,
                asset: asset.asset,
            };
        });
        await db.latestAccountAssets.where("accountId").equals(accountId).delete();
        await db.latestAccountAssets.bulkPut(latestEntries);
        await db.historicalAccountAssets.bulkPut(historicalEntries);
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
        await db.historicalAccountHeaders.put(data);
        await db.latestAccountHeaders.put(data);
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
        await db.latestAccountHeaders
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
        // Find affected records to get their account IDs and nonces before deleting
        const affectedRecords = await db.historicalAccountHeaders
            .where("accountCommitment")
            .anyOf(accountCommitments)
            .toArray();
        // Collect affected (accountId, nonce) pairs
        const accountNonces = new Map();
        for (const record of affectedRecords) {
            if (!accountNonces.has(record.id)) {
                accountNonces.set(record.id, new Set());
            }
            accountNonces.get(record.id).add(record.nonce);
        }
        // Delete matching records from historical account headers
        await db.historicalAccountHeaders
            .where("accountCommitment")
            .anyOf(accountCommitments)
            .delete();
        // Delete historical storage/map/assets for affected (accountId, nonce) pairs
        for (const [accountId, nonces] of accountNonces) {
            for (const nonce of nonces) {
                await db.historicalAccountStorages
                    .where("[accountId+nonce]")
                    .equals([accountId, nonce])
                    .delete();
                await db.historicalStorageMapEntries
                    .where("[accountId+nonce]")
                    .equals([accountId, nonce])
                    .delete();
                await db.historicalAccountAssets
                    .where("[accountId+nonce]")
                    .equals([accountId, nonce])
                    .delete();
            }
        }
        // Rebuild latest for each affected account
        for (const accountId of accountNonces.keys()) {
            const remaining = await db.historicalAccountHeaders
                .where("id")
                .equals(accountId)
                .toArray();
            if (remaining.length === 0) {
                // Account completely undone — clear all latest tables
                await db.latestAccountHeaders.where("id").equals(accountId).delete();
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
            else {
                // Find the record with the highest nonce
                let maxRecord = remaining[0];
                for (const record of remaining) {
                    if (BigInt(record.nonce) > BigInt(maxRecord.nonce)) {
                        maxRecord = record;
                    }
                }
                const maxNonce = maxRecord.nonce;
                // Rebuild latest account header
                await db.latestAccountHeaders.put(maxRecord);
                // Rebuild latest storage
                await db.latestAccountStorages
                    .where("accountId")
                    .equals(accountId)
                    .delete();
                const histStorage = await db.historicalAccountStorages
                    .where("[accountId+nonce]")
                    .equals([accountId, maxNonce])
                    .toArray();
                if (histStorage.length > 0) {
                    await db.latestAccountStorages.bulkPut(histStorage.map((s) => ({
                        accountId: s.accountId,
                        slotName: s.slotName,
                        slotValue: s.slotValue,
                        slotType: s.slotType,
                    })));
                }
                // Rebuild latest storage map entries
                await db.latestStorageMapEntries
                    .where("accountId")
                    .equals(accountId)
                    .delete();
                const histMapEntries = await db.historicalStorageMapEntries
                    .where("[accountId+nonce]")
                    .equals([accountId, maxNonce])
                    .toArray();
                if (histMapEntries.length > 0) {
                    await db.latestStorageMapEntries.bulkPut(histMapEntries.map((e) => ({
                        accountId: e.accountId,
                        slotName: e.slotName,
                        key: e.key,
                        value: e.value,
                    })));
                }
                // Rebuild latest assets
                await db.latestAccountAssets
                    .where("accountId")
                    .equals(accountId)
                    .delete();
                const histAssets = await db.historicalAccountAssets
                    .where("[accountId+nonce]")
                    .equals([accountId, maxNonce])
                    .toArray();
                if (histAssets.length > 0) {
                    await db.latestAccountAssets.bulkPut(histAssets.map((a) => ({
                        accountId: a.accountId,
                        vaultKey: a.vaultKey,
                        faucetIdPrefix: a.faucetIdPrefix,
                        asset: a.asset,
                    })));
                }
            }
        }
    }
    catch (error) {
        logWebStoreError(error, `Error undoing account states: ${accountCommitments.join(",")}`);
    }
}
