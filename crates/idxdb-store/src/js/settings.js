import { getDatabase, CLIENT_VERSION_SETTING_KEY, } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";
export const PARTIAL_MMR_GENERATION_SETTING_KEY = "partialMmrGeneration";
const INTERNAL_SETTING_KEYS = new Set([
    CLIENT_VERSION_SETTING_KEY,
    PARTIAL_MMR_GENERATION_SETTING_KEY,
]);
const PARTIAL_MMR_GENERATION_BYTES = 8;
export async function getSetting(dbId, key) {
    try {
        const db = getDatabase(dbId);
        const allMatchingRecords = await db.settings
            .where("key")
            .equals(key)
            .toArray();
        if (allMatchingRecords.length === 0) {
            console.log("No setting record found for given key.");
            return null;
        }
        const matchingRecord = allMatchingRecords[0];
        const valueBase64 = uint8ArrayToBase64(matchingRecord.value);
        return {
            key: matchingRecord.key,
            value: valueBase64,
        };
    }
    catch (error) {
        logWebStoreError(error, `Error while fetching setting key: ${key}`);
    }
}
export async function insertSetting(dbId, key, value) {
    try {
        const db = getDatabase(dbId);
        const setting = {
            key,
            value,
        };
        await db.settings.put(setting);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting setting with key: ${key} and value(base64): ${uint8ArrayToBase64(value)}`);
    }
}
export async function removeSetting(dbId, key) {
    try {
        const db = getDatabase(dbId);
        await db.settings.where("key").equals(key).delete();
    }
    catch (error) {
        logWebStoreError(error, `Error deleting setting with key: ${key}`);
    }
}
export async function listSettingKeys(dbId) {
    try {
        const db = getDatabase(dbId);
        const keys = await db.settings
            .toArray()
            .then((settings) => settings.map((setting) => setting.key));
        return keys.filter((key) => !INTERNAL_SETTING_KEYS.has(key));
    }
    catch (error) {
        logWebStoreError(error, `Error listing setting keys`);
    }
}
export async function bumpPartialMmrGeneration(settings) {
    const current = await settings.get(PARTIAL_MMR_GENERATION_SETTING_KEY);
    const next = decodePartialMmrGeneration(current?.value) + 1n;
    await settings.put({
        key: PARTIAL_MMR_GENERATION_SETTING_KEY,
        value: encodePartialMmrGeneration(next),
    });
}
function decodePartialMmrGeneration(value) {
    if (!value) {
        return 0n;
    }
    if (value.length !== PARTIAL_MMR_GENERATION_BYTES) {
        throw new Error("partial MMR generation should be 8 bytes");
    }
    const view = new DataView(value.buffer, value.byteOffset, value.byteLength);
    return view.getBigUint64(0, true);
}
function encodePartialMmrGeneration(value) {
    const bytes = new Uint8Array(PARTIAL_MMR_GENERATION_BYTES);
    const view = new DataView(bytes.buffer);
    view.setBigUint64(0, value, true);
    return bytes;
}
