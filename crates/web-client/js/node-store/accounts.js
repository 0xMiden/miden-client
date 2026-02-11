import { getDatabase } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

export async function getAccountIds(dbId) {
  try {
    const db = getDatabase(dbId);
    const rows = db.prepare("SELECT id FROM tracked_accounts").all();
    return rows.map((r) => r.id);
  } catch (error) {
    logWebStoreError(error, "Error while fetching account IDs");
  }
  return [];
}

export async function getAllAccountHeaders(dbId) {
  try {
    const db = getDatabase(dbId);
    // Get the latest record (highest nonce) per account id
    const rows = db
      .prepare(
        `SELECT * FROM accounts a1
         WHERE CAST(a1.nonce AS INTEGER) = (
           SELECT MAX(CAST(a2.nonce AS INTEGER)) FROM accounts a2 WHERE a2.id = a1.id
         )`
      )
      .all();

    return rows.map((record) => {
      let accountSeedBase64 = undefined;
      if (record.accountSeed && record.accountSeed.length > 0) {
        accountSeedBase64 = uint8ArrayToBase64(record.accountSeed);
      }
      return {
        id: record.id,
        nonce: record.nonce,
        vaultRoot: record.vaultRoot,
        storageRoot: record.storageRoot || "",
        codeRoot: record.codeRoot || "",
        accountSeed: accountSeedBase64,
        locked: !!record.locked,
        committed: !!record.committed,
        accountCommitment: record.accountCommitment || "",
      };
    });
  } catch (error) {
    logWebStoreError(error, "Error while fetching account headers");
  }
}

export async function getAccountHeader(dbId, accountId) {
  try {
    const db = getDatabase(dbId);
    const record = db
      .prepare(
        "SELECT * FROM accounts WHERE id = ? ORDER BY CAST(nonce AS INTEGER) DESC LIMIT 1"
      )
      .get(accountId);

    if (!record) {
      console.log("No account header record found for given ID.");
      return null;
    }

    let accountSeedBase64 = undefined;
    if (record.accountSeed && record.accountSeed.length > 0) {
      accountSeedBase64 = uint8ArrayToBase64(record.accountSeed);
    }

    return {
      id: record.id,
      nonce: record.nonce,
      vaultRoot: record.vaultRoot,
      storageRoot: record.storageRoot,
      codeRoot: record.codeRoot,
      accountSeed: accountSeedBase64,
      locked: !!record.locked,
    };
  } catch (error) {
    logWebStoreError(
      error,
      `Error while fetching account header for id: ${accountId}`
    );
  }
}

export async function getAccountHeaderByCommitment(dbId, accountCommitment) {
  try {
    const db = getDatabase(dbId);
    const record = db
      .prepare("SELECT * FROM accounts WHERE accountCommitment = ?")
      .get(accountCommitment);

    if (!record) {
      return undefined;
    }

    let accountSeedBase64 = undefined;
    if (record.accountSeed && record.accountSeed.length > 0) {
      accountSeedBase64 = uint8ArrayToBase64(record.accountSeed);
    }

    return {
      id: record.id,
      nonce: record.nonce,
      vaultRoot: record.vaultRoot,
      storageRoot: record.storageRoot,
      codeRoot: record.codeRoot,
      accountSeed: accountSeedBase64,
      locked: !!record.locked,
    };
  } catch (error) {
    logWebStoreError(
      error,
      `Error fetching account header for commitment ${accountCommitment}`
    );
  }
}

export async function getAccountCode(dbId, codeRoot) {
  try {
    const db = getDatabase(dbId);
    const record = db
      .prepare("SELECT * FROM account_code WHERE root = ?")
      .get(codeRoot);

    if (!record) {
      console.log("No records found for given code root.");
      return null;
    }

    return {
      root: record.root,
      code: uint8ArrayToBase64(record.code),
    };
  } catch (error) {
    logWebStoreError(error, `Error fetching account code for root ${codeRoot}`);
  }
}

export async function getAccountStorage(dbId, storageCommitment) {
  try {
    const db = getDatabase(dbId);
    const rows = db
      .prepare(
        "SELECT slotName, slotValue, slotType FROM account_storage WHERE commitment = ?"
      )
      .all(storageCommitment);

    return rows.map((record) => ({
      slotName: record.slotName,
      slotValue: record.slotValue,
      slotType: record.slotType,
    }));
  } catch (error) {
    logWebStoreError(
      error,
      `Error fetching account storage for commitment ${storageCommitment}`
    );
  }
}

export async function getAccountStorageMaps(dbId, roots) {
  try {
    const db = getDatabase(dbId);
    const rootsArr = Array.from(roots);
    if (rootsArr.length === 0) return [];
    const placeholders = rootsArr.map(() => "?").join(",");
    const rows = db
      .prepare(
        `SELECT root, key, value FROM storage_map_entries WHERE root IN (${placeholders})`
      )
      .all(...rootsArr);
    return rows;
  } catch (error) {
    logWebStoreError(
      error,
      `Error fetching account storage maps for roots ${roots.join(", ")}`
    );
  }
}

export async function getAccountVaultAssets(dbId, vaultRoot) {
  try {
    const db = getDatabase(dbId);
    const rows = db
      .prepare("SELECT asset FROM account_assets WHERE root = ?")
      .all(vaultRoot);
    return rows.map((record) => ({ asset: record.asset }));
  } catch (error) {
    logWebStoreError(
      error,
      `Error fetching account vault for root ${vaultRoot}`
    );
  }
}

export async function getAccountAuthByPubKeyCommitment(
  dbId,
  pubKeyCommitmentHex
) {
  const db = getDatabase(dbId);
  const record = db
    .prepare(
      "SELECT secretKeyHex FROM account_auth WHERE pubKeyCommitmentHex = ?"
    )
    .get(pubKeyCommitmentHex);

  if (!record) {
    throw new Error("Account auth not found in cache.");
  }

  return { secretKey: record.secretKeyHex };
}

export async function getAccountAddresses(dbId, accountId) {
  try {
    const db = getDatabase(dbId);
    const rows = db
      .prepare("SELECT id, address FROM addresses WHERE id = ?")
      .all(accountId);

    if (rows.length === 0) {
      console.log("No address records found for given account ID.");
      return [];
    }
    return rows;
  } catch (error) {
    logWebStoreError(
      error,
      `Error while fetching account addresses for id: ${accountId}`
    );
  }
}

export async function upsertAccountCode(dbId, codeRoot, code) {
  try {
    const db = getDatabase(dbId);
    db.prepare(
      "INSERT OR REPLACE INTO account_code (root, code) VALUES (?, ?)"
    ).run(codeRoot, code);
  } catch (error) {
    logWebStoreError(error, `Error inserting code with root: ${codeRoot}`);
  }
}

export async function upsertAccountStorage(dbId, storageSlots) {
  try {
    const db = getDatabase(dbId);
    const stmt = db.prepare(
      "INSERT OR REPLACE INTO account_storage (commitment, slotName, slotValue, slotType) VALUES (?, ?, ?, ?)"
    );
    const insertMany = db.transaction((slots) => {
      for (const slot of slots) {
        stmt.run(slot.commitment, slot.slotName, slot.slotValue, slot.slotType);
      }
    });
    insertMany(storageSlots);
  } catch (error) {
    logWebStoreError(error, "Error inserting storage slots");
  }
}

export async function upsertStorageMapEntries(dbId, entries) {
  try {
    const db = getDatabase(dbId);
    const stmt = db.prepare(
      "INSERT OR REPLACE INTO storage_map_entries (root, key, value) VALUES (?, ?, ?)"
    );
    const insertMany = db.transaction((items) => {
      for (const entry of items) {
        stmt.run(entry.root, entry.key, entry.value);
      }
    });
    insertMany(entries);
  } catch (error) {
    logWebStoreError(error, "Error inserting storage map entries");
  }
}

export async function upsertVaultAssets(dbId, assets) {
  try {
    const db = getDatabase(dbId);
    const stmt = db.prepare(
      "INSERT OR REPLACE INTO account_assets (root, vaultKey, faucetIdPrefix, asset) VALUES (?, ?, ?, ?)"
    );
    const insertMany = db.transaction((items) => {
      for (const asset of items) {
        stmt.run(asset.root, asset.vaultKey, asset.faucetIdPrefix, asset.asset);
      }
    });
    insertMany(assets);
  } catch (error) {
    logWebStoreError(error, "Error inserting assets");
  }
}

export async function upsertAccountRecord(
  dbId,
  accountId,
  codeRoot,
  storageRoot,
  vaultRoot,
  nonce,
  committed,
  commitment,
  accountSeed
) {
  try {
    const db = getDatabase(dbId);
    db.prepare(
      `INSERT OR REPLACE INTO accounts
       (id, codeRoot, storageRoot, vaultRoot, nonce, committed, accountCommitment, accountSeed, locked)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0)`
    ).run(
      accountId,
      codeRoot,
      storageRoot,
      vaultRoot,
      nonce,
      committed ? 1 : 0,
      commitment,
      accountSeed || null
    );
    db.prepare("INSERT OR REPLACE INTO tracked_accounts (id) VALUES (?)").run(
      accountId
    );
  } catch (error) {
    logWebStoreError(error, `Error inserting account: ${accountId}`);
  }
}

export async function insertAccountAuth(dbId, pubKeyCommitmentHex, secretKey) {
  try {
    const db = getDatabase(dbId);
    db.prepare(
      "INSERT OR REPLACE INTO account_auth (pubKeyCommitmentHex, secretKeyHex) VALUES (?, ?)"
    ).run(pubKeyCommitmentHex, secretKey);
  } catch (error) {
    logWebStoreError(
      error,
      `Error inserting account auth for pubKey: ${pubKeyCommitmentHex}`
    );
  }
}

export async function insertAccountAddress(dbId, accountId, address) {
  try {
    const db = getDatabase(dbId);
    db.prepare(
      "INSERT OR REPLACE INTO addresses (id, address) VALUES (?, ?)"
    ).run(accountId, address);
  } catch (error) {
    logWebStoreError(
      error,
      `Error inserting address for account ID ${accountId}`
    );
  }
}

export async function removeAccountAddress(dbId, address) {
  try {
    const db = getDatabase(dbId);
    db.prepare("DELETE FROM addresses WHERE address = ?").run(address);
  } catch (error) {
    logWebStoreError(
      error,
      `Error removing address with value: ${String(address)}`
    );
  }
}

export async function upsertForeignAccountCode(
  dbId,
  accountId,
  code,
  codeRoot
) {
  try {
    const db = getDatabase(dbId);
    // Reuse upsertAccountCode logic inline
    db.prepare(
      "INSERT OR REPLACE INTO account_code (root, code) VALUES (?, ?)"
    ).run(codeRoot, code);
    db.prepare(
      "INSERT OR REPLACE INTO foreign_account_code (accountId, codeRoot) VALUES (?, ?)"
    ).run(accountId, codeRoot);
  } catch (error) {
    logWebStoreError(
      error,
      `Error upserting foreign account code for account: ${accountId}`
    );
  }
}

export async function getForeignAccountCode(dbId, accountIds) {
  try {
    const db = getDatabase(dbId);
    const ids = Array.from(accountIds);
    if (ids.length === 0) return null;
    const placeholders = ids.map(() => "?").join(",");
    const rows = db
      .prepare(
        `SELECT f.accountId, c.code FROM foreign_account_code f
         JOIN account_code c ON f.codeRoot = c.root
         WHERE f.accountId IN (${placeholders})`
      )
      .all(...ids);

    if (rows.length === 0) {
      console.log("No records found for the given account IDs.");
      return null;
    }

    return rows.map((row) => ({
      accountId: row.accountId,
      code: uint8ArrayToBase64(row.code),
    }));
  } catch (error) {
    logWebStoreError(error, "Error fetching foreign account code");
  }
}

export async function lockAccount(dbId, accountId) {
  try {
    const db = getDatabase(dbId);
    db.prepare("UPDATE accounts SET locked = 1 WHERE id = ?").run(accountId);
  } catch (error) {
    logWebStoreError(error, `Error locking account: ${accountId}`);
  }
}

export async function undoAccountStates(dbId, accountCommitments) {
  try {
    const db = getDatabase(dbId);
    const commitments = Array.from(accountCommitments);
    if (commitments.length === 0) return;
    const placeholders = commitments.map(() => "?").join(",");
    db.prepare(
      `DELETE FROM accounts WHERE accountCommitment IN (${placeholders})`
    ).run(...commitments);
  } catch (error) {
    logWebStoreError(
      error,
      `Error undoing account states: ${accountCommitments.join(",")}`
    );
  }
}
