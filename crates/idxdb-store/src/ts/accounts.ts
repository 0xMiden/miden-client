import {
  getDatabase,
  IAccount,
  JsStorageMapEntry,
  JsStorageSlot,
  JsVaultAsset,
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

    await db.latestAccountStorages.bulkPut(latestEntries);
  } catch (error) {
    logWebStoreError(error, `Error inserting storage slots`);
  }
}

export async function upsertStorageMapEntries(
  dbId: string,
  accountId: string,
  entries: JsStorageMapEntry[]
) {
  try {
    const db = getDatabase(dbId);

    await db.latestStorageMapEntries
      .where("accountId")
      .equals(accountId)
      .delete();

    if (entries.length === 0) return;

    const latestEntries = entries.map((entry) => ({
      accountId,
      slotName: entry.slotName,
      key: entry.key,
      value: entry.value,
    }));

    await db.latestStorageMapEntries.bulkPut(latestEntries);
  } catch (error) {
    logWebStoreError(error, `Error inserting storage map entries`);
  }
}

export async function upsertVaultAssets(
  dbId: string,
  accountId: string,
  assets: JsVaultAsset[]
) {
  try {
    const db = getDatabase(dbId);

    await db.latestAccountAssets.where("accountId").equals(accountId).delete();

    if (assets.length === 0) return;

    const latestEntries = assets.map((asset) => ({
      accountId,
      vaultKey: asset.vaultKey,
      faucetIdPrefix: asset.faucetIdPrefix,
      asset: asset.asset,
    }));

    await db.latestAccountAssets.bulkPut(latestEntries);
  } catch (error: unknown) {
    logWebStoreError(error, `Error inserting assets`);
  }
}

export async function applyTransactionDelta(
  dbId: string,
  accountId: string,
  nonce: string,
  updatedSlots: JsStorageSlot[],
  changedMapEntries: JsStorageMapEntry[],
  changedAssets: JsVaultAsset[],
  codeRoot: string,
  storageRoot: string,
  vaultRoot: string,
  committed: boolean,
  commitment: string,
  accountSeed: Uint8Array | undefined
) {
  try {
    const db = getDatabase(dbId);

    await db.dexie.transaction(
      "rw",
      [
        db.latestAccountStorages,
        db.historicalAccountStorages,
        db.latestStorageMapEntries,
        db.historicalStorageMapEntries,
        db.latestAccountAssets,
        db.historicalAccountAssets,
        db.latestAccountHeaders,
        db.historicalAccountHeaders,
      ],
      async () => {
        // Apply storage delta: read old → archive → write new
        for (const slot of updatedSlots) {
          const oldSlot = await db.latestAccountStorages
            .where("[accountId+slotName]")
            .equals([accountId, slot.slotName])
            .first();

          await db.historicalAccountStorages.put({
            accountId,
            replacedAtNonce: nonce,
            slotName: slot.slotName,
            oldSlotValue: oldSlot?.slotValue ?? null,
            slotType: slot.slotType,
          });

          await db.latestAccountStorages.put({
            accountId,
            slotName: slot.slotName,
            slotValue: slot.slotValue,
            slotType: slot.slotType,
          });
        }

        // Process map entries: read old → archive → update latest
        for (const entry of changedMapEntries) {
          const oldEntry = await db.latestStorageMapEntries
            .where("[accountId+slotName+key]")
            .equals([accountId, entry.slotName, entry.key])
            .first();

          await db.historicalStorageMapEntries.put({
            accountId,
            replacedAtNonce: nonce,
            slotName: entry.slotName,
            key: entry.key,
            oldValue: oldEntry?.value ?? null,
          });

          // "" means removal
          if (entry.value === "") {
            await db.latestStorageMapEntries
              .where("[accountId+slotName+key]")
              .equals([accountId, entry.slotName, entry.key])
              .delete();
          } else {
            await db.latestStorageMapEntries.put({
              accountId,
              slotName: entry.slotName,
              key: entry.key,
              value: entry.value,
            });
          }
        }

        // Apply vault delta: read old → archive → update latest
        for (const entry of changedAssets) {
          const oldAsset = await db.latestAccountAssets
            .where("[accountId+vaultKey]")
            .equals([accountId, entry.vaultKey])
            .first();

          await db.historicalAccountAssets.put({
            accountId,
            replacedAtNonce: nonce,
            vaultKey: entry.vaultKey,
            faucetIdPrefix: entry.faucetIdPrefix,
            oldAsset: oldAsset?.asset ?? null,
          });

          // "" means removal
          if (entry.asset === "") {
            await db.latestAccountAssets
              .where("[accountId+vaultKey]")
              .equals([accountId, entry.vaultKey])
              .delete();
          } else {
            await db.latestAccountAssets.put({
              accountId,
              vaultKey: entry.vaultKey,
              faucetIdPrefix: entry.faucetIdPrefix,
              asset: entry.asset,
            });
          }
        }

        // Archive old header and write new header
        const oldHeader = await db.latestAccountHeaders
          .where("id")
          .equals(accountId)
          .first();

        if (oldHeader) {
          await db.historicalAccountHeaders.put({
            id: accountId,
            replacedAtNonce: nonce,
            codeRoot: oldHeader.codeRoot,
            storageRoot: oldHeader.storageRoot,
            vaultRoot: oldHeader.vaultRoot,
            nonce: oldHeader.nonce,
            committed: oldHeader.committed,
            accountSeed: oldHeader.accountSeed,
            accountCommitment: oldHeader.accountCommitment,
            locked: oldHeader.locked,
          });
        }

        await db.latestAccountHeaders.put({
          id: accountId,
          codeRoot,
          storageRoot,
          vaultRoot,
          nonce,
          committed,
          accountSeed,
          accountCommitment: commitment,
          locked: false,
        } as IAccount);
      }
    );
  } catch (error) {
    logWebStoreError(error, `Error applying transaction delta`);
  }
}

export async function applyFullAccountState(
  dbId: string,
  accountState: {
    accountId: string;
    nonce: string;
    storageSlots: JsStorageSlot[];
    storageMapEntries: JsStorageMapEntry[];
    assets: JsVaultAsset[];
    codeRoot: string;
    storageRoot: string;
    vaultRoot: string;
    committed: boolean;
    accountCommitment: string;
    accountSeed: Uint8Array | undefined;
  }
) {
  try {
    const db = getDatabase(dbId);
    const {
      accountId,
      nonce,
      storageSlots,
      storageMapEntries,
      assets,
      codeRoot,
      storageRoot,
      vaultRoot,
      committed,
      accountCommitment,
      accountSeed,
    } = accountState;

    await db.dexie.transaction(
      "rw",
      [
        db.latestAccountStorages,
        db.historicalAccountStorages,
        db.latestStorageMapEntries,
        db.historicalStorageMapEntries,
        db.latestAccountAssets,
        db.historicalAccountAssets,
        db.latestAccountHeaders,
        db.historicalAccountHeaders,
      ],
      async () => {
        // Read all old latest entries before replacing
        const oldSlots = await db.latestAccountStorages
          .where("accountId")
          .equals(accountId)
          .toArray();
        const oldMapEntries = await db.latestStorageMapEntries
          .where("accountId")
          .equals(accountId)
          .toArray();
        const oldAssets = await db.latestAccountAssets
          .where("accountId")
          .equals(accountId)
          .toArray();
        const oldHeader = await db.latestAccountHeaders
          .where("id")
          .equals(accountId)
          .first();

        // Archive old storage slots to historical
        for (const slot of oldSlots) {
          await db.historicalAccountStorages.put({
            accountId,
            replacedAtNonce: nonce,
            slotName: slot.slotName,
            oldSlotValue: slot.slotValue,
            slotType: slot.slotType,
          });
        }

        // Archive old storage map entries to historical
        for (const entry of oldMapEntries) {
          await db.historicalStorageMapEntries.put({
            accountId,
            replacedAtNonce: nonce,
            slotName: entry.slotName,
            key: entry.key,
            oldValue: entry.value,
          });
        }

        // Archive old vault assets to historical
        for (const asset of oldAssets) {
          await db.historicalAccountAssets.put({
            accountId,
            replacedAtNonce: nonce,
            vaultKey: asset.vaultKey,
            faucetIdPrefix: asset.faucetIdPrefix,
            oldAsset: asset.asset,
          });
        }

        // Build sets of old keys to detect genuinely new entries
        const oldSlotNames = new Set(oldSlots.map((s) => s.slotName));
        const oldMapKeys = new Set(
          oldMapEntries.map((e) => `${e.slotName}\0${e.key}`)
        );
        const oldAssetKeys = new Set(oldAssets.map((a) => a.vaultKey));

        // Delete all latest entries and insert new
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

        // Insert new storage slots to latest
        if (storageSlots.length > 0) {
          await db.latestAccountStorages.bulkPut(
            storageSlots.map((slot) => ({
              accountId,
              slotName: slot.slotName,
              slotValue: slot.slotValue,
              slotType: slot.slotType,
            }))
          );
        }

        // Write NULL entries for genuinely new storage slots
        for (const slot of storageSlots) {
          if (!oldSlotNames.has(slot.slotName)) {
            await db.historicalAccountStorages.put({
              accountId,
              replacedAtNonce: nonce,
              slotName: slot.slotName,
              oldSlotValue: null,
              slotType: slot.slotType,
            });
          }
        }

        // Insert new map entries to latest
        if (storageMapEntries.length > 0) {
          await db.latestStorageMapEntries.bulkPut(
            storageMapEntries.map((entry) => ({
              accountId,
              slotName: entry.slotName,
              key: entry.key,
              value: entry.value,
            }))
          );
        }

        // Write NULL entries for genuinely new map entries
        for (const entry of storageMapEntries) {
          if (!oldMapKeys.has(`${entry.slotName}\0${entry.key}`)) {
            await db.historicalStorageMapEntries.put({
              accountId,
              replacedAtNonce: nonce,
              slotName: entry.slotName,
              key: entry.key,
              oldValue: null,
            });
          }
        }

        // Insert new vault assets to latest
        if (assets.length > 0) {
          await db.latestAccountAssets.bulkPut(
            assets.map((asset) => ({
              accountId,
              vaultKey: asset.vaultKey,
              faucetIdPrefix: asset.faucetIdPrefix,
              asset: asset.asset,
            }))
          );
        }

        // Write NULL entries for genuinely new assets
        for (const asset of assets) {
          if (!oldAssetKeys.has(asset.vaultKey)) {
            await db.historicalAccountAssets.put({
              accountId,
              replacedAtNonce: nonce,
              vaultKey: asset.vaultKey,
              faucetIdPrefix: asset.faucetIdPrefix,
              oldAsset: null,
            });
          }
        }

        // Archive old header and write new header
        if (oldHeader) {
          await db.historicalAccountHeaders.put({
            id: accountId,
            replacedAtNonce: nonce,
            codeRoot: oldHeader.codeRoot,
            storageRoot: oldHeader.storageRoot,
            vaultRoot: oldHeader.vaultRoot,
            nonce: oldHeader.nonce,
            committed: oldHeader.committed,
            accountSeed: oldHeader.accountSeed,
            accountCommitment: oldHeader.accountCommitment,
            locked: oldHeader.locked,
          });
        }

        await db.latestAccountHeaders.put({
          id: accountId,
          codeRoot,
          storageRoot,
          vaultRoot,
          nonce,
          committed,
          accountSeed,
          accountCommitment,
          locked: false,
        } as IAccount);
      }
    );
  } catch (error) {
    logWebStoreError(error, `Error applying full account state`);
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

    await db.latestAccountHeaders.put(data as IAccount);
  } catch (error) {
    logWebStoreError(error, `Error inserting account: ${accountId}`);
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

export async function undoAccountStates(
  dbId: string,
  accountCommitments: string[]
) {
  try {
    const db = getDatabase(dbId);

    await db.dexie.transaction(
      "rw",
      [
        db.latestAccountStorages,
        db.historicalAccountStorages,
        db.latestStorageMapEntries,
        db.historicalStorageMapEntries,
        db.latestAccountAssets,
        db.historicalAccountAssets,
        db.latestAccountHeaders,
        db.historicalAccountHeaders,
      ],
      async () => {
        // Step 1: Resolve nonces from both latest and historical headers
        const accountNonces = new Map<string, Set<string>>();

        for (const commitment of accountCommitments) {
          const latestRecord = await db.latestAccountHeaders
            .where("accountCommitment")
            .equals(commitment)
            .first();

          if (latestRecord) {
            if (!accountNonces.has(latestRecord.id)) {
              accountNonces.set(latestRecord.id, new Set());
            }
            accountNonces.get(latestRecord.id)!.add(latestRecord.nonce);
            continue;
          }

          const histRecord = await db.historicalAccountHeaders
            .where("accountCommitment")
            .equals(commitment)
            .first();

          if (histRecord) {
            if (!accountNonces.has(histRecord.id)) {
              accountNonces.set(histRecord.id, new Set());
            }
            accountNonces.get(histRecord.id)!.add(histRecord.nonce);
          }
        }

        // Step 2: Process each account, nonces in reverse order
        for (const [accountId, nonces] of accountNonces) {
          const sortedNonces = [...nonces].sort((a, b) =>
            Number(BigInt(b) - BigInt(a))
          );

          for (const nonce of sortedNonces) {
            // Restore old storage slots
            const oldSlots = await db.historicalAccountStorages
              .where("[accountId+replacedAtNonce]")
              .equals([accountId, nonce])
              .toArray();

            for (const slot of oldSlots) {
              if (slot.oldSlotValue !== null) {
                await db.latestAccountStorages.put({
                  accountId: slot.accountId,
                  slotName: slot.slotName,
                  slotValue: slot.oldSlotValue,
                  slotType: slot.slotType,
                });
              } else {
                await db.latestAccountStorages
                  .where("[accountId+slotName]")
                  .equals([accountId, slot.slotName])
                  .delete();
              }
            }

            // Restore old storage map entries
            const oldMapEntries = await db.historicalStorageMapEntries
              .where("[accountId+replacedAtNonce]")
              .equals([accountId, nonce])
              .toArray();

            for (const entry of oldMapEntries) {
              if (entry.oldValue !== null) {
                await db.latestStorageMapEntries.put({
                  accountId: entry.accountId,
                  slotName: entry.slotName,
                  key: entry.key,
                  value: entry.oldValue,
                });
              } else {
                await db.latestStorageMapEntries
                  .where("[accountId+slotName+key]")
                  .equals([accountId, entry.slotName, entry.key])
                  .delete();
              }
            }

            // Restore old vault assets
            const oldAssets = await db.historicalAccountAssets
              .where("[accountId+replacedAtNonce]")
              .equals([accountId, nonce])
              .toArray();

            for (const asset of oldAssets) {
              if (asset.oldAsset !== null) {
                await db.latestAccountAssets.put({
                  accountId: asset.accountId,
                  vaultKey: asset.vaultKey,
                  faucetIdPrefix: asset.faucetIdPrefix,
                  asset: asset.oldAsset,
                });
              } else {
                await db.latestAccountAssets
                  .where("[accountId+vaultKey]")
                  .equals([accountId, asset.vaultKey])
                  .delete();
              }
            }

            // Restore old header or delete account if no old header exists
            const oldHeader = await db.historicalAccountHeaders
              .where("[id+replacedAtNonce]")
              .equals([accountId, nonce])
              .first();

            if (oldHeader) {
              await db.latestAccountHeaders.put({
                id: oldHeader.id,
                codeRoot: oldHeader.codeRoot,
                storageRoot: oldHeader.storageRoot,
                vaultRoot: oldHeader.vaultRoot,
                nonce: oldHeader.nonce,
                committed: oldHeader.committed,
                accountSeed: oldHeader.accountSeed,
                accountCommitment: oldHeader.accountCommitment,
                locked: oldHeader.locked,
              } as IAccount);
            } else {
              await db.latestAccountHeaders
                .where("id")
                .equals(accountId)
                .delete();
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
            }

            // Delete consumed historical entries for this nonce
            await db.historicalAccountStorages
              .where("[accountId+replacedAtNonce]")
              .equals([accountId, nonce])
              .delete();
            await db.historicalStorageMapEntries
              .where("[accountId+replacedAtNonce]")
              .equals([accountId, nonce])
              .delete();
            await db.historicalAccountAssets
              .where("[accountId+replacedAtNonce]")
              .equals([accountId, nonce])
              .delete();
            await db.historicalAccountHeaders
              .where("[id+replacedAtNonce]")
              .equals([accountId, nonce])
              .delete();
          }
        }
      }
    );
  } catch (error) {
    logWebStoreError(
      error,
      `Error undoing account states: ${accountCommitments.join(",")}`
    );
  }
}
