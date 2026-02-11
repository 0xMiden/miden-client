import { getDatabase, CLIENT_VERSION_SETTING_KEY } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

const INTERNAL_SETTING_KEYS = new Set([CLIENT_VERSION_SETTING_KEY]);

export async function getSetting(dbId, key) {
  try {
    const db = getDatabase(dbId);
    const record = db
      .prepare("SELECT key, value FROM settings WHERE key = ?")
      .get(key);

    if (!record) {
      console.log("No setting record found for given key.");
      return null;
    }

    return {
      key: record.key,
      value: uint8ArrayToBase64(record.value),
    };
  } catch (error) {
    logWebStoreError(error, `Error while fetching setting key: ${key}`);
  }
}

export async function insertSetting(dbId, key, value) {
  try {
    const db = getDatabase(dbId);
    db.prepare(
      "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)"
    ).run(key, value);
  } catch (error) {
    logWebStoreError(error, `Error inserting setting with key: ${key}`);
  }
}

export async function removeSetting(dbId, key) {
  try {
    const db = getDatabase(dbId);
    db.prepare("DELETE FROM settings WHERE key = ?").run(key);
  } catch (error) {
    logWebStoreError(error, `Error deleting setting with key: ${key}`);
  }
}

export async function listSettingKeys(dbId) {
  try {
    const db = getDatabase(dbId);
    const rows = db.prepare("SELECT key FROM settings").all();
    return rows
      .map((r) => r.key)
      .filter((key) => !INTERNAL_SETTING_KEYS.has(key));
  } catch (error) {
    logWebStoreError(error, "Error listing setting keys");
  }
}
