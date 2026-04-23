import { afterEach, describe, expect, it } from "vitest";

import { insertBlockHeader } from "./chainData.js";
import { forceImportStore } from "./import.js";
import { getDatabase, openDatabase } from "./schema.js";
import { PARTIAL_MMR_GENERATION_SETTING_KEY } from "./settings.js";
import { uniqueDbName } from "./test-utils.js";
import { uint8ArrayToBase64 } from "./utils.js";

const openDbIds: string[] = [];

afterEach(async () => {
  for (const dbId of openDbIds) {
    const db = getDatabase(dbId);
    db.dexie.close();
    await db.dexie.delete();
  }

  openDbIds.length = 0;
});

describe("Partial MMR generation", () => {
  it("bumps when block headers are updated", async () => {
    const dbId = await openDatabase(uniqueDbName(), "1.0.0");
    openDbIds.push(dbId);

    await insertBlockHeader(
      dbId,
      7,
      new Uint8Array([1, 2, 3]),
      new Uint8Array([4, 5, 6]),
      true
    );

    const generation = await getDatabase(dbId).settings.get(
      PARTIAL_MMR_GENERATION_SETTING_KEY
    );

    expect(Array.from(generation?.value ?? [])).toEqual([
      1, 0, 0, 0, 0, 0, 0, 0,
    ]);
  });

  it("overwrites imported generation values", async () => {
    const dbId = await openDatabase(uniqueDbName(), "1.0.0");
    openDbIds.push(dbId);

    const importedGeneration = generationBytes(5n);
    const storeDump = JSON.stringify({
      settings: [
        {
          key: PARTIAL_MMR_GENERATION_SETTING_KEY,
          value: {
            __type: "Uint8Array",
            data: uint8ArrayToBase64(importedGeneration),
          },
        },
      ],
    });

    await forceImportStore(dbId, storeDump);

    const generation = await getDatabase(dbId).settings.get(
      PARTIAL_MMR_GENERATION_SETTING_KEY
    );

    expect(Array.from(generation?.value ?? [])).toEqual([
      6, 0, 0, 0, 0, 0, 0, 0,
    ]);
  });
});

function generationBytes(value: bigint): Uint8Array {
  const bytes = new Uint8Array(8);
  const view = new DataView(bytes.buffer);
  view.setBigUint64(0, value, true);
  return bytes;
}
