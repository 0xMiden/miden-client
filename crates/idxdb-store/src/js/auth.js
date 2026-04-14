import { getDatabase } from "./schema.js";
import { logWebStoreError } from "./utils.js";
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
export async function removeAccountAuth(dbId, pubKeyCommitmentHex) {
    try {
        const db = getDatabase(dbId);
        await db.accountAuths
            .where("pubKeyCommitmentHex")
            .equals(pubKeyCommitmentHex)
            .delete();
    }
    catch (error) {
        logWebStoreError(error, `Error removing account auth for pubKey: ${pubKeyCommitmentHex}`);
    }
}
export async function insertAccountKeyMapping(dbId, accountIdHex, pubKeyCommitmentHex) {
    try {
        const db = getDatabase(dbId);
        const data = {
            accountIdHex,
            pubKeyCommitmentHex,
        };
        await db.accountKeyMappings.put(data);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting account key mapping for account ${accountIdHex} and key ${pubKeyCommitmentHex}`);
    }
}
export async function getKeyCommitmentsByAccountId(dbId, accountIdHex) {
    try {
        const db = getDatabase(dbId);
        const mappings = await db.accountKeyMappings
            .where("accountIdHex")
            .equals(accountIdHex)
            .toArray();
        return mappings.map((mapping) => mapping.pubKeyCommitmentHex);
    }
    catch (error) {
        logWebStoreError(error, `Error getting key commitments for account: ${accountIdHex}`);
        return [];
    }
}
export async function removeAllMappingsForKey(dbId, pubKeyCommitmentHex) {
    try {
        const db = getDatabase(dbId);
        await db.accountKeyMappings
            .where("pubKeyCommitmentHex")
            .equals(pubKeyCommitmentHex)
            .delete();
    }
    catch (error) {
        logWebStoreError(error, `Error removing all mappings for key: ${pubKeyCommitmentHex}`);
    }
}
export async function getAccountIdByKeyCommitment(dbId, pubKeyCommitmentHex) {
    try {
        const db = getDatabase(dbId);
        const mapping = await db.accountKeyMappings
            .where("pubKeyCommitmentHex")
            .equals(pubKeyCommitmentHex)
            .first();
        return mapping?.accountIdHex ?? null;
    }
    catch (error) {
        logWebStoreError(error, `Error fetching account by public key commitment: ${pubKeyCommitmentHex}`);
        return null;
    }
}
