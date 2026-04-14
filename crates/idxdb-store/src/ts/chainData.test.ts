import { afterEach, describe, expect, it } from "vitest";

import {
  getPartialBlockchainPeaksByBlockNum,
  insertBlockHeader,
} from "./chainData.js";
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
const PEAKS_FROM_SYNC = new Uint8Array([10, 11, 12]);
const PEAKS_FROM_BACKFILL = new Uint8Array([99, 98, 97]);

describe("insertBlockHeader: add-if-not-exists semantics", () => {
  it("inserts a brand-new row when none exists (genesis path)", async () => {
    const dbId = await openTestDb();

    await insertBlockHeader(dbId, BLOCK_NUM, HEADER_V1, PEAKS_FROM_SYNC, false);

    const stored = await getDatabase(dbId).blockHeaders.get(BLOCK_NUM);
    expect(stored).toBeDefined();
    expect(stored!.header).toEqual(HEADER_V1);
    expect(stored!.partialBlockchainPeaks).toEqual(PEAKS_FROM_SYNC);
    expect(stored!.hasClientNotes).toBe("false");
  });

  it("does NOT overwrite existing peaks when called a second time for the same block", async () => {
    // This is the core regression test for #2037. `applyStateSync` writes the
    // correct historical peaks first. Later `get_and_store_authenticated_block`
    // calls `insertBlockHeader` with a DIFFERENT peaks payload (peaks for the
    // caller's current PartialMmr forest). The old `put` behavior clobbered
    // the first write; we must keep it.
    const dbId = await openTestDb();

    // Step 1: sync writes correct peaks for block N.
    await insertBlockHeader(dbId, BLOCK_NUM, HEADER_V1, PEAKS_FROM_SYNC, false);

    // Step 2: authenticated-block backfill tries to overwrite with wrong peaks.
    await insertBlockHeader(
      dbId,
      BLOCK_NUM,
      HEADER_V2,
      PEAKS_FROM_BACKFILL,
      true
    );

    // Peaks and header from the first write must still be there.
    const stored = await getDatabase(dbId).blockHeaders.get(BLOCK_NUM);
    expect(stored!.header).toEqual(HEADER_V1);
    expect(stored!.partialBlockchainPeaks).toEqual(PEAKS_FROM_SYNC);

    // The peaks reader uses the same table lookup — double-check it serves the
    // preserved bytes (it base64-encodes them so we decode).
    const peaks = await getPartialBlockchainPeaksByBlockNum(dbId, BLOCK_NUM);
    expect(peaks).toBeDefined();
    const decoded = Uint8Array.from(atob(peaks!.peaks!), (c) =>
      c.charCodeAt(0)
    );
    expect(decoded).toEqual(PEAKS_FROM_SYNC);
  });

  it("upgrades has_client_notes from false to true on a second call (matches SQLite set_block_header_has_client_notes)", async () => {
    // Scenario: block N was synced as irrelevant (hasClientNotes=false).
    // Later a private note with inclusion block N arrives via the transport
    // layer, `get_and_store_authenticated_block` fires with hasClientNotes=true.
    // SQLite does an explicit upgrade after its INSERT OR IGNORE; the IndexedDB
    // path must match or `get_tracked_block_header_numbers` misses this block.
    const dbId = await openTestDb();

    await insertBlockHeader(dbId, BLOCK_NUM, HEADER_V1, PEAKS_FROM_SYNC, false);

    let stored = await getDatabase(dbId).blockHeaders.get(BLOCK_NUM);
    expect(stored!.hasClientNotes).toBe("false");

    // Second insert with hasClientNotes=true.
    await insertBlockHeader(
      dbId,
      BLOCK_NUM,
      HEADER_V2, // (ignored — header stays HEADER_V1)
      PEAKS_FROM_BACKFILL, // (ignored — peaks stay PEAKS_FROM_SYNC)
      true
    );

    stored = await getDatabase(dbId).blockHeaders.get(BLOCK_NUM);
    // Header + peaks preserved...
    expect(stored!.header).toEqual(HEADER_V1);
    expect(stored!.partialBlockchainPeaks).toEqual(PEAKS_FROM_SYNC);
    // ...but has_client_notes upgraded to true.
    expect(stored!.hasClientNotes).toBe("true");
  });

  it("does NOT downgrade has_client_notes from true to false on a second call with false", async () => {
    // Mirror SQLite's semantics: `set_block_header_has_client_notes` only sets
    // the flag to true; there is no downgrade path. Once a block is known to
    // contain a client note, subsequent writes should not flip that back.
    const dbId = await openTestDb();

    await insertBlockHeader(dbId, BLOCK_NUM, HEADER_V1, PEAKS_FROM_SYNC, true);

    await insertBlockHeader(
      dbId,
      BLOCK_NUM,
      HEADER_V2,
      PEAKS_FROM_BACKFILL,
      false
    );

    const stored = await getDatabase(dbId).blockHeaders.get(BLOCK_NUM);
    expect(stored!.hasClientNotes).toBe("true");
  });
});
