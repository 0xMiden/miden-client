// Disabling `any` checks since this file mostly deals
// with importing DB types and we're testing this which
// should be enough + the TS compiler.
/* eslint-disable */
import { getDatabase } from "./schema.js";
import { logWebStoreError } from "./utils.js";
type ImportableInput =
  | { type: "Blob"; value: { __type: "Blob"; data: string } }
  | { type: "Array"; value: any[] }
  | { type: "Object"; value: Record<string, any> }
  | { type: "Primitive"; value: any };

async function recursivelyTransformForImport(
  obj: ImportableInput
): Promise<any> {
  switch (obj.type) {
    case "Blob":
      return new Blob([base64ToUint8Array(obj.value.data)]);
    case "Array":
      return await Promise.all(
        obj.value.map((v) =>
          recursivelyTransformForImport({ type: getImportType(v), value: v })
        )
      );
    case "Object":
      return Object.fromEntries(
        await Promise.all(
          Object.entries(obj.value).map(async ([key, value]) => [
            key,
            await recursivelyTransformForImport({
              type: getImportType(value),
              value,
            }),
          ])
        )
      );
    case "Primitive":
      return obj.value;
  }
}

function getImportType(value: any): ImportableInput["type"] {
  if (value && typeof value === "object" && value.__type === "Blob") {
    return "Blob";
  }
  if (Array.isArray(value)) return "Array";
  if (value && typeof value === "object") return "Object";
  return "Primitive";
}

export async function transformForImport(obj: any): Promise<any> {
  return recursivelyTransformForImport({
    type: getImportType(obj),
    value: obj,
  });
}

export async function forceImportStore(dbId: string, jsonStr: string) {
  try {
    const db = getDatabase(dbId);
    let dbJson = JSON.parse(jsonStr);
    if (typeof dbJson === "string") {
      dbJson = JSON.parse(dbJson);
    }

    const jsonTableNames = Object.keys(dbJson);
    const dbTableNames = db.dexie.tables.map((t) => t.name);

    if (jsonTableNames.length === 0) {
      throw new Error("No tables found in the provided JSON.");
    }

    await db.dexie.transaction("rw", dbTableNames, async () => {
      await Promise.all(db.dexie.tables.map((t) => t.clear()));

      for (const tableName of jsonTableNames) {
        const table = db.dexie.table(tableName);

        if (!dbTableNames.includes(tableName)) {
          console.warn(
            `Table "${tableName}" does not exist in the database schema. Skipping.`
          );
          continue;
        }

        const records = dbJson[tableName];

        const transformedRecords = await Promise.all(
          records.map(transformForImport)
        );

        await table.bulkPut(transformedRecords);
      }
    });

    console.log("Store imported successfully.");
  } catch (err) {
    logWebStoreError(err);
  }
}

function base64ToUint8Array(base64: string) {
  const binaryString = atob(base64);
  const len = binaryString.length;
  const bytes = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}
