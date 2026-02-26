// Disabling `any` checks since this file mostly deals with exporting DB types.
/* eslint-disable  @typescript-eslint/no-explicit-any */
/* eslint-disable  @typescript-eslint/no-unsafe-return */
/* eslint-disable  @typescript-eslint/no-unsafe-assignment */
/* eslint-disable  @typescript-eslint/no-unsafe-member-access */
/* eslint-disable  @typescript-eslint/no-unsafe-argument */
import { getDatabase } from "./schema.js";
import { decryptSecretKey } from "./crypto.js";
import { uint8ArrayToBase64 } from "./utils.js";
type TransformableInput =
  | { type: "Uint8Array"; value: Uint8Array }
  | { type: "Blob"; value: Blob }
  | { type: "Array"; value: any[] }
  | { type: "Record"; value: Record<string, any> }
  | { type: "Primitive"; value: any };

async function recursivelyTransformForExport(
  obj: TransformableInput
): Promise<any> {
  switch (obj.type) {
    case "Uint8Array":
      return Array.from(obj.value);
    case "Blob":
      return {
        __type: "Blob" as const,
        data: uint8ArrayToBase64(new Uint8Array(await obj.value.arrayBuffer())),
      };
    case "Array":
      return await Promise.all(
        obj.value.map((v) =>
          recursivelyTransformForExport({ type: getInputType(v), value: v })
        )
      );
    case "Record":
      return Object.fromEntries(
        await Promise.all(
          Object.entries(obj.value).map(async ([key, value]) => [
            key,
            await recursivelyTransformForExport({
              type: getInputType(value),
              value,
            }),
          ])
        )
      );
    case "Primitive":
      return obj.value;
  }
}

function getInputType(value: any): TransformableInput["type"] {
  if (value instanceof Uint8Array) return "Uint8Array";
  if (value instanceof Blob) return "Blob";
  if (Array.isArray(value)) return "Array";
  if (value && typeof value === "object") return "Record";
  return "Primitive";
}

export async function transformForExport(obj: any): Promise<any> {
  return recursivelyTransformForExport({ type: getInputType(obj), value: obj });
}

export async function exportStore(dbId: string) {
  const db = getDatabase(dbId);
  const dbJson: Record<string, any> = {};

  for (const table of db.dexie.tables) {
    let records = await table.toArray();

    // Decrypt auth records to plaintext for export so they survive the
    // export/import cycle (the non-extractable CryptoKey cannot be serialized).
    if (table.name === "accountAuth") {
      records = await Promise.all(
        records.map(async (record: any) => {
          if (record.encryptedSecretKey && record.iv) {
            try {
              const secretKeyHex = await decryptSecretKey(
                dbId,
                record.encryptedSecretKey,
                record.iv
              );
              return {
                pubKeyCommitmentHex: record.pubKeyCommitmentHex,
                secretKeyHex,
              };
            } catch {
              console.warn(
                `Failed to decrypt auth for export: ${record.pubKeyCommitmentHex}. ` +
                  `Including raw encrypted record — it may not be usable after import.`
              );
              return record;
            }
          }
          return record;
        })
      );
    }

    dbJson[table.name] = await Promise.all(records.map(transformForExport));
  }

  return JSON.stringify(dbJson);
}
