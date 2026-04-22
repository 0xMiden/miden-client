import { afterEach, describe, expect, it } from "vitest";

import { insertBlockHeader } from "./chainData.js";
import { getDatabase, openDatabase } from "./schema.js";
import { uniqueDbName } from "./test-utils.js";

// Track opened DB ids for per-test cleanup so suites don't leak state into
// each other (fake-indexeddb is process-global).
const openDbIds: string[] = [];

afterEach(async () => {
  for (const dbId of openDbIds) {
    const db = getDatabase(dbId);
    db.dexie.close();
    await db.dexie.delete();
  }
  openDbIds.length = 0;
});

async function openTestDb(): Promise<string> {
  const name = uniqueDbName();
  await openDatabase(name, "0.1.0");
  openDbIds.push(name);
  return name;
}

const BLOCK_NUM = 100;
const HEADER_V1 = new Uint8Array([1, 2, 3]);
const HEADER_V2 = new Uint8Array([9, 9, 9]);

describe("insertBlockHeader: add-if-not-exists semantics", () => {
  it("inserts a brand-new row when none exists (genesis path)", async () => {
    const dbId = await openTestDb();

    await insertBlockHeader(dbId, BLOCK_NUM, HEADER_V1, false);

    const stored = await getDatabase(dbId).blockHeaders.get(BLOCK_NUM);
    expect(stored).toBeDefined();
    expect(stored!.header).toEqual(HEADER_V1);
    expect(stored!.hasClientNotes).toBe("false");
  });

  it("does NOT overwrite the existing header when called a second time for the same block", async () => {
    const dbId = await openTestDb();

    // Step 1: first insert stores header V1.
    await insertBlockHeader(dbId, BLOCK_NUM, HEADER_V1, false);

    // Step 2: second insert with a different payload must NOT replace V1.
    await insertBlockHeader(dbId, BLOCK_NUM, HEADER_V2, true);

    const stored = await getDatabase(dbId).blockHeaders.get(BLOCK_NUM);
    expect(stored!.header).toEqual(HEADER_V1);
  });

  it("upgrades has_client_notes from false to true on a second call (matches SQLite set_block_header_has_client_notes)", async () => {
    // Scenario: block N was synced as irrelevant (hasClientNotes=false).
    // Later a private note with inclusion block N arrives via the transport
    // layer, `get_and_store_authenticated_block` fires with hasClientNotes=true.
    // SQLite does an explicit upgrade after its INSERT OR IGNORE; the IndexedDB
    // path must match or `get_tracked_block_header_numbers` misses this block.
    const dbId = await openTestDb();

    await insertBlockHeader(dbId, BLOCK_NUM, HEADER_V1, false);

    let stored = await getDatabase(dbId).blockHeaders.get(BLOCK_NUM);
    expect(stored!.hasClientNotes).toBe("false");

    // Second insert with hasClientNotes=true.
    await insertBlockHeader(
      dbId,
      BLOCK_NUM,
      HEADER_V2, // (ignored — header stays HEADER_V1)
      true
    );

    stored = await getDatabase(dbId).blockHeaders.get(BLOCK_NUM);
    // Header preserved...
    expect(stored!.header).toEqual(HEADER_V1);
    // ...but has_client_notes upgraded to true.
    expect(stored!.hasClientNotes).toBe("true");
  });

  it("does NOT downgrade has_client_notes from true to false on a second call with false", async () => {
    // Mirror SQLite's semantics: `set_block_header_has_client_notes` only sets
    // the flag to true; there is no downgrade path. Once a block is known to
    // contain a client note, subsequent writes should not flip that back.
    const dbId = await openTestDb();

    await insertBlockHeader(dbId, BLOCK_NUM, HEADER_V1, true);

    await insertBlockHeader(dbId, BLOCK_NUM, HEADER_V2, false);

    const stored = await getDatabase(dbId).blockHeaders.get(BLOCK_NUM);
    expect(stored!.hasClientNotes).toBe("true");
  });
});
