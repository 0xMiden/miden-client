import { describe, it, expect, afterEach } from "vitest";
import Dexie from "dexie";
import { MidenDatabase, CLIENT_VERSION_SETTING_KEY } from "./schema.js";

// The v1 store definitions, mirroring what MidenDatabase defines in schema.ts.
// IMPORTANT: If the v1 schema in schema.ts changes, update this copy too.
const V1_STORES = {
  accountCode: "root",
  accountStorage: "[commitment+slotName],commitment",
  storageMapEntries: "[root+key],root",
  accountAssets: "[root+vaultKey],root,faucetIdPrefix",
  accountAuth: "pubKeyCommitmentHex",
  accounts: "&accountCommitment,id,[id+nonce],codeRoot,storageRoot,vaultRoot",
  addresses: "address,id",
  transactions: "id,statusVariant",
  transactionScripts: "scriptRoot",
  inputNotes: "noteId,nullifier,stateDiscriminant",
  outputNotes: "noteId,recipientDigest,stateDiscriminant,nullifier",
  notesScripts: "scriptRoot",
  stateSync: "id",
  blockHeaders: "blockNum,hasClientNotes",
  partialBlockchainNodes: "id",
  tags: "id++,tag,source_note_id,source_account_id",
  foreignAccountCode: "accountId",
  settings: "key",
  trackedAccounts: "&id",
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();

// Unique DB names to avoid collisions between tests.
let dbCounter = 0;
function uniqueDbName(): string {
  return `test-miden-${++dbCounter}-${Date.now()}`;
}

// Track DBs for cleanup.
const openDbs: Dexie[] = [];

afterEach(async () => {
  for (const db of openDbs) {
    db.close();
    await db.delete();
  }
  openDbs.length = 0;
});

function trackDb(db: Dexie): Dexie {
  openDbs.push(db);
  return db;
}

describe("MidenDatabase migrations", () => {
  it("fresh DB creation seeds stateSync and stores client version", async () => {
    const name = uniqueDbName();
    const db = new MidenDatabase(name);
    openDbs.push(db.dexie);

    const result = await db.open("1.0.0");
    expect(result).toBe(true);

    // stateSync should be seeded by the populate hook
    const syncRecord = await db.stateSync.get(1);
    expect(syncRecord).toEqual({ id: 1, blockNum: 0 });

    // client version should be stored in settings
    const versionRecord = await db.settings.get(CLIENT_VERSION_SETTING_KEY);
    expect(versionRecord).toBeDefined();
    expect(decoder.decode(versionRecord!.value)).toBe("1.0.0");
  });

  it("v1 → v2 migration preserves data", async () => {
    const name = uniqueDbName();

    // Step 1: Create a v1 database and insert test data
    const dbV1 = trackDb(new Dexie(name));
    dbV1.version(1).stores(V1_STORES);
    await dbV1.open();

    await dbV1.table("settings").put({
      key: CLIENT_VERSION_SETTING_KEY,
      value: encoder.encode("0.9.0"),
    });
    await dbV1.table("stateSync").put({ id: 1, blockNum: 42 });
    await dbV1.table("accounts").put({
      id: "acct-1",
      codeRoot: "code-root-1",
      storageRoot: "storage-root-1",
      vaultRoot: "vault-root-1",
      nonce: "1",
      committed: true,
      accountCommitment: "commitment-1",
      locked: false,
    });
    await dbV1.table("inputNotes").put({
      noteId: "note-1",
      stateDiscriminant: 1,
      assets: new Uint8Array([1, 2, 3]),
      serialNumber: new Uint8Array([4, 5, 6]),
      inputs: new Uint8Array([7, 8, 9]),
      scriptRoot: "script-root-1",
      nullifier: "nullifier-1",
      serializedCreatedAt: "2025-01-01",
      state: new Uint8Array([10, 11]),
    });

    dbV1.close();

    // Step 2: Open with v1 + v2 (v2 adds a new index to accounts)
    const dbV2 = trackDb(new Dexie(name));
    dbV2.version(1).stores(V1_STORES);
    dbV2
      .version(2)
      .stores({
        accounts:
          "&accountCommitment,id,[id+nonce],codeRoot,storageRoot,vaultRoot,locked",
      })
      .upgrade((tx) => {
        // Example data transform: ensure all accounts have locked field
        return tx
          .table("accounts")
          .toCollection()
          .modify((record: Record<string, unknown>) => {
            if (record.locked === undefined) {
              record.locked = false;
            }
          });
      });
    await dbV2.open();

    // Verify data survived migration
    const syncRecord = await dbV2.table("stateSync").get(1);
    expect(syncRecord).toEqual({ id: 1, blockNum: 42 });

    const account = await dbV2.table("accounts").get("commitment-1");
    expect(account).toBeDefined();
    expect(account.id).toBe("acct-1");
    expect(account.locked).toBe(false);

    const note = await dbV2.table("inputNotes").get("note-1");
    expect(note).toBeDefined();
    expect(note.nullifier).toBe("nullifier-1");

    const versionRecord = await dbV2.table("settings").get(CLIENT_VERSION_SETTING_KEY);
    expect(decoder.decode(versionRecord.value)).toBe("0.9.0");
  });

  it("multi-version jump (v1 → v3) runs all intermediate migrations", async () => {
    const name = uniqueDbName();

    // Step 1: Create a v1 database with test data
    const dbV1 = trackDb(new Dexie(name));
    dbV1.version(1).stores(V1_STORES);
    await dbV1.open();

    await dbV1.table("stateSync").put({ id: 1, blockNum: 10 });
    await dbV1.table("settings").put({
      key: "testKey",
      value: encoder.encode("testValue"),
    });

    dbV1.close();

    // Step 2: Open with v1 + v2 + v3
    const dbV3 = trackDb(new Dexie(name));
    const migrationLog: number[] = [];

    dbV3.version(1).stores(V1_STORES);
    dbV3
      .version(2)
      .stores({
        // Add a new index to blockHeaders
        blockHeaders: "blockNum,hasClientNotes,header",
      })
      .upgrade(() => {
        migrationLog.push(2);
      });
    dbV3
      .version(3)
      .stores({
        // Add a new index to transactions
        transactions: "id,statusVariant,blockNum",
      })
      .upgrade(() => {
        migrationLog.push(3);
      });
    await dbV3.open();

    // Both upgrade callbacks should have fired in order
    expect(migrationLog).toEqual([2, 3]);

    // Original data should be preserved
    const syncRecord = await dbV3.table("stateSync").get(1);
    expect(syncRecord).toEqual({ id: 1, blockNum: 10 });

    const setting = await dbV3.table("settings").get("testKey");
    expect(decoder.decode(setting.value)).toBe("testValue");
  });

  it("migration failure triggers fallback reset", async () => {
    const name = uniqueDbName();

    // Step 1: Create a v1 database with data
    const dbV1 = trackDb(new Dexie(name));
    dbV1.version(1).stores(V1_STORES);
    await dbV1.open();
    await dbV1.table("stateSync").put({ id: 1, blockNum: 99 });
    dbV1.close();

    // Step 2: Open via MidenDatabase. Monkey-patch dexie.open to fail on
    // the first call (simulating a migration error), then succeed on retry.
    const midenDb = new MidenDatabase(name);
    openDbs.push(midenDb.dexie);

    let callCount = 0;
    const originalOpen = midenDb.dexie.open.bind(midenDb.dexie);
    midenDb.dexie.open = async () => {
      callCount++;
      if (callCount === 1) {
        throw new Error("Simulated migration failure");
      }
      return originalOpen();
    };

    const result = await midenDb.open("2.0.0");
    expect(result).toBe(true);

    // open() should have been called twice: first failed, second succeeded
    expect(callCount).toBe(2);

    // After reset, stateSync should be re-seeded (the old blockNum=99 is gone)
    const syncRecord = await midenDb.stateSync.get(1);
    expect(syncRecord).toEqual({ id: 1, blockNum: 0 });

    // Client version should be stored
    const versionRecord = await midenDb.settings.get(CLIENT_VERSION_SETTING_KEY);
    expect(decoder.decode(versionRecord!.value)).toBe("2.0.0");
  });

  it("reopening same version is a no-op and preserves data", async () => {
    const name = uniqueDbName();

    // First open
    const db1 = new MidenDatabase(name);
    openDbs.push(db1.dexie);
    await db1.open("1.0.0");

    // Insert some extra data beyond the seed
    await db1.settings.put({
      key: "userSetting",
      value: encoder.encode("myValue"),
    });

    const syncBefore = await db1.stateSync.get(1);
    expect(syncBefore).toEqual({ id: 1, blockNum: 0 });

    db1.dexie.close();

    // Second open — same version, same schema
    const db2 = new MidenDatabase(name);
    openDbs.push(db2.dexie);
    const result = await db2.open("1.0.0");
    expect(result).toBe(true);

    // All data should persist
    const syncAfter = await db2.stateSync.get(1);
    expect(syncAfter).toEqual({ id: 1, blockNum: 0 });

    const userSetting = await db2.settings.get("userSetting");
    expect(decoder.decode(userSetting!.value)).toBe("myValue");

    const versionRecord = await db2.settings.get(CLIENT_VERSION_SETTING_KEY);
    expect(decoder.decode(versionRecord!.value)).toBe("1.0.0");
  });
});
