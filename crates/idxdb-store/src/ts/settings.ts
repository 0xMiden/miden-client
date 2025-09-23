import { settings } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

export async function getValue(key: string) {
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
  } catch (error) {
    logWebStoreError(error, `Error while fetching setting key: ${key}`);
  }
}

export async function insertValue(
  key: string,
  value: Uint8Array
): Promise<void> {
  try {
    const setting = {
      key,
      value,
    };
    await settings.put(setting);
  } catch (error) {
    logWebStoreError(
      error,
      `Error inserting setting with key: ${key} and value(base64): ${uint8ArrayToBase64(value)}`
    );
  }
}

export async function removeValue(key: string): Promise<void> {
  try {
    await settings.where("key").equals(key).delete();
  } catch (error) {
    logWebStoreError(error, `Error deleting setting with key: ${key}`);
  }
}

export async function listKeys() {
  try {
    const keys: string[] = await settings
      .toArray()
      .then((settings) => settings.map((setting) => setting.key));
    return keys;
  } catch (error) {
    logWebStoreError(error, `Error listing setting keys`);
  }
}
