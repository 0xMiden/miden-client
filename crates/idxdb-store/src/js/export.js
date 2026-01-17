// Disabling `any` checks since this file mostly deals with exporting DB types.
/* eslint-disable  @typescript-eslint/no-explicit-any */
/* eslint-disable  @typescript-eslint/no-unsafe-return */
/* eslint-disable  @typescript-eslint/no-unsafe-assignment */
import { getDatabase } from "./schema.js";
import { uint8ArrayToBase64 } from "./utils.js";
async function recursivelyTransformForExport(obj) {
    switch (obj.type) {
        case "Uint8Array":
            return Array.from(obj.value);
        case "Blob":
            return {
                __type: "Blob",
                data: uint8ArrayToBase64(new Uint8Array(await obj.value.arrayBuffer())),
            };
        case "Array":
            return await Promise.all(obj.value.map((v) => recursivelyTransformForExport({ type: getInputType(v), value: v })));
        case "Record":
            return Object.fromEntries(await Promise.all(Object.entries(obj.value).map(async ([key, value]) => [
                key,
                await recursivelyTransformForExport({
                    type: getInputType(value),
                    value,
                }),
            ])));
        case "Primitive":
            return obj.value;
    }
}
function getInputType(value) {
    if (value instanceof Uint8Array)
        return "Uint8Array";
    if (value instanceof Blob)
        return "Blob";
    if (Array.isArray(value))
        return "Array";
    if (value && typeof value === "object")
        return "Record";
    return "Primitive";
}
export async function transformForExport(obj) {
    return recursivelyTransformForExport({ type: getInputType(obj), value: obj });
}
export async function exportStore(dbId) {
    const db = getDatabase(dbId);
    const dbJson = {};
    for (const table of db.dexie.tables) {
        const records = await table.toArray();
        dbJson[table.name] = await Promise.all(records.map(transformForExport));
    }
    return JSON.stringify(dbJson);
}
