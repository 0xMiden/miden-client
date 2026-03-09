import { describe, it, expect } from "vitest";
import { openDatabase, getDatabase } from "./schema.js";
import { applyTransactionDelta, undoAccountStates } from "./accounts.js";
import { uniqueDbName } from "./test-utils.js";

describe("Account delta and undo operations", () => {
  // Use a consistent version so ensureClientVersion doesn't nuke the DB.
  const CLIENT_VERSION = "0.0.1";

  // The JS layer doesn't validate data formats — all validation happens in the
  // Rust layer before values reach IndexedDB. So we use short readable strings
  // here instead of real-length hex values.
  const ACCOUNT_ID = "0xacc1";
  const CODE_ROOT = "0xcode1";

  // Nonce "1" state
  const STORAGE_ROOT_N1 = "0xsroot1";
  const VAULT_ROOT_N1 = "0xvroot1";
  const COMMITMENT_N1 = "0xcommit1";
  const SLOT_VALUE_N1 = "0xbal100";
  const MAP_VALUE_N1 = "0xmval1";
  const ASSET_N1 = "0xasset1";

  // Nonce "2" state
  const STORAGE_ROOT_N2 = "0xsroot2";
  const VAULT_ROOT_N2 = "0xvroot2";
  const COMMITMENT_N2 = "0xcommit2";
  const SLOT_VALUE_N2 = "0xbal200";
  const MAP_VALUE_N2 = "0xmval2";
  const ASSET_N2 = "0xasset2";

  // Shared keys
  const SLOT_NAME = "balance";
  const MAP_SLOT_NAME = "metadata";
  const MAP_KEY = "0xmkey1";
  const VAULT_KEY = "0xvk1";
  const FAUCET_PREFIX = "0xfaucet1";

  it("undo restores previous account state", async () => {
    const dbId = uniqueDbName();
    await openDatabase(dbId, CLIENT_VERSION);
    const db = getDatabase(dbId);

    // Apply nonce "1" — initial account state
    await applyTransactionDelta(
      dbId,
      ACCOUNT_ID, // accountId
      "1", // nonce
      [{ slotName: SLOT_NAME, slotValue: SLOT_VALUE_N1, slotType: 0 }], // updatedSlots
      [{ slotName: MAP_SLOT_NAME, key: MAP_KEY, value: MAP_VALUE_N1 }], // changedMapEntries
      [{ vaultKey: VAULT_KEY, faucetIdPrefix: FAUCET_PREFIX, asset: ASSET_N1 }], // changedAssets
      CODE_ROOT, // codeRoot
      STORAGE_ROOT_N1, // storageRoot
      VAULT_ROOT_N1, // vaultRoot
      false, // committed
      COMMITMENT_N1 // commitment
    );

    // Apply nonce "2" — updated account state with changed values
    await applyTransactionDelta(
      dbId,
      ACCOUNT_ID, // accountId
      "2", // nonce
      [{ slotName: SLOT_NAME, slotValue: SLOT_VALUE_N2, slotType: 0 }], // updatedSlots
      [{ slotName: MAP_SLOT_NAME, key: MAP_KEY, value: MAP_VALUE_N2 }], // changedMapEntries
      [{ vaultKey: VAULT_KEY, faucetIdPrefix: FAUCET_PREFIX, asset: ASSET_N2 }], // changedAssets
      CODE_ROOT, // codeRoot
      STORAGE_ROOT_N2, // storageRoot
      VAULT_ROOT_N2, // vaultRoot
      false, // committed
      COMMITMENT_N2 // commitment
    );

    // Verify latest shows nonce "2" state
    const beforeUndo = await db.latestAccountHeaders
      .where("id")
      .equals(ACCOUNT_ID)
      .first();
    expect(beforeUndo?.nonce).toBe("2");
    expect(beforeUndo?.storageRoot).toBe(STORAGE_ROOT_N2);

    // Undo nonce "2" — should restore nonce "1" as the latest state
    await undoAccountStates(dbId, [COMMITMENT_N2]);

    // Validation: Check that latest state now shows the initial account state

    const afterUndo = await db.latestAccountHeaders
      .where("id")
      .equals(ACCOUNT_ID)
      .first();
    expect(afterUndo).toBeDefined();
    expect(afterUndo?.nonce).toBe("1");
    expect(afterUndo?.storageRoot).toBe(STORAGE_ROOT_N1);
    expect(afterUndo?.vaultRoot).toBe(VAULT_ROOT_N1);

    // Storage
    const latestStorage = await db.latestAccountStorages
      .where("accountId")
      .equals(ACCOUNT_ID)
      .toArray();
    expect(latestStorage).toHaveLength(1);
    expect(latestStorage[0].slotValue).toBe(SLOT_VALUE_N1);

    // Map entries
    const latestMaps = await db.latestStorageMapEntries
      .where("accountId")
      .equals(ACCOUNT_ID)
      .toArray();
    expect(latestMaps).toHaveLength(1);
    expect(latestMaps[0].value).toBe(MAP_VALUE_N1);

    // Assets
    const latestAssets = await db.latestAccountAssets
      .where("accountId")
      .equals(ACCOUNT_ID)
      .toArray();
    expect(latestAssets).toHaveLength(1);
    expect(latestAssets[0].asset).toBe(ASSET_N1);

    // Historical headers should only have nonce "1" left
    const historicalHeaders = await db.historicalAccountHeaders
      .where("id")
      .equals(ACCOUNT_ID)
      .toArray();
    expect(historicalHeaders).toHaveLength(1);
    expect(historicalHeaders[0].nonce).toBe("1");
  });
});
