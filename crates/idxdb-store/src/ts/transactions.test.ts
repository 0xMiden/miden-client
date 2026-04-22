import { describe, it, expect, afterEach } from "vitest";
import { openDatabase, getDatabase } from "./schema.js";
import { applyTransactionBatch } from "./transactions.js";

// Unique DB names to avoid collisions between tests.
let dbCounter = 0;
function uniqueDbName(): string {
  return `test-tx-${++dbCounter}-${Date.now()}`;
}

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

// Build a minimal valid payload for applyTransactionBatch. Short readable
// strings are fine here — the JS layer does not validate data formats (all
// validation happens in the Rust layer before values reach IndexedDB).
// See the same pattern in accounts.test.ts.
function buildPayload(suffix: string) {
  const dummy = new Uint8Array([1, 2, 3]);
  return {
    transactionRecord: {
      id: `tx-${suffix}`,
      details: dummy,
      blockNum: 1,
      statusVariant: 0,
      status: dummy,
    },
    accountState: {
      kind: "delta" as const,
      accountId: `0xacc-${suffix}`,
      nonce: "1",
      updatedSlots: [],
      changedMapEntries: [],
      changedAssets: [],
      codeRoot: "0xcode",
      storageRoot: `0xsroot-${suffix}`,
      vaultRoot: `0xvroot-${suffix}`,
      committed: false,
      commitment: `0xcommit-${suffix}`,
    },
    inputNotes: [
      {
        noteId: `note-${suffix}`,
        noteAssets: dummy,
        serialNumber: dummy,
        inputs: dummy,
        noteScriptRoot: `script-root-${suffix}`,
        noteScript: dummy,
        nullifier: `nullifier-${suffix}`,
        createdAt: `0x${suffix}`,
        stateDiscriminant: 0,
        state: dummy,
      },
    ],
    outputNotes: [],
    tags: [],
  };
}

describe("applyTransactionBatch atomicity", () => {
  it("commits all writes from a valid 2-payload batch (positive control)", async () => {
    const dbId = await openTestDb();
    const db = getDatabase(dbId);

    await applyTransactionBatch(dbId, [buildPayload("a"), buildPayload("b")]);

    expect(await db.transactions.count()).toBe(2);
    expect(await db.inputNotes.count()).toBe(2);
    expect(await db.latestAccountHeaders.count()).toBe(2);
  });

  it("rolls back all writes when a mid-batch write fails", async () => {
    const dbId = await openTestDb();
    const db = getDatabase(dbId);

    // Install a Dexie `creating` hook on inputNotes that throws on the second
    // insert. This simulates a realistic mid-batch Dexie write failure (e.g.
    // quota exceeded, constraint violation) — payload 1 has already written
    // its row before payload 2's inputNote insert fails. If the outer Dexie
    // transaction is truly atomic, payload 1's writes must also roll back.
    let noteCreations = 0;
    db.inputNotes.hook("creating", () => {
      noteCreations += 1;
      if (noteCreations === 2) {
        throw new Error("simulated Dexie write failure");
      }
    });

    await expect(
      applyTransactionBatch(dbId, [buildPayload("a"), buildPayload("b")])
    ).rejects.toThrow();

    expect(await db.transactions.count()).toBe(0);
    expect(await db.inputNotes.count()).toBe(0);
    expect(await db.latestAccountHeaders.count()).toBe(0);
    expect(await db.notesScripts.count()).toBe(0);
  });
});
