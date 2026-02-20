import {
  getDatabase,
  IAccount,
  IHistoricalAccountAsset,
  IHistoricalStorageMapEntry,
  JsStorageMapEntry,
  JsStorageSlot,
  JsVaultAsset,
  MidenDatabase,
} from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

function seedToBase64(seed: Uint8Array | undefined): string | undefined {
  return seed ? uint8ArrayToBase64(seed) : undefined;
}

export async function getAccountIds(dbId: string) {
  try {
    const db = getDatabase(dbId);
    const records = await db.latestAccountHeaders.toArray();
    return records.map((entry) => entry.id);
  } catch (error) {
    logWebStoreError(error, "Error while fetching account IDs");
  }

  return [];
}

export async function getAllAccountHeaders(dbId: string) {
  try {
    const db = getDatabase(dbId);
    const records = await db.latestAccountHeaders.toArray();

    const resultObject = records.map((record) => ({
      id: record.id,
      nonce: record.nonce,
      vaultRoot: record.vaultRoot,
      storageRoot: record.storageRoot || "",
      codeRoot: record.codeRoot || "",
      accountSeed: seedToBase64(record.accountSeed),
      locked: record.locked,
      committed: record.committed,
      accountCommitment: record.accountCommitment || "",
    }));

    return resultObject;
  } catch (error) {
    logWebStoreError(error, "Error while fetching account headers");
  }
}

export async function getAccountHeader(dbId: string, accountId: string) {
  try {
    const db = getDatabase(dbId);
    const record = await db.latestAccountHeaders
      .where("id")
      .equals(accountId)
      .first();

    if (!record) {
      console.log("No account header record found for given ID.");
      return null;
    }

    return {
      id: record.id,
      nonce: record.nonce,
      vaultRoot: record.vaultRoot,
      storageRoot: record.storageRoot,
      codeRoot: record.codeRoot,
      accountSeed: seedToBase64(record.accountSeed),
      locked: record.locked,
    };
  } catch (error) {
    logWebStoreError(
      error,
      `Error while fetching account header for id: ${accountId}`
    );
  }
}

export async function getAccountHeaderByCommitment(
  dbId: string,
  accountCommitment: string
) {
  try {
    const db = getDatabase(dbId);
    const record = await db.historicalAccountHeaders
      .where("accountCommitment")
      .equals(accountCommitment)
      .first();

    if (!record) {
      return undefined;
    }

    return {
      id: record.id,
      nonce: record.nonce,
      vaultRoot: record.vaultRoot,
      storageRoot: record.storageRoot,
      codeRoot: record.codeRoot,
      accountSeed: seedToBase64(record.accountSeed),
      locked: record.locked,
    };
  } catch (error) {
    logWebStoreError(
      error,
      `Error fetching account header for commitment ${accountCommitment}`
    );
  }
}

export async function getAccountCode(dbId: string, codeRoot: string) {
  try {
    const db = getDatabase(dbId);
    const allMatchingRecords = await db.accountCodes
      .where("root")
      .equals(codeRoot)
      .toArray();

    const codeRecord = allMatchingRecords[0];

    if (codeRecord === undefined) {
      console.log("No records found for given code root.");
      return null;
    }

    const codeBase64 = uint8ArrayToBase64(codeRecord.code);
    return {
      root: codeRecord.root,
      code: codeBase64,
    };
  } catch (error) {
    logWebStoreError(error, `Error fetching account code for root ${codeRoot}`);
  }
}

export async function getAccountStorage(dbId: string, accountId: string) {
  try {
    const db = getDatabase(dbId);
    const allMatchingRecords = await db.latestAccountStorages
      .where("accountId")
      .equals(accountId)
      .toArray();

    const slots = allMatchingRecords.map((record) => {
      return {
        slotName: record.slotName,
        slotValue: record.slotValue,
        slotType: record.slotType,
      };
    });
    return slots;
  } catch (error) {
    logWebStoreError(
      error,
      `Error fetching account storage for account ${accountId}`
    );
  }
}

export async function getAccountStorageMaps(dbId: string, accountId: string) {
  try {
    const db = getDatabase(dbId);
    const allMatchingRecords = await db.latestStorageMapEntries
      .where("accountId")
      .equals(accountId)
      .toArray();

    return allMatchingRecords;
  } catch (error) {
    logWebStoreError(
      error,
      `Error fetching account storage maps for account ${accountId}`
    );
  }
}

export async function getAccountVaultAssets(dbId: string, accountId: string) {
  try {
    const db = getDatabase(dbId);
    const allMatchingRecords = await db.latestAccountAssets
      .where("accountId")
      .equals(accountId)
      .toArray();

    const assets = allMatchingRecords.map((record) => {
      return {
        asset: record.asset,
      };
    });

    return assets;
  } catch (error: unknown) {
    logWebStoreError(
      error,
      `Error fetching account vault for account ${accountId}`
    );
  }
}

export async function getAccountAuthByPubKeyCommitment(
  dbId: string,
  pubKeyCommitmentHex: string
) {
  const db = getDatabase(dbId);
  const accountSecretKey = await db.accountAuths
    .where("pubKeyCommitmentHex")
    .equals(pubKeyCommitmentHex)
    .first();

  if (!accountSecretKey) {
    throw new Error("Account auth not found in cache.");
  }

  const data = {
    secretKey: accountSecretKey.secretKeyHex,
  };

  return data;
}

export async function getAccountAddresses(dbId: string, accountId: string) {
  try {
    const db = getDatabase(dbId);
    const allMatchingRecords = await db.addresses
      .where("id")
      .equals(accountId)
      .toArray();

    if (allMatchingRecords.length === 0) {
      console.log("No address records found for given account ID.");
      return [];
    }

    return allMatchingRecords;
  } catch (error) {
    logWebStoreError(
      error,
      `Error while fetching account addresses for id: ${accountId}`
    );
  }
}

export async function upsertAccountCode(
  dbId: string,
  codeRoot: string,
  code: Uint8Array
) {
  try {
    const db = getDatabase(dbId);
    const data = {
      root: codeRoot,
      code,
    };

    await db.accountCodes.put(data);
  } catch (error) {
    logWebStoreError(error, `Error inserting code with root: ${codeRoot}`);
  }
}

export async function upsertAccountStorage(
  dbId: string,
  accountId: string,
  nonce: string,
  storageSlots: JsStorageSlot[]
) {
  try {
    const db = getDatabase(dbId);
    await db.latestAccountStorages
      .where("accountId")
      .equals(accountId)
      .delete();

    if (storageSlots.length === 0) return;

    const latestEntries = storageSlots.map((slot) => ({
      accountId,
      slotName: slot.slotName,
      slotValue: slot.slotValue,
      slotType: slot.slotType,
    }));

    const historicalEntries = latestEntries.map((entry) => ({
      ...entry,
      nonce,
    }));

    await db.latestAccountStorages.bulkPut(latestEntries);
    await db.historicalAccountStorages.bulkPut(historicalEntries);
  } catch (error) {
    logWebStoreError(error, `Error inserting storage slots`);
  }
}

export async function upsertStorageMapEntries(
  dbId: string,
  accountId: string,
  nonce: string,
  entries: JsStorageMapEntry[]
) {
  try {
    const db = getDatabase(dbId);

    // Read old latest entries before clearing, to detect removals
    const oldEntries = await db.latestStorageMapEntries
      .where("accountId")
      .equals(accountId)
      .toArray();

    await db.latestStorageMapEntries
      .where("accountId")
      .equals(accountId)
      .delete();

    // Build a set of new keys for fast lookup
    const newKeySet = new Set(entries.map((e) => `${e.slotName}\0${e.key}`));

    // Write tombstones to historical for entries that existed but are now absent
    for (const old of oldEntries) {
      if (!newKeySet.has(`${old.slotName}\0${old.key}`)) {
        await db.historicalStorageMapEntries.put({
          accountId,
          nonce,
          slotName: old.slotName,
          key: old.key,
          value: null,
        } as IHistoricalStorageMapEntry);
      }
    }

    if (entries.length === 0) return;

    const latestEntries = entries.map((entry) => ({
      accountId,
      slotName: entry.slotName,
      key: entry.key,
      value: entry.value,
    }));

    const historicalEntries = latestEntries.map((entry) => ({
      ...entry,
      nonce,
    }));

    await db.latestStorageMapEntries.bulkPut(latestEntries);
    await db.historicalStorageMapEntries.bulkPut(historicalEntries);
  } catch (error) {
    logWebStoreError(error, `Error inserting storage map entries`);
  }
}

export async function upsertVaultAssets(
  dbId: string,
  accountId: string,
  nonce: string,
  assets: JsVaultAsset[]
) {
  try {
    const db = getDatabase(dbId);

    // Read old latest entries before clearing, to detect removals
    const oldAssets = await db.latestAccountAssets
      .where("accountId")
      .equals(accountId)
      .toArray();

    await db.latestAccountAssets.where("accountId").equals(accountId).delete();

    // Build a set of new vault keys for fast lookup
    const newKeySet = new Set(assets.map((a) => a.vaultKey));

    // Write tombstones to historical for assets that existed but are now absent
    for (const old of oldAssets) {
      if (!newKeySet.has(old.vaultKey)) {
        await db.historicalAccountAssets.put({
          accountId,
          nonce,
          vaultKey: old.vaultKey,
          faucetIdPrefix: old.faucetIdPrefix,
          asset: null,
        } as IHistoricalAccountAsset);
      }
    }

    if (assets.length === 0) return;

    const latestEntries = assets.map((asset) => ({
      accountId,
      vaultKey: asset.vaultKey,
      faucetIdPrefix: asset.faucetIdPrefix,
      asset: asset.asset,
    }));

    const historicalEntries = latestEntries.map((entry) => ({
      ...entry,
      nonce,
    }));

    await db.latestAccountAssets.bulkPut(latestEntries);
    await db.historicalAccountAssets.bulkPut(historicalEntries);
  } catch (error: unknown) {
    logWebStoreError(error, `Error inserting assets`);
  }
}

export async function applyStorageDelta(
  dbId: string,
  accountId: string,
  nonce: string,
  updatedSlots: JsStorageSlot[],
  changedMapEntries: JsStorageMapEntry[]
) {
  try {
    const db = getDatabase(dbId);

    // Upsert updated slots to both latest and historical
    for (const slot of updatedSlots) {
      await db.latestAccountStorages.put({
        accountId,
        slotName: slot.slotName,
        slotValue: slot.slotValue,
        slotType: slot.slotType,
      });
      await db.historicalAccountStorages.put({
        accountId,
        nonce,
        slotName: slot.slotName,
        slotValue: slot.slotValue,
        slotType: slot.slotType,
      });
    }

    // Process map entries: value="" means removal
    for (const entry of changedMapEntries) {
      if (entry.value === "") {
        // Removal: delete from latest, write tombstone to historical
        await db.latestStorageMapEntries
          .where("[accountId+slotName+key]")
          .equals([accountId, entry.slotName, entry.key])
          .delete();
        await db.historicalStorageMapEntries.put({
          accountId,
          nonce,
          slotName: entry.slotName,
          key: entry.key,
          value: null,
        } as IHistoricalStorageMapEntry);
      } else {
        // Update: put to both latest and historical
        await db.latestStorageMapEntries.put({
          accountId,
          slotName: entry.slotName,
          key: entry.key,
          value: entry.value,
        });
        await db.historicalStorageMapEntries.put({
          accountId,
          nonce,
          slotName: entry.slotName,
          key: entry.key,
          value: entry.value,
        });
      }
    }
  } catch (error) {
    logWebStoreError(error, `Error applying storage delta`);
  }
}

export async function applyVaultDelta(
  dbId: string,
  accountId: string,
  nonce: string,
  changedAssets: JsVaultAsset[]
) {
  try {
    const db = getDatabase(dbId);

    for (const entry of changedAssets) {
      if (entry.asset === "") {
        // Removal: delete from latest, write tombstone to historical
        await db.latestAccountAssets
          .where("[accountId+vaultKey]")
          .equals([accountId, entry.vaultKey])
          .delete();
        await db.historicalAccountAssets.put({
          accountId,
          nonce,
          vaultKey: entry.vaultKey,
          faucetIdPrefix: entry.faucetIdPrefix,
          asset: null,
        } as IHistoricalAccountAsset);
      } else {
        // Update: put to both latest and historical
        await db.latestAccountAssets.put({
          accountId,
          vaultKey: entry.vaultKey,
          faucetIdPrefix: entry.faucetIdPrefix,
          asset: entry.asset,
        });
        await db.historicalAccountAssets.put({
          accountId,
          nonce,
          vaultKey: entry.vaultKey,
          faucetIdPrefix: entry.faucetIdPrefix,
          asset: entry.asset,
        });
      }
    }
  } catch (error) {
    logWebStoreError(error, `Error applying vault delta`);
  }
}

export async function upsertAccountRecord(
  dbId: string,
  accountId: string,
  codeRoot: string,
  storageRoot: string,
  vaultRoot: string,
  nonce: string,
  committed: boolean,
  commitment: string,
  accountSeed: Uint8Array | undefined
) {
  try {
    const db = getDatabase(dbId);
    const data = {
      id: accountId,
      codeRoot,
      storageRoot,
      vaultRoot,
      nonce,
      committed,
      accountSeed,
      accountCommitment: commitment,
      locked: false,
    };

    await db.historicalAccountHeaders.put(data as IAccount);
    await db.latestAccountHeaders.put(data as IAccount);
  } catch (error) {
    logWebStoreError(error, `Error inserting account: ${accountId}`);
  }
}

export async function insertAccountAuth(
  dbId: string,
  pubKeyCommitmentHex: string,
  secretKey: string
) {
  try {
    const db = getDatabase(dbId);
    const data = {
      pubKeyCommitmentHex,
      secretKeyHex: secretKey,
    };

    await db.accountAuths.add(data);
  } catch (error) {
    logWebStoreError(
      error,
      `Error inserting account auth for pubKey: ${pubKeyCommitmentHex}`
    );
  }
}

export async function insertAccountAddress(
  dbId: string,
  accountId: string,
  address: Uint8Array
) {
  try {
    const db = getDatabase(dbId);
    const data = {
      id: accountId,
      address,
    };

    await db.addresses.put(data);
  } catch (error) {
    logWebStoreError(
      error,
      `Error inserting address with value: ${String(address)} for the account ID ${accountId}`
    );
  }
}

export async function removeAccountAddress(dbId: string, address: Uint8Array) {
  try {
    const db = getDatabase(dbId);
    await db.addresses.where("address").equals(address).delete();
  } catch (error) {
    logWebStoreError(
      error,
      `Error removing address with value: ${String(address)}`
    );
  }
}

export async function upsertForeignAccountCode(
  dbId: string,
  accountId: string,
  code: Uint8Array,
  codeRoot: string
) {
  try {
    const db = getDatabase(dbId);
    await upsertAccountCode(dbId, codeRoot, code);

    const data = {
      accountId,
      codeRoot,
    };

    await db.foreignAccountCode.put(data);
  } catch (error) {
    logWebStoreError(
      error,
      `Error upserting foreign account code for account: ${accountId}`
    );
  }
}

export async function getForeignAccountCode(
  dbId: string,
  accountIds: string[]
) {
  try {
    const db = getDatabase(dbId);
    const foreignAccounts = await db.foreignAccountCode
      .where("accountId")
      .anyOf(accountIds)
      .toArray();

    if (foreignAccounts.length === 0) {
      console.log("No records found for the given account IDs.");
      return null;
    }

    const codeRoots = foreignAccounts.map((account) => account.codeRoot);

    const accountCode = await db.accountCodes
      .where("root")
      .anyOf(codeRoots)
      .toArray();

    const processedCode = foreignAccounts
      .map((foreignAccount) => {
        const matchingCode = accountCode.find(
          (code) => code.root === foreignAccount.codeRoot
        );

        if (matchingCode === undefined) {
          return undefined;
        }

        const codeBase64 = uint8ArrayToBase64(matchingCode.code);

        return {
          accountId: foreignAccount.accountId,
          code: codeBase64,
        };
      })
      .filter((matchingCode) => matchingCode !== undefined);
    return processedCode;
  } catch (error) {
    logWebStoreError(error, "Error fetching foreign account code");
  }
}

export async function lockAccount(dbId: string, accountId: string) {
  try {
    const db = getDatabase(dbId);
    await db.latestAccountHeaders
      .where("id")
      .equals(accountId)
      .modify({ locked: true });
    // Also lock historical rows so that undo/rebuild preserves the lock.
    await db.historicalAccountHeaders
      .where("id")
      .equals(accountId)
      .modify({ locked: true });
  } catch (error) {
    logWebStoreError(error, `Error locking account: ${accountId}`);
  }
}

/**
 * Rebuilds latest storage slots from historical data.
 * Groups by slotName, takes the entry with MAX(nonce) per slot.
 * Slots cannot be removed, so no tombstone filtering needed.
 */
async function rebuildLatestStorageSlots(db: MidenDatabase, accountId: string) {
  await db.latestAccountStorages.where("accountId").equals(accountId).delete();
  const allHist = await db.historicalAccountStorages
    .where("accountId")
    .equals(accountId)
    .toArray();

  // Group by slotName, take MAX(nonce) per slot
  const bySlot = new Map<string, (typeof allHist)[0]>();
  for (const entry of allHist) {
    const existing = bySlot.get(entry.slotName);
    if (!existing || BigInt(entry.nonce) > BigInt(existing.nonce)) {
      bySlot.set(entry.slotName, entry);
    }
  }

  if (bySlot.size > 0) {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const entries = [...bySlot.values()].map(({ nonce, ...rest }) => rest);
    await db.latestAccountStorages.bulkPut(entries);
  }
}

/**
 * Rebuilds latest storage map entries from historical data.
 * Groups by (slotName, key), takes the entry with MAX(nonce) per key.
 * Filters out tombstones (value === null).
 */
async function rebuildLatestStorageMapEntries(
  db: MidenDatabase,
  accountId: string
) {
  await db.latestStorageMapEntries
    .where("accountId")
    .equals(accountId)
    .delete();
  const allHist = await db.historicalStorageMapEntries
    .where("accountId")
    .equals(accountId)
    .toArray();

  // Group by (slotName, key), take MAX(nonce) per key
  const byKey = new Map<string, (typeof allHist)[0]>();
  for (const entry of allHist) {
    const compositeKey = `${entry.slotName}\0${entry.key}`;
    const existing = byKey.get(compositeKey);
    if (!existing || BigInt(entry.nonce) > BigInt(existing.nonce)) {
      byKey.set(compositeKey, entry);
    }
  }

  // Filter out tombstones and strip nonce
  const entries = [...byKey.values()]
    .filter((e) => e.value !== null)
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .map(({ nonce, value, ...rest }) => ({
      ...rest,
      value: value as string,
    }));
  if (entries.length > 0) {
    await db.latestStorageMapEntries.bulkPut(entries);
  }
}

/**
 * Rebuilds latest vault assets from historical data.
 * Groups by vaultKey, takes the entry with MAX(nonce) per key.
 * Filters out tombstones (asset === null).
 */
async function rebuildLatestVaultAssets(db: MidenDatabase, accountId: string) {
  await db.latestAccountAssets.where("accountId").equals(accountId).delete();
  const allHist = await db.historicalAccountAssets
    .where("accountId")
    .equals(accountId)
    .toArray();

  // Group by vaultKey, take MAX(nonce) per key
  const byKey = new Map<string, (typeof allHist)[0]>();
  for (const entry of allHist) {
    const existing = byKey.get(entry.vaultKey);
    if (!existing || BigInt(entry.nonce) > BigInt(existing.nonce)) {
      byKey.set(entry.vaultKey, entry);
    }
  }

  // Filter out tombstones and strip nonce
  const entries = [...byKey.values()]
    .filter((e) => e.asset !== null)
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .map(({ nonce, asset, ...rest }) => ({
      ...rest,
      asset: asset as string,
    }));
  if (entries.length > 0) {
    await db.latestAccountAssets.bulkPut(entries);
  }
}

export async function undoAccountStates(
  dbId: string,
  accountCommitments: string[]
) {
  try {
    const db = getDatabase(dbId);

    // Find affected records to get their account IDs and nonces before deleting
    const affectedRecords = await db.historicalAccountHeaders
      .where("accountCommitment")
      .anyOf(accountCommitments)
      .toArray();

    // Collect affected (accountId, nonce) pairs
    const accountNonces = new Map<string, Set<string>>();
    for (const record of affectedRecords) {
      if (!accountNonces.has(record.id)) {
        accountNonces.set(record.id, new Set());
      }
      accountNonces.get(record.id)!.add(record.nonce);
    }

    // Delete matching records from historical account headers
    await db.historicalAccountHeaders
      .where("accountCommitment")
      .anyOf(accountCommitments)
      .delete();

    // Delete historical storage/map/assets for affected (accountId, nonce) pairs
    for (const [accountId, nonces] of accountNonces) {
      for (const nonce of nonces) {
        await db.historicalAccountStorages
          .where("[accountId+nonce]")
          .equals([accountId, nonce])
          .delete();
        await db.historicalStorageMapEntries
          .where("[accountId+nonce]")
          .equals([accountId, nonce])
          .delete();
        await db.historicalAccountAssets
          .where("[accountId+nonce]")
          .equals([accountId, nonce])
          .delete();
      }
    }

    // Rebuild latest for each affected account
    for (const accountId of accountNonces.keys()) {
      const remaining = await db.historicalAccountHeaders
        .where("id")
        .equals(accountId)
        .toArray();

      if (remaining.length === 0) {
        // Account completely undone — clear all latest tables
        await db.latestAccountHeaders.where("id").equals(accountId).delete();
        await db.latestAccountStorages
          .where("accountId")
          .equals(accountId)
          .delete();
        await db.latestStorageMapEntries
          .where("accountId")
          .equals(accountId)
          .delete();
        await db.latestAccountAssets
          .where("accountId")
          .equals(accountId)
          .delete();
      } else {
        // Find the record with the highest nonce
        let maxRecord = remaining[0];
        for (const record of remaining) {
          if (BigInt(record.nonce) > BigInt(maxRecord.nonce)) {
            maxRecord = record;
          }
        }

        // Rebuild latest from historical using MAX(nonce) per key
        await db.latestAccountHeaders.put(maxRecord);
        await rebuildLatestStorageSlots(db, accountId);
        await rebuildLatestStorageMapEntries(db, accountId);
        await rebuildLatestVaultAssets(db, accountId);
      }
    }
  } catch (error) {
    logWebStoreError(
      error,
      `Error undoing account states: ${accountCommitments.join(",")}`
    );
  }
}

export async function removeAccountAuth(
  dbId: string,
  pubKeyCommitmentHex: string
) {
  try {
    const db = getDatabase(dbId);
    await db.accountAuths
      .where("pubKeyCommitmentHex")
      .equals(pubKeyCommitmentHex)
      .delete();
  } catch (error) {
    logWebStoreError(
      error,
      `Error removing account auth for pubKey: ${pubKeyCommitmentHex}`
    );
  }
}

export async function insertAccountKeyMapping(
  dbId: string,
  accountIdHex: string,
  pubKeyCommitmentHex: string
) {
  try {
    const db = getDatabase(dbId);
    const data = {
      accountIdHex,
      pubKeyCommitmentHex,
    };
    await db.accountKeyMappings.put(data);
  } catch (error) {
    logWebStoreError(
      error,
      `Error inserting account key mapping for account ${accountIdHex} and key ${pubKeyCommitmentHex}`
    );
  }
}

export async function removeAccountKeyMapping(
  dbId: string,
  accountIdHex: string,
  pubKeyCommitmentHex: string
): Promise<boolean> {
  try {
    const db = getDatabase(dbId);
    const deletedCount = await db.accountKeyMappings
      .where("[accountIdHex+pubKeyCommitmentHex]")
      .equals([accountIdHex, pubKeyCommitmentHex])
      .delete();
    return deletedCount > 0;
  } catch (error) {
    logWebStoreError(
      error,
      `Error removing account key mapping for account ${accountIdHex} and key ${pubKeyCommitmentHex}`
    );
    return false;
  }
}

export async function getKeyCommitmentsByAccountId(
  dbId: string,
  accountIdHex: string
): Promise<string[]> {
  try {
    const db = getDatabase(dbId);
    const mappings = await db.accountKeyMappings
      .where("accountIdHex")
      .equals(accountIdHex)
      .toArray();
    return mappings.map((mapping) => mapping.pubKeyCommitmentHex);
  } catch (error) {
    logWebStoreError(
      error,
      `Error getting key commitments for account: ${accountIdHex}`
    );
    return [];
  }
}

export async function removeAllMappingsForKey(
  dbId: string,
  pubKeyCommitmentHex: string
) {
  try {
    const db = getDatabase(dbId);
    await db.accountKeyMappings
      .where("pubKeyCommitmentHex")
      .equals(pubKeyCommitmentHex)
      .delete();
  } catch (error) {
    logWebStoreError(
      error,
      `Error removing all mappings for key: ${pubKeyCommitmentHex}`
    );
  }
}

export async function getAccountIdByKeyCommitment(
  dbId: string,
  pubKeyCommitmentHex: string
): Promise<string | null> {
  try {
    const db = getDatabase(dbId);
    const mapping = await db.accountKeyMappings
      .where("pubKeyCommitmentHex")
      .equals(pubKeyCommitmentHex)
      .first();
    return mapping?.accountIdHex ?? null;
  } catch (error) {
    logWebStoreError(
      error,
      `Error fetching account by public key commitment: ${pubKeyCommitmentHex}`
    );
    return null;
  }
}
