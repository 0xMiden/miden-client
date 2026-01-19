import { getDatabase, CLIENT_VERSION_SETTING_KEY } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

const INTERNAL_SETTING_KEYS = new Set([CLIENT_VERSION_SETTING_KEY]);

export async function getSetting(dbId: string, key: string) {
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
  } catch (error) {
    logWebStoreError(error, `Error while fetching setting key: ${key}`);
  }
}

export async function insertSetting(
  dbId: string,
  key: string,
  value: Uint8Array
): Promise<void> {
  try {
    const db = getDatabase(dbId);
    const setting = {
      key,
      value,
    };
    await db.settings.put(setting);
  } catch (error) {
    logWebStoreError(
      error,
      `Error inserting setting with key: ${key} and value(base64): ${uint8ArrayToBase64(value)}`
    );
  }
}

export async function removeSetting(dbId: string, key: string): Promise<void> {
  try {
    const db = getDatabase(dbId);
    await db.settings.where("key").equals(key).delete();
  } catch (error) {
    logWebStoreError(error, `Error deleting setting with key: ${key}`);
  }
}

export async function listSettingKeys(dbId: string) {
  try {
    const db = getDatabase(dbId);
    const keys: string[] = await db.settings
      .toArray()
      .then((settings) => settings.map((setting) => setting.key));
    return keys.filter((key) => !INTERNAL_SETTING_KEYS.has(key));
  } catch (error) {
    logWebStoreError(error, `Error listing setting keys`);
  }
}
