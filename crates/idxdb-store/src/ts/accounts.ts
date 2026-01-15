import {
  accountCodes,
  accountStorages,
  accountAssets,
  accountAuths,
  accounts,
  addresses,
  foreignAccountCode,
  IAccount,
  IAccountAsset,
  IAccountStorage,
  ITrackedAccount,
  storageMapEntries,
  IStorageMapEntry,
  trackedAccounts,
} from "./schema.js";
import { JsStorageMapEntry, JsStorageSlot, JsVaultAsset } from "./sync.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

// GET FUNCTIONS
export async function getAccountIds() {
  try {
    const tracked = await trackedAccounts.toArray();
    return tracked.map((entry) => entry.id);
  } catch (error) {
    logWebStoreError(error, "Error while fetching account IDs");
  }

  return [];
}

export async function getAllAccountHeaders() {
  try {
    // Use a Map to track the latest record for each id based on nonce
    const latestRecordsMap: Map<string, IAccount> = new Map();

    await accounts.each((record) => {
      const existingRecord = latestRecordsMap.get(record.id);
      if (
        !existingRecord ||
        BigInt(record.nonce) > BigInt(existingRecord.nonce)
      ) {
        latestRecordsMap.set(record.id, record);
      }
    });

    // Extract the latest records from the Map
    const latestRecords = Array.from(latestRecordsMap.values());

    const resultObject = await Promise.all(
      latestRecords.map((record) => {
        let accountSeedBase64: string | undefined = undefined;
        if (record.accountSeed) {
          const seedAsBytes = new Uint8Array(record.accountSeed);
          if (seedAsBytes.length > 0) {
            accountSeedBase64 = uint8ArrayToBase64(seedAsBytes);
          }
        }

        return {
          id: record.id,
          nonce: record.nonce,
          vaultRoot: record.vaultRoot, // Fallback if missing
          storageRoot: record.storageRoot || "",
          codeRoot: record.codeRoot || "",
          accountSeed: accountSeedBase64, // null or base64 string
          locked: record.locked,
          committed: record.committed, // Use actual value or default
          accountCommitment: record.accountCommitment || "", // Keep original field name
        };
      })
    );

    return resultObject;
  } catch (error) {
    logWebStoreError(error, "Error while fetching account headers");
  }
}

export async function getAccountHeader(accountId: string) {
  try {
    // Fetch all records matching the given id
    const allMatchingRecords = await accounts
      .where("id")
      .equals(accountId)
      .toArray();

    if (allMatchingRecords.length === 0) {
      console.log("No account header record found for given ID.");
      return null;
    }

    // Convert nonce to BigInt and sort
    // Note: This assumes all nonces are valid BigInt strings.
    const sortedRecords = allMatchingRecords.sort((a, b) => {
      const bigIntA = BigInt(a.nonce);
      const bigIntB = BigInt(b.nonce);
      return bigIntA > bigIntB ? -1 : bigIntA < bigIntB ? 1 : 0;
    });

    // The first record is the most recent one due to the sorting
    const mostRecentRecord: IAccount | undefined = sortedRecords[0];

    if (mostRecentRecord === undefined) {
      return null;
    }

    let accountSeedBase64: string | undefined = undefined;

    if (mostRecentRecord.accountSeed) {
      // Ensure accountSeed is processed as a Uint8Array and converted to Base64
      if (mostRecentRecord.accountSeed.length > 0) {
        accountSeedBase64 = uint8ArrayToBase64(mostRecentRecord.accountSeed);
      }
    }
    const AccountHeader = {
      id: mostRecentRecord.id,
      nonce: mostRecentRecord.nonce,
      vaultRoot: mostRecentRecord.vaultRoot,
      storageRoot: mostRecentRecord.storageRoot,
      codeRoot: mostRecentRecord.codeRoot,
      accountSeed: accountSeedBase64,
      locked: mostRecentRecord.locked,
    };
    return AccountHeader;
  } catch (error) {
    logWebStoreError(
      error,
      `Error while fetching account header for id: ${accountId}`
    );
  }
}

export async function getAccountHeaderByCommitment(accountCommitment: string) {
  try {
    // Fetch all records matching the given commitment
    const allMatchingRecords = await accounts
      .where("accountCommitment")
      .equals(accountCommitment)
      .toArray();

    if (allMatchingRecords.length == 0) {
      return undefined;
    }

    // There should be only one match
    const matchingRecord: IAccount | undefined = allMatchingRecords[0];

    if (matchingRecord === undefined) {
      console.log("No account header record found for given commitment.");
      return null;
    }

    let accountSeedBase64: string | undefined = undefined;
    if (matchingRecord.accountSeed) {
      accountSeedBase64 = uint8ArrayToBase64(matchingRecord.accountSeed);
    }
    const AccountHeader = {
      id: matchingRecord.id,
      nonce: matchingRecord.nonce,
      vaultRoot: matchingRecord.vaultRoot,
      storageRoot: matchingRecord.storageRoot,
      codeRoot: matchingRecord.codeRoot,
      accountSeed: accountSeedBase64,
      locked: matchingRecord.locked,
    };
    return AccountHeader;
  } catch (error) {
    logWebStoreError(
      error,
      `Error fetching account header for commitment ${accountCommitment}`
    );
  }
}

export async function getAccountCode(codeRoot: string) {
  try {
    // Fetch all records matching the given root
    const allMatchingRecords = await accountCodes
      .where("root")
      .equals(codeRoot)
      .toArray();

    // The first record is the only one due to the uniqueness constraint
    const codeRecord = allMatchingRecords[0];

    if (codeRecord === undefined) {
      console.log("No records found for given code root.");
      return null;
    }

    // Convert the code Blob to an ArrayBuffer
    const codeBase64 = uint8ArrayToBase64(codeRecord.code);
    return {
      root: codeRecord.root,
      code: codeBase64,
    };
  } catch (error) {
    logWebStoreError(error, `Error fetching account code for root ${codeRoot}`);
  }
}

export async function getAccountStorage(storageCommitment: string) {
  try {
    const allMatchingRecords = await accountStorages
      .where("commitment")
      .equals(storageCommitment)
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
      `Error fetching account storage for commitment ${storageCommitment}`
    );
  }
}

export async function getAccountStorageMaps(roots: string[]) {
  try {
    const allMatchingRecords = await storageMapEntries
      .where("root")
      .anyOf(roots)
      .toArray();

    return allMatchingRecords;
  } catch (error) {
    logWebStoreError(
      error,
      `Error fetching account storage maps for roots ${roots.join(", ")}`
    );
  }
}

export async function getAccountVaultAssets(vaultRoot: string) {
  try {
    // Fetch all records matching the given root
    const allMatchingRecords = await accountAssets
      .where("root")
      .equals(vaultRoot)
      .toArray();

    // Map the records to their asset values
    const assets = allMatchingRecords.map((record) => {
      return {
        asset: record.asset,
      };
    });

    return assets;
  } catch (error: unknown) {
    logWebStoreError(
      error,
      `Error fetching account vault for root ${vaultRoot}`
    );
  }
}

export async function getAccountAuthByPubKey(pubKey: string) {
  // Try to get the account auth from the store
  const accountSecretKey = await accountAuths
    .where("pubKey")
    .equals(pubKey)
    .first();

  // If it's not in the cache, throw an error
  if (!accountSecretKey) {
    throw new Error("Account auth not found in cache.");
  }

  const data = {
    secretKey: accountSecretKey.secretKey,
  };

  return data;
}

export async function getAccountAddresses(accountId: string) {
  try {
    // Fetch all records matching the given accountId
    const allMatchingRecords = await addresses
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

// INSERT FUNCTIONS

export async function upsertAccountCode(codeRoot: string, code: Uint8Array) {
  try {
    // Prepare the data object to insert
    const data = {
      root: codeRoot, // Using codeRoot as the key
      code,
    };

    // Perform the insert using Dexie
    await accountCodes.put(data);
  } catch (error) {
    logWebStoreError(error, `Error inserting code with root: ${codeRoot}`);
  }
}

export async function upsertAccountStorage(storageSlots: JsStorageSlot[]) {
  try {
    let processedSlots = storageSlots.map((slot) => {
      return {
        commitment: slot.commitment,
        slotName: slot.slotName,
        slotValue: slot.slotValue,
        slotType: slot.slotType,
      } as IAccountStorage;
    });

    await accountStorages.bulkPut(processedSlots);
  } catch (error) {
    logWebStoreError(error, `Error inserting storage slots`);
  }
}

export async function upsertStorageMapEntries(entries: JsStorageMapEntry[]) {
  try {
    let processedEntries = entries.map((entry) => {
      return {
        root: entry.root,
        key: entry.key,
        value: entry.value,
      } as IStorageMapEntry;
    });

    await storageMapEntries.bulkPut(processedEntries);
  } catch (error) {
    logWebStoreError(error, `Error inserting storage map entries`);
  }
}

export async function upsertVaultAssets(assets: JsVaultAsset[]) {
  try {
    let processedAssets = assets.map((asset) => {
      return {
        root: asset.root,
        vaultKey: asset.vaultKey,
        faucetIdPrefix: asset.faucetIdPrefix,
        asset: asset.asset,
      } as IAccountAsset;
    });

    await accountAssets.bulkPut(processedAssets);
  } catch (error: unknown) {
    logWebStoreError(error, `Error inserting assets`);
  }
}
export async function upsertAccountRecord(
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

    await accounts.put(data as IAccount);
    await trackedAccounts.put({ id: accountId } as ITrackedAccount);
  } catch (error) {
    logWebStoreError(error, `Error inserting account: ${accountId}`);
  }
}

export async function insertAccountAuth(pubKey: string, secretKey: string) {
  try {
    // Prepare the data object to insert
    const data = {
      pubKey: pubKey,
      secretKey: secretKey,
    };

    // Perform the insert using Dexie
    await accountAuths.add(data);
  } catch (error) {
    logWebStoreError(
      error,
      `Error inserting account auth for pubKey: ${pubKey}`
    );
  }
}

export async function insertAccountAddress(
  accountId: string,
  address: Uint8Array
) {
  try {
    // Prepare the data object to insert
    const data = {
      id: accountId,
      address,
    };

    // Perform the insert using Dexie
    await addresses.put(data);
  } catch (error) {
    logWebStoreError(
      error,
      `Error inserting address with value: ${String(address)} for the account ID ${accountId}`
    );
  }
}

export async function removeAccountAddress(address: Uint8Array) {
  try {
    // Perform the delete using Dexie
    await addresses.where("address").equals(address).delete();
  } catch (error) {
    logWebStoreError(
      error,
      `Error removing address with value: ${String(address)}`
    );
  }
}

export async function upsertForeignAccountCode(
  accountId: string,
  code: Uint8Array,
  codeRoot: string
) {
  try {
    await upsertAccountCode(codeRoot, code);

    const data = {
      accountId,
      codeRoot,
    };

    await foreignAccountCode.put(data);
  } catch (error) {
    logWebStoreError(
      error,
      `Error upserting foreign account code for account: ${accountId}`
    );
  }
}

export async function getForeignAccountCode(accountIds: string[]) {
  try {
    const foreignAccounts = await foreignAccountCode
      .where("accountId")
      .anyOf(accountIds)
      .toArray();

    if (foreignAccounts.length === 0) {
      console.log("No records found for the given account IDs.");
      return null; // No records found
    }

    const codeRoots = foreignAccounts.map((account) => account.codeRoot);

    const accountCode = await accountCodes
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

export async function lockAccount(accountId: string) {
  try {
    await accounts.where("id").equals(accountId).modify({ locked: true });
  } catch (error) {
    logWebStoreError(error, `Error locking account: ${accountId}`);
  }
}

// Delete functions
export async function undoAccountStates(accountCommitments: string[]) {
  try {
    await accounts
      .where("accountCommitment")
      .anyOf(accountCommitments)
      .delete();
  } catch (error) {
    logWebStoreError(
      error,
      `Error undoing account states: ${accountCommitments.join(",")}`
    );
  }
}
