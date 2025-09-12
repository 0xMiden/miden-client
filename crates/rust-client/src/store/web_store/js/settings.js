import { settings } from "./schema.js";
import { logWebStoreError } from "./utils.js";

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
        const Setting = {
            key: matchingRecord.key,
            value: matchingRecord.value,
        };
        return Setting;
    }
    catch (error) {
        logWebStoreError(error, `Error while fetching setting key: ${key}`);
    }
}
export async function insertSettingValue(key, value) {
    try {
        const SettingValue = {
            value,
        };
        await settings.put(SettingValue, key);
    }
    catch (error) {
        logWebStoreError(error, `Error inserting account: ${accountId}`);
    }
}
export async function deleteSettingValue(key) {
    try {
        await settings
            .where("key")
            .equals(key)
            .delete();
    }
    catch (error) {
        logWebStoreError(error, `Error deleting setting with key: ${key}`);
    }
}
//# sourceMappingURL=settings.js.map
