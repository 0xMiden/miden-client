import {
  getDatabase,
  IAccount,
  IAccountAsset,
  IAccountStorage,
  ITrackedAccount,
  IStorageMapEntry,
  JsStorageMapEntry,
  JsStorageSlot,
  JsVaultAsset,
} from "./schema.js";
import { encryptSecretKey, decryptSecretKey } from "./crypto.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

export async function getAccountIds(dbId: string) {
  try {
    const db = getDatabase(dbId);
    const tracked = await db.trackedAccounts.toArray();
    return tracked.map((entry) => entry.id);
  } catch (error) {
    logWebStoreError(error, "Error while fetching account IDs");
  }

  return [];
}

export async function getAllAccountHeaders(dbId: string) {
  try {
    const db = getDatabase(dbId);
    const latestRecordsMap: Map<string, IAccount> = new Map();

    await db.accounts.each((record) => {
      const existingRecord = latestRecordsMap.get(record.id);
      if (
        !existingRecord ||
        BigInt(record.nonce) > BigInt(existingRecord.nonce)
      ) {
        latestRecordsMap.set(record.id, record);
      }
    });

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
          vaultRoot: record.vaultRoot,
          storageRoot: record.storageRoot || "",
          codeRoot: record.codeRoot || "",
          accountSeed: accountSeedBase64,
          locked: record.locked,
          committed: record.committed,
          accountCommitment: record.accountCommitment || "",
        };
      })
    );

    return resultObject;
  } catch (error) {
    logWebStoreError(error, "Error while fetching account headers");
  }
}

export async function getAccountHeader(dbId: string, accountId: string) {
  try {
    const db = getDatabase(dbId);
    const allMatchingRecords = await db.accounts
      .where("id")
      .equals(accountId)
      .toArray();

    if (allMatchingRecords.length === 0) {
      console.log("No account header record found for given ID.");
      return null;
    }

    const sortedRecords = allMatchingRecords.sort((a, b) => {
      const bigIntA = BigInt(a.nonce);
      const bigIntB = BigInt(b.nonce);
      return bigIntA > bigIntB ? -1 : bigIntA < bigIntB ? 1 : 0;
    });

    const mostRecentRecord: IAccount | undefined = sortedRecords[0];

    if (mostRecentRecord === undefined) {
      return null;
    }

    let accountSeedBase64: string | undefined = undefined;

    if (mostRecentRecord.accountSeed) {
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

export async function getAccountHeaderByCommitment(
  dbId: string,
  accountCommitment: string
) {
  try {
    const db = getDatabase(dbId);
    const allMatchingRecords = await db.accounts
      .where("accountCommitment")
      .equals(accountCommitment)
      .toArray();

    if (allMatchingRecords.length == 0) {
      return undefined;
    }

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

export async function getAccountStorage(
  dbId: string,
  storageCommitment: string
) {
  try {
    const db = getDatabase(dbId);
    const allMatchingRecords = await db.accountStorages
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

export async function getAccountStorageMaps(dbId: string, roots: string[]) {
  try {
    const db = getDatabase(dbId);
    const allMatchingRecords = await db.storageMapEntries
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

export async function getAccountVaultAssets(dbId: string, vaultRoot: string) {
  try {
    const db = getDatabase(dbId);
    const allMatchingRecords = await db.accountAssets
      .where("root")
      .equals(vaultRoot)
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
      `Error fetching account vault for root ${vaultRoot}`
    );
  }
}

export async function getAccountAuthByPubKeyCommitment(
  dbId: string,
  pubKeyCommitmentHex: string
) {
  const db = getDatabase(dbId);
  const accountAuth = await db.accountAuths
    .where("pubKeyCommitmentHex")
    .equals(pubKeyCommitmentHex)
    .first();

  if (!accountAuth) {
    throw new Error("Account auth not found in cache.");
  }

  let secretKeyHex: string;

  if (accountAuth.encryptedSecretKey && accountAuth.iv) {
    // Decrypt the encrypted secret key
    try {
      secretKeyHex = await decryptSecretKey(
        dbId,
        accountAuth.encryptedSecretKey,
        accountAuth.iv
      );
    } catch {
      // Decryption failed — the encryption key was likely lost (e.g. after a
      // store export/import cycle). The CryptoKey is non-exportable and does
      // not survive serialization.
      throw new Error(
        "Failed to decrypt account auth secret key. " +
          "The encryption key may have been lost after a store export/import. " +
          "The account must be re-imported from an account file."
      );
    }
  } else if (accountAuth.secretKeyHex) {
    // Legacy fallback: plaintext secret key — migrate to encrypted in place
    secretKeyHex = accountAuth.secretKeyHex;
    const { encrypted, iv } = await encryptSecretKey(dbId, secretKeyHex);
    await db.accountAuths.put({
      pubKeyCommitmentHex,
      encryptedSecretKey: encrypted,
      iv,
    });
  } else {
    throw new Error("Account auth record has no secret key data.");
  }

  return { secretKey: secretKeyHex };
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
  storageSlots: JsStorageSlot[]
) {
  try {
    const db = getDatabase(dbId);
    let processedSlots = storageSlots.map((slot) => {
      return {
        commitment: slot.commitment,
        slotName: slot.slotName,
        slotValue: slot.slotValue,
        slotType: slot.slotType,
      } as IAccountStorage;
    });

    await db.accountStorages.bulkPut(processedSlots);
  } catch (error) {
    logWebStoreError(error, `Error inserting storage slots`);
  }
}

export async function upsertStorageMapEntries(
  dbId: string,
  entries: JsStorageMapEntry[]
) {
  try {
    const db = getDatabase(dbId);
    let processedEntries = entries.map((entry) => {
      return {
        root: entry.root,
        key: entry.key,
        value: entry.value,
      } as IStorageMapEntry;
    });

    await db.storageMapEntries.bulkPut(processedEntries);
  } catch (error) {
    logWebStoreError(error, `Error inserting storage map entries`);
  }
}

export async function upsertVaultAssets(dbId: string, assets: JsVaultAsset[]) {
  try {
    const db = getDatabase(dbId);
    let processedAssets = assets.map((asset) => {
      return {
        root: asset.root,
        vaultKey: asset.vaultKey,
        faucetIdPrefix: asset.faucetIdPrefix,
        asset: asset.asset,
      } as IAccountAsset;
    });

    await db.accountAssets.bulkPut(processedAssets);
  } catch (error: unknown) {
    logWebStoreError(error, `Error inserting assets`);
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

    await db.accounts.put(data as IAccount);
    await db.trackedAccounts.put({ id: accountId } as ITrackedAccount);
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
    const { encrypted, iv } = await encryptSecretKey(dbId, secretKey);
    const data = {
      pubKeyCommitmentHex,
      encryptedSecretKey: encrypted,
      iv,
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
    await db.accounts.where("id").equals(accountId).modify({ locked: true });
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
    await db.accounts
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
