import { describe, it, expect, afterEach } from "vitest";
import Dexie from "dexie";

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
  // Placeholder for the actual v1→v2 migration test. When the first real
  // migration is introduced, replace the dummy schema and upgrade logic below
  // with the production V1_STORES → V2 change. The test structure (create v1
  // DB, insert data, reopen as v2, verify data survived) stays the same.
  //
  // This uses a raw Dexie instance with a toy schema because there's no real
  // migration yet — the purpose is to validate the vitest + fake-indexeddb
  // test setup and provide a working template for future migration tests.
  it("v1 → v2 migration preserves data", async () => {
    const name = uniqueDbName();

    const testV1 = {
      items: "id,category",
      settings: "key",
    };

    // Step 1: Create a v1 database and insert test data
    const dbV1 = trackDb(new Dexie(name));
    dbV1.version(1).stores(testV1);
    await dbV1.open();

    await dbV1
      .table("items")
      .put({ id: "item-1", category: "a", name: "Alice" });
    await dbV1.table("items").put({ id: "item-2", category: "b", name: "Bob" });
    await dbV1
      .table("settings")
      .put({ key: "color", value: encoder.encode("blue") });

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
});
