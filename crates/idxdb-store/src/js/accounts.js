import { accountCodes, accountStorages, accountAssets, accountAuths, accounts, foreignAccountCode, storageMapEntries, } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";
// GET FUNCTIONS
export async function getAccountIds() {
    try {
        const allIds = new Set(); // Use a Set to ensure uniqueness
        // Iterate over each account entry
        await accounts.each((account) => {
            allIds.add(account.id); // Assuming 'account' has an 'id' property
        });
        return Array.from(allIds); // Convert back to array to return a list of unique IDs
    }
    catch (error) {
        logWebStoreError(error, "Error while fetching account IDs");
    }
}
export async function getAllAccountHeaders() {
    try {
        // Use a Map to track the latest record for each id based on nonce
        const latestRecordsMap = new Map();
        await accounts.each((record) => {
            const existingRecord = latestRecordsMap.get(record.id);
            if (!existingRecord ||
                BigInt(record.nonce) > BigInt(existingRecord.nonce)) {
                latestRecordsMap.set(record.id, record);
            }
        });
        // Extract the latest records from the Map
        const latestRecords = Array.from(latestRecordsMap.values());
        const resultObject = await Promise.all(latestRecords.map((record) => {
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
                vaultRoot: record.vaultRoot, // Fallback if missing
                storageRoot: record.storageRoot || "",
                codeRoot: record.codeRoot || "",
                accountSeed: accountSeedBase64, // null or base64 string
                locked: record.locked,
                committed: record.committed, // Use actual value or default
                accountCommitment: record.accountCommitment || "", // Keep original field name
            };
        }));
        return resultObject;
    }
    catch (error) {
        logWebStoreError(error, "Error while fetching account headers");
    }
}
export async function getAccountHeader(accountId) {
    try {
        // Fetch all records matching the given id
        const allMatchingRecords = await accounts
            .where("id")
            .equals(accountId)
            .toArray();
        if (allMatchingRecords.length === 0) {
            console.log("No account header record found for given ID.");
            return null;
        }
        // Convert nonce to BigInt and sort
        // Note: This assumes all nonces are valid BigInt strings.
        const sortedRecords = allMatchingRecords.sort((a, b) => {
            const bigIntA = BigInt(a.nonce);
            const bigIntB = BigInt(b.nonce);
            return bigIntA > bigIntB ? -1 : bigIntA < bigIntB ? 1 : 0;
        });
        // The first record is the most recent one due to the sorting
        const mostRecentRecord = sortedRecords[0];
        if (mostRecentRecord === undefined) {
            return null;
        }
        let accountSeedBase64 = undefined;
        if (mostRecentRecord.accountSeed) {
            // Ensure accountSeed is processed as a Uint8Array and converted to Base64
            if (mostRecentRecord.accountSeed.length > 0) {
                accountSeedBase64 = uint8ArrayToBase64(mostRecentRecord.accountSeed);
            }
        }
        const AccountHeader = {
            id: mostRecentRecord.id,
            nonce: mostRecentRecord.nonce,
            vaultRoot: mostRecentRecord.vaultRoot,
            storageRoot: mostRecentRecord.storageRoot,
            codeRoot: mostRecentRecord.codeRoot,
            accountSeed: accountSeedBase64,
            locked: mostRecentRecord.locked,
        };
        return AccountHeader;
    }
    catch (error) {
        logWebStoreError(error, `Error while fetching account header for id: ${accountId}`);
    }
}
export async function getAccountHeaderByCommitment(accountCommitment) {
    try {
        // Fetch all records matching the given commitment
        const allMatchingRecords = await accounts
            .where("accountCommitment")
            .equals(accountCommitment)
            .toArray();
        if (allMatchingRecords.length == 0) {
            return undefined;
        }
        // There should be only one match
        const matchingRecord = allMatchingRecords[0];
        if (matchingRecord === undefined) {
            console.log("No account header record found for given commitment.");
            return null;
        }
        let accountSeedBase64 = undefined;
        if (matchingRecord.accountSeed) {
            accountSeedBase64 = uint8ArrayToBase64(matchingRecord.accountSeed);
        }
        const AccountHeader = {
            id: matchingRecord.id,
            nonce: matchingRecord.nonce,
            vaultRoot: matchingRecord.vaultRoot,
            storageRoot: matchingRecord.storageRoot,
            codeRoot: matchingRecord.codeRoot,
            accountSeed: accountSeedBase64,
            locked: matchingRecord.locked,
        };
        return AccountHeader;
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account header for commitment ${accountCommitment}`);
    }
}
export async function getAccountCode(codeRoot) {
    try {
        // Fetch all records matching the given root
        const allMatchingRecords = await accountCodes
            .where("root")
            .equals(codeRoot)
            .toArray();
        // The first record is the only one due to the uniqueness constraint
        const codeRecord = allMatchingRecords[0];
        if (codeRecord === undefined) {
            console.log("No records found for given code root.");
            return null;
        }
        // Convert the code Blob to an ArrayBuffer
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
export async function getAccountStorage(storageCommitment) {
    try {
        const allMatchingRecords = await accountStorages
            .where("commitment")
            .equals(storageCommitment)
            .toArray();
        const slots = allMatchingRecords.map((record) => {
            return {
                slotIndex: record.slotIndex,
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
export async function getAccountStorageMaps(roots) {
    try {
        const allMatchingRecords = await storageMapEntries
            .where("root")
            .anyOf(roots)
            .toArray();
        return allMatchingRecords;
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account storage maps for roots ${roots.join(", ")}`);
    }
}
export async function getAccountVaultAssets(vaultRoot) {
    try {
        // Fetch all records matching the given root
        const allMatchingRecords = await accountAssets
            .where("root")
            .equals(vaultRoot)
            .toArray();
        // Map the records to their asset values
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
export async function getAccountAuthByPubKey(pubKey) {
    // Try to get the account auth from the store
    const accountSecretKey = await accountAuths
        .where("pubKey")
        .equals(pubKey)
        .first();
    // If it's not in the cache, throw an error
    if (!accountSecretKey) {
        throw new Error("Account auth not found in cache.");
    }
    const data = {
        secretKey: accountSecretKey.secretKey,
    };
    return data;
}
// INSERT FUNCTIONS
export async function upsertAccountCode(codeRoot, code) {
    try {
        // Prepare the data object to insert
        const data = {
            root: codeRoot, // Using codeRoot as the key
            code,
        };
        // Perform the insert using Dexie
        await accountCodes.put(data);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting code with root: ${codeRoot}`);
    }
}
export async function upsertAccountStorage(storageSlots) {
    try {
        let processedSlots = storageSlots.map((slot) => {
            return {
                commitment: slot.commitment,
                slotIndex: slot.slotIndex,
                slotValue: slot.slotValue,
                slotType: slot.slotType,
            };
        });
        await accountStorages.bulkPut(processedSlots);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting storage slots`);
    }
}
export async function upsertStorageMapEntries(entries) {
    try {
        let processedEntries = entries.map((entry) => {
            return {
                root: entry.root,
                key: entry.key,
                value: entry.value,
            };
        });
        await storageMapEntries.bulkPut(processedEntries);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting storage map entries`);
    }
}
export async function upsertVaultAssets(assets) {
    try {
        let processedAssets = assets.map((asset) => {
            return {
                root: asset.root,
                vaultKey: asset.vaultKey,
                faucetIdPrefix: asset.faucetIdPrefix,
                asset: asset.asset,
            };
        });
        await accountAssets.bulkPut(processedAssets);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting assets`);
    }
}
export async function upsertAccountRecord(accountId, codeRoot, storageRoot, vaultRoot, nonce, committed, commitment, accountSeed) {
    try {
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
        await accounts.put(data);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting account: ${accountId}`);
    }
}
export async function insertAccountAuth(pubKey, secretKey) {
    try {
        // Prepare the data object to insert
        const data = {
            pubKey: pubKey,
            secretKey: secretKey,
        };
        // Perform the insert using Dexie
        await accountAuths.add(data);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting account auth for pubKey: ${pubKey}`);
    }
}
export async function upsertForeignAccountCode(accountId, code, codeRoot) {
    try {
        await upsertAccountCode(codeRoot, code);
        const data = {
            accountId,
            codeRoot,
        };
        await foreignAccountCode.put(data);
    }
    catch (error) {
        logWebStoreError(error, `Error upserting foreign account code for account: ${accountId}`);
    }
}
export async function getForeignAccountCode(accountIds) {
    try {
        const foreignAccounts = await foreignAccountCode
            .where("accountId")
            .anyOf(accountIds)
            .toArray();
        if (foreignAccounts.length === 0) {
            console.log("No records found for the given account IDs.");
            return null; // No records found
        }
        const codeRoots = foreignAccounts.map((account) => account.codeRoot);
        const accountCode = await accountCodes
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
export async function lockAccount(accountId) {
    try {
        await accounts.where("id").equals(accountId).modify({ locked: true });
    }
    catch (error) {
        logWebStoreError(error, `Error locking account: ${accountId}`);
    }
}
// Delete functions
export async function undoAccountStates(accountCommitments) {
    try {
        await accounts
            .where("accountCommitment")
            .anyOf(accountCommitments)
            .delete();
    }
    catch (error) {
        logWebStoreError(error, `Error undoing account states: ${accountCommitments.join(",")}`);
    }
}
//# sourceMappingURL=accounts.js.map