import { describe, it, expect, afterEach } from "vitest";
import { openDatabase, getDatabase } from "./schema.js";
import { upsertInputNote, getInputNoteByOffset } from "./notes.js";

// Unique DB names to avoid collisions between tests.
let dbCounter = 0;
function uniqueDbName(): string {
  return `test-notes-${++dbCounter}-${Date.now()}`;
}

// Track DB IDs for cleanup.
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

// Consumed state discriminants (must match InputNoteState constants on the Rust side).
const STATE_CONSUMED_AUTHENTICATED_LOCAL = 6;
const STATE_CONSUMED_UNAUTHENTICATED_LOCAL = 7;
const STATE_CONSUMED_EXTERNAL = 8;
const STATE_EXPECTED = 0;

const CONSUMED_STATES = new Uint8Array([
  STATE_CONSUMED_AUTHENTICATED_LOCAL,
  STATE_CONSUMED_UNAUTHENTICATED_LOCAL,
  STATE_CONSUMED_EXTERNAL,
]);

const DUMMY_BYTES = new Uint8Array([1, 2, 3]);
const DUMMY_SCRIPT_ROOT = "script-root-1";

/**
 * Insert a minimal input note with consumption metadata.
 * The noteId is stored in the `createdAt` field so we can recover it from the
 * processed (base64-encoded) output of `getInputNoteByOffset`.
 */
async function insertNote(
  dbId: string,
  noteId: string,
  opts: {
    stateDiscriminant?: number;
    consumedBlockHeight?: number;
    consumedTxOrder?: number;
    consumerAccountId?: string;
  } = {}
) {
  await upsertInputNote(
    dbId,
    noteId,
    DUMMY_BYTES,
    DUMMY_BYTES,
    DUMMY_BYTES,
    DUMMY_SCRIPT_ROOT,
    DUMMY_BYTES,
    `nullifier-${noteId}`,
    noteId, // store noteId as createdAt so we can read it back from processed output
    opts.stateDiscriminant ?? STATE_CONSUMED_EXTERNAL,
    DUMMY_BYTES,
    opts.consumedBlockHeight,
    opts.consumedTxOrder,
    opts.consumerAccountId
  );
}

/**
 * Iterate through all notes using `getInputNoteByOffset`, collecting noteIds
 * from the actual function output. The noteId is recovered from the `createdAt`
 * field of the processed result (where we stored it during insertion).
 */
async function collectAllNoteIds(
  dbId: string,
  states: Uint8Array,
  consumer?: string,
  blockStart?: number,
  blockEnd?: number
): Promise<string[]> {
  const ids: string[] = [];
  let offset = 0;

  // eslint-disable-next-line no-constant-condition
  while (true) {
    const result = await getInputNoteByOffset(
      dbId,
      states,
      consumer,
      blockStart,
      blockEnd,
      offset
    );
    if (!result || result.length === 0) break;
    // createdAt holds the noteId (see insertNote)
    ids.push(result[0].createdAt);
    offset++;
  }

  return ids;
}

// ORDERING TESTS
// ================================================================================================

describe("getInputNoteByOffset ordering", () => {
  it("returns notes ordered by block height", async () => {
    const dbId = await openTestDb();

    await insertNote(dbId, "note-b3", {
      consumedBlockHeight: 3,
      consumedTxOrder: 0,
    });
    await insertNote(dbId, "note-b1", {
      consumedBlockHeight: 1,
      consumedTxOrder: 0,
    });
    await insertNote(dbId, "note-b2", {
      consumedBlockHeight: 2,
      consumedTxOrder: 0,
    });

    const ids = await collectAllNoteIds(dbId, CONSUMED_STATES);
    expect(ids).toEqual(["note-b1", "note-b2", "note-b3"]);
  });

  it("returns notes ordered by tx order within same block", async () => {
    const dbId = await openTestDb();

    await insertNote(dbId, "note-tx2", {
      consumedBlockHeight: 5,
      consumedTxOrder: 2,
    });
    await insertNote(dbId, "note-tx0", {
      consumedBlockHeight: 5,
      consumedTxOrder: 0,
    });
    await insertNote(dbId, "note-tx1", {
      consumedBlockHeight: 5,
      consumedTxOrder: 1,
    });

    const ids = await collectAllNoteIds(dbId, CONSUMED_STATES);
    expect(ids).toEqual(["note-tx0", "note-tx1", "note-tx2"]);
  });

  it("sorts null tx order last within same block (fallback path)", async () => {
    const dbId = await openTestDb();

    await insertNote(dbId, "note-ordered", {
      consumedBlockHeight: 5,
      consumedTxOrder: 0,
    });
    await insertNote(dbId, "note-unordered", {
      consumedBlockHeight: 5,
      // no consumedTxOrder
    });

    // No consumer -> fallback path that includes null tx_order notes
    const ids = await collectAllNoteIds(dbId, CONSUMED_STATES);
    expect(ids).toEqual(["note-ordered", "note-unordered"]);
  });

  it("uses noteId as tiebreaker for same block and tx order", async () => {
    const dbId = await openTestDb();

    await insertNote(dbId, "note-c", {
      consumedBlockHeight: 1,
      consumedTxOrder: 0,
    });
    await insertNote(dbId, "note-a", {
      consumedBlockHeight: 1,
      consumedTxOrder: 0,
    });
    await insertNote(dbId, "note-b", {
      consumedBlockHeight: 1,
      consumedTxOrder: 0,
    });

    const ids = await collectAllNoteIds(dbId, CONSUMED_STATES);
    expect(ids).toEqual(["note-a", "note-b", "note-c"]);
  });
});

// CONSUMER FILTER TESTS
// ================================================================================================

describe("getInputNoteByOffset consumer filtering", () => {
  it("filters by consumer account", async () => {
    const dbId = await openTestDb();

    await insertNote(dbId, "note-alice-1", {
      consumedBlockHeight: 1,
      consumedTxOrder: 0,
      consumerAccountId: "0xalice",
    });
    await insertNote(dbId, "note-bob", {
      consumedBlockHeight: 1,
      consumedTxOrder: 1,
      consumerAccountId: "0xbob",
    });
    await insertNote(dbId, "note-alice-2", {
      consumedBlockHeight: 2,
      consumedTxOrder: 0,
      consumerAccountId: "0xalice",
    });

    const ids = await collectAllNoteIds(dbId, CONSUMED_STATES, "0xalice");
    expect(ids).toEqual(["note-alice-1", "note-alice-2"]);
  });

  it("excludes notes without tx order when consumer is set", async () => {
    const dbId = await openTestDb();

    await insertNote(dbId, "note-with-order", {
      consumedBlockHeight: 1,
      consumedTxOrder: 0,
      consumerAccountId: "0xalice",
    });
    await insertNote(dbId, "note-without-order", {
      consumedBlockHeight: 1,
      // no consumedTxOrder — won't appear in compound index
      consumerAccountId: "0xalice",
    });

    const ids = await collectAllNoteIds(dbId, CONSUMED_STATES, "0xalice");
    // Only the note with tx_order should be returned (cursor path uses compound index).
    expect(ids).toEqual(["note-with-order"]);
  });
});

// BLOCK RANGE FILTER TESTS
// ================================================================================================

describe("getInputNoteByOffset block range filtering", () => {
  it("filters by block range", async () => {
    const dbId = await openTestDb();

    await insertNote(dbId, "note-b1", {
      consumedBlockHeight: 1,
      consumedTxOrder: 0,
    });
    await insertNote(dbId, "note-b3", {
      consumedBlockHeight: 3,
      consumedTxOrder: 0,
    });
    await insertNote(dbId, "note-b5", {
      consumedBlockHeight: 5,
      consumedTxOrder: 0,
    });
    await insertNote(dbId, "note-b7", {
      consumedBlockHeight: 7,
      consumedTxOrder: 0,
    });

    // Block range 3..=5
    const ids = await collectAllNoteIds(
      dbId,
      CONSUMED_STATES,
      undefined,
      3,
      5
    );
    expect(ids).toEqual(["note-b3", "note-b5"]);
  });

  it("filters by consumer and block range combined", async () => {
    const dbId = await openTestDb();

    await insertNote(dbId, "alice-b1", {
      consumedBlockHeight: 1,
      consumedTxOrder: 0,
      consumerAccountId: "0xalice",
    });
    await insertNote(dbId, "alice-b3", {
      consumedBlockHeight: 3,
      consumedTxOrder: 0,
      consumerAccountId: "0xalice",
    });
    await insertNote(dbId, "bob-b3", {
      consumedBlockHeight: 3,
      consumedTxOrder: 1,
      consumerAccountId: "0xbob",
    });
    await insertNote(dbId, "alice-b5", {
      consumedBlockHeight: 5,
      consumedTxOrder: 0,
      consumerAccountId: "0xalice",
    });

    const ids = await collectAllNoteIds(
      dbId,
      CONSUMED_STATES,
      "0xalice",
      3,
      5
    );
    expect(ids).toEqual(["alice-b3", "alice-b5"]);
  });
});

// STATE FILTER TESTS
// ================================================================================================

describe("getInputNoteByOffset state filtering", () => {
  it("skips non-consumed notes", async () => {
    const dbId = await openTestDb();

    await insertNote(dbId, "consumed", {
      stateDiscriminant: STATE_CONSUMED_EXTERNAL,
      consumedBlockHeight: 1,
      consumedTxOrder: 0,
    });
    await insertNote(dbId, "expected", {
      stateDiscriminant: STATE_EXPECTED,
    });

    const ids = await collectAllNoteIds(dbId, CONSUMED_STATES);
    expect(ids).toEqual(["consumed"]);
  });

  it("returns empty when no notes match", async () => {
    const dbId = await openTestDb();

    const result = await getInputNoteByOffset(
      dbId,
      CONSUMED_STATES,
      undefined,
      undefined,
      undefined,
      0
    );
    expect(result).toEqual([]);
  });
});
