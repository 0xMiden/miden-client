import { getDatabase } from "./schema.js";
import { logWebStoreError } from "./utils.js";

function transformValueForImport(value) {
  if (value && typeof value === "object" && value.__type === "Blob") {
    return Buffer.from(base64ToUint8Array(value.data));
  }
  if (Array.isArray(value)) {
    // Check if this is a number array that should become a Buffer
    if (value.length > 0 && typeof value[0] === "number") {
      return Buffer.from(value);
    }
    return value.map(transformValueForImport);
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([k, v]) => [k, transformValueForImport(v)])
    );
  }
  return value;
}

export async function forceImportStore(dbId, jsonStr) {
  try {
    const db = getDatabase(dbId);
    let dbJson = JSON.parse(jsonStr);
    if (typeof dbJson === "string") {
      dbJson = JSON.parse(dbJson);
    }

    const jsonTableNames = Object.keys(dbJson);
    if (jsonTableNames.length === 0) {
      throw new Error("No tables found in the provided JSON.");
    }

    // Get list of tables in DB
    const dbTableNames = db
      .prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
      )
      .all()
      .map((r) => r.name);

    const importAll = db.transaction(() => {
      // Clear all tables
      for (const name of dbTableNames) {
        db.exec(`DELETE FROM "${name}"`);
      }

      for (const tableName of jsonTableNames) {
        if (!dbTableNames.includes(tableName)) {
          console.warn(
            `Table "${tableName}" does not exist in the database schema. Skipping.`
          );
          continue;
        }

        const records = dbJson[tableName];
        for (const record of records) {
          const transformed = transformValueForImport(record);
          const keys = Object.keys(transformed);
          const values = keys.map((k) => transformed[k]);
          const placeholders = keys.map(() => "?").join(",");
          const columns = keys.map((k) => `"${k}"`).join(",");
          db.prepare(
            `INSERT INTO "${tableName}" (${columns}) VALUES (${placeholders})`
          ).run(...values);
        }
      }
    });
    importAll();
    console.log("Store imported successfully.");
  } catch (err) {
    logWebStoreError(err);
  }
}

function base64ToUint8Array(base64) {
  const binaryString = atob(base64);
  const len = binaryString.length;
  const bytes = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}
