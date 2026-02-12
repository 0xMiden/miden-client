/**
 * Settings operations for the WASM SQLite store.
 */

import { getDatabase, CLIENT_VERSION_SETTING_KEY } from "./schema.js";
import { logError, uint8ArrayToBase64 } from "./utils.js";

const INTERNAL_SETTING_KEYS = new Set([CLIENT_VERSION_SETTING_KEY]);

export function getSetting(
  dbId: string,
  key: string
): { key: string; value: string } | null {
  try {
    const db = getDatabase(dbId);
    const row = db.get<{ name: string; value: Uint8Array }>(
      "SELECT name, value FROM settings WHERE name = ?",
      [key]
    );

    if (!row) {
      return null;
    }

    const valueBytes =
      row.value instanceof Uint8Array
        ? row.value
        : new Uint8Array(row.value as ArrayBuffer);
    const valueBase64 = uint8ArrayToBase64(valueBytes);

    return {
      key: key,
      value: valueBase64,
    };
  } catch (error) {
    logError(error, `Error while fetching setting key: ${key}`);
    return null;
  }
}

export function insertSetting(
  dbId: string,
  key: string,
  value: Uint8Array
): void {
  try {
    const db = getDatabase(dbId);
    db.run("INSERT OR REPLACE INTO settings (name, value) VALUES (?, ?)", [
      key,
      value,
    ]);
  } catch (error) {
    logError(
      error,
      `Error inserting setting with key: ${key} and value(base64): ${uint8ArrayToBase64(value)}`
    );
  }
}

export function removeSetting(dbId: string, key: string): void {
  try {
    const db = getDatabase(dbId);
    db.run("DELETE FROM settings WHERE name = ?", [key]);
  } catch (error) {
    logError(error, `Error deleting setting with key: ${key}`);
  }
}

export function listSettingKeys(dbId: string): string[] {
  try {
    const db = getDatabase(dbId);
    const rows = db.all<{ name: string }>("SELECT name FROM settings");
    return rows
      .map((row) => row.name)
      .filter((key) => !INTERNAL_SETTING_KEYS.has(key));
  } catch (error) {
    logError(error, `Error listing setting keys`);
    return [];
  }
}
