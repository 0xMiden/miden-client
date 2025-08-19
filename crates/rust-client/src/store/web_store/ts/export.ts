import { db } from "../js/schema.js";
import { uint8ArrayToBase64 } from "./utils.js";
async function recursivelyTransformForExport(
  obj: Uint8Array
): Promise<number[]>;
async function recursivelyTransformForExport(
  obj: Blob
): Promise<{ __type: "Blob"; data: string }>;
async function recursivelyTransformForExport<T>(
  obj: T[]
): Promise<Awaited<ReturnType<typeof recursivelyTransformForExport>>[]>;
async function recursivelyTransformForExport<T extends Record<string, any>>(
  obj: T
): Promise<{
  [K in keyof T]: Awaited<ReturnType<typeof recursivelyTransformForExport>>;
}>;
async function recursivelyTransformForExport<T>(obj: T): Promise<T>;

// Implementation
async function recursivelyTransformForExport(obj: any): Promise<any> {
  if (obj instanceof Uint8Array) {
    return Array.from(obj);
  }
  if (obj instanceof Blob) {
    const blobBuffer = await obj.arrayBuffer();
    return {
      __type: "Blob" as const,
      data: uint8ArrayToBase64(new Uint8Array(blobBuffer)),
    };
  }
  if (Array.isArray(obj)) {
    return await Promise.all(obj.map(recursivelyTransformForExport));
  }
  if (obj && typeof obj === "object") {
    const entries = await Promise.all(
      Object.entries(obj).map(async ([key, value]) => [
        key,
        await recursivelyTransformForExport(value),
      ])
    );
    return Object.fromEntries(entries);
  }
  return obj;
}

export async function exportStore() {
  const dbJson: Record<string, any> = {};

  for (const table of db.tables) {
    const records = await table.toArray();
    dbJson[table.name] = await Promise.all(
      records.map(recursivelyTransformForExport)
    );
  }

  return JSON.stringify(dbJson);
}
