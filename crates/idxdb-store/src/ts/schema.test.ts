import { describe, it, expect, afterEach } from "vitest";
import Dexie from "dexie";
import { MidenDatabase, CLIENT_VERSION_SETTING_KEY } from "./schema.js";

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

// The v1→v2 migration test uses a raw Dexie instance to test the migration
// framework in isolation. In production, MidenDatabase.open() calls
// ensureClientVersion() which nukes the DB on major/minor upgrades — so
// migrations don't actually run yet. The test validates that the infrastructure
// works correctly for when the nuke is removed and migrations take over.
describe("MidenDatabase migrations", () => {
  it("v1 → v2 migration preserves data", async () => {
    const name = uniqueDbName();

    // Minimal schema independent from the production one.
    const testV1 = {
      items: "id,category",
      settings: "key",
    };

    // Step 1: Create a v1 database and insert test data
    const dbV1 = trackDb(new Dexie(name));
    dbV1.version(1).stores(testV1);
    await dbV1.open();

    await dbV1.table("items").put({ id: "item-1", category: "a", name: "Alice" });
    await dbV1.table("items").put({ id: "item-2", category: "b", name: "Bob" });
    await dbV1.table("settings").put({ key: "color", value: encoder.encode("blue") });

    dbV1.close();

    // Step 2: Open with v1 + v2 (v2 adds an index and a data transform)
    const dbV2 = trackDb(new Dexie(name));
    dbV2.version(1).stores(testV1);
    dbV2
      .version(2)
      .stores({ items: "id,category,name" })
      .upgrade((tx) => {
        return tx
          .table("items")
          .toCollection()
          .modify((record: Record<string, unknown>) => {
            if (!record.name) {
              record.name = "unknown";
            }
          });
      });
    await dbV2.open();

    // Verify data survived migration
    const item1 = await dbV2.table("items").get("item-1");
    expect(item1).toBeDefined();
    expect(item1.name).toBe("Alice");
    expect(item1.category).toBe("a");

    const item2 = await dbV2.table("items").get("item-2");
    expect(item2).toBeDefined();
    expect(item2.name).toBe("Bob");

    const setting = await dbV2.table("settings").get("color");
    expect(decoder.decode(setting.value)).toBe("blue");
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
    await db2.open("1.0.0");

    // All data should persist
    const syncAfter = await db2.stateSync.get(1);
    expect(syncAfter).toEqual({ id: 1, blockNum: 0 });

    const userSetting = await db2.settings.get("userSetting");
    expect(decoder.decode(userSetting!.value)).toBe("myValue");

    const versionRecord = await db2.settings.get(CLIENT_VERSION_SETTING_KEY);
    expect(decoder.decode(versionRecord!.value)).toBe("1.0.0");
  });
});
