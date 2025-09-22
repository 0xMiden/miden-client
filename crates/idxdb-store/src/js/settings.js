import { settings } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";
export async function getSettingValue(key) {
    try {
        // Fetch all records matching the given key
        const allMatchingRecords = await settings
            .where("key")
            .equals(key)
            .toArray();
        if (allMatchingRecords.length === 0) {
            console.log("No setting record found for given key.");
            return null;
        }
        // There should be only one match
        const matchingRecord = allMatchingRecords[0];
        // Convert the setting value to base64
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
export async function insertSettingValue(key, value) {
    try {
        const setting = {
            key,
            value,
        };
        await settings.put(setting);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting setting with key: ${key} and value(base64): ${uint8ArrayToBase64(value)}`);
    }
}
export async function deleteSettingValue(key) {
    try {
        await settings.where("key").equals(key).delete();
    }
    catch (error) {
        logWebStoreError(error, `Error deleting setting with key: ${key}`);
    }
}
//# sourceMappingURL=settings.js.map