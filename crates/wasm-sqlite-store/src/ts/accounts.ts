/**
 * Account operations for the WASM SQLite store.
 *
 * SQL queries match the native sqlite-store's schema and query patterns.
 */

import { getDatabase } from "./schema.js";
import { logError, uint8ArrayToBase64 } from "./utils.js";

// Interfaces for data returned to Rust
export interface AccountHeaderRow {
  id: string;
  nonce: string;
  vaultRoot: string;
  storageCommitment: string;
  codeCommitment: string;
  accountSeed: string | null;
  locked: boolean;
  accountCommitment: string;
}

export interface AccountCodeRow {
  commitment: string;
  code: string; // base64
}

export interface AccountStorageRow {
  commitment: string;
  slotName: string;
  slotValue: string;
  slotType: number;
}

export interface StorageMapEntryRow {
  root: string;
  key: string;
  value: string;
}

export interface AccountAssetRow {
  root: string;
  vaultKey: string;
  faucetIdPrefix: string;
  asset: string;
}

export interface AddressRow {
  address: string; // base64
  id: string;
}

export interface ForeignAccountCodeRow {
  accountId: string;
  code: string; // base64
}

export function getAccountIds(dbId: string): string[] {
  try {
    const db = getDatabase(dbId);
    const rows = db.all<{ id: string }>("SELECT id FROM tracked_accounts");
    return rows.map((row) => row.id);
  } catch (error) {
    logError(error, "Error while fetching account IDs");
    return [];
  }
}

export function getAllAccountHeaders(dbId: string): AccountHeaderRow[] {
  try {
    const db = getDatabase(dbId);
    const rows = db.all<{
      id: number;
      nonce: number;
      vault_root: string;
      storage_commitment: string;
      code_commitment: string;
      account_seed: Uint8Array | null;
      locked: number;
      account_commitment: string;
    }>(
      `SELECT
        a.id,
        a.nonce,
        a.vault_root,
        a.storage_commitment,
        a.code_commitment,
        a.account_seed,
        a.locked,
        a.account_commitment
      FROM accounts AS a
      JOIN (
        SELECT id, MAX(nonce) AS nonce
        FROM accounts
        GROUP BY id
      ) AS latest
      ON a.id = latest.id
      AND a.nonce = latest.nonce
      ORDER BY a.id`
    );

    return rows.map((row) => ({
      id: row.id.toString(),
      nonce: row.nonce.toString(),
      vaultRoot: row.vault_root,
      storageCommitment: row.storage_commitment,
      codeCommitment: row.code_commitment,
      accountSeed: row.account_seed
        ? uint8ArrayToBase64(
            row.account_seed instanceof Uint8Array
              ? row.account_seed
              : new Uint8Array(row.account_seed as ArrayBuffer)
          )
        : null,
      locked: row.locked === 1,
      accountCommitment: row.account_commitment,
    }));
  } catch (error) {
    logError(error, "Error while fetching account headers");
    return [];
  }
}

export function getAccountHeader(
  dbId: string,
  accountId: string
): AccountHeaderRow | null {
  try {
    const db = getDatabase(dbId);
    const row = db.get<{
      id: number;
      nonce: number;
      vault_root: string;
      storage_commitment: string;
      code_commitment: string;
      account_seed: Uint8Array | null;
      locked: number;
      account_commitment: string;
    }>(
      `SELECT
        a.id,
        a.nonce,
        a.vault_root,
        a.storage_commitment,
        a.code_commitment,
        a.account_seed,
        a.locked,
        a.account_commitment
      FROM accounts AS a
      JOIN (
        SELECT id, MAX(nonce) AS nonce
        FROM accounts
        WHERE id = ?
        GROUP BY id
      ) AS latest
      ON a.id = latest.id
      AND a.nonce = latest.nonce`,
      [accountId]
    );

    if (!row) return null;

    return {
      id: row.id.toString(),
      nonce: row.nonce.toString(),
      vaultRoot: row.vault_root,
      storageCommitment: row.storage_commitment,
      codeCommitment: row.code_commitment,
      accountSeed: row.account_seed
        ? uint8ArrayToBase64(
            row.account_seed instanceof Uint8Array
              ? row.account_seed
              : new Uint8Array(row.account_seed as ArrayBuffer)
          )
        : null,
      locked: row.locked === 1,
      accountCommitment: row.account_commitment,
    };
  } catch (error) {
    logError(error, `Error while fetching account header: ${accountId}`);
    return null;
  }
}

export function getAccountHeaderByCommitment(
  dbId: string,
  accountCommitment: string
): AccountHeaderRow | null {
  try {
    const db = getDatabase(dbId);
    const row = db.get<{
      id: number;
      nonce: number;
      vault_root: string;
      storage_commitment: string;
      code_commitment: string;
      account_seed: Uint8Array | null;
      locked: number;
      account_commitment: string;
    }>(
      `SELECT id, nonce, vault_root, storage_commitment, code_commitment,
              account_seed, locked, account_commitment
       FROM accounts WHERE account_commitment = ?`,
      [accountCommitment]
    );

    if (!row) return null;

    return {
      id: row.id.toString(),
      nonce: row.nonce.toString(),
      vaultRoot: row.vault_root,
      storageCommitment: row.storage_commitment,
      codeCommitment: row.code_commitment,
      accountSeed: row.account_seed
        ? uint8ArrayToBase64(
            row.account_seed instanceof Uint8Array
              ? row.account_seed
              : new Uint8Array(row.account_seed as ArrayBuffer)
          )
        : null,
      locked: row.locked === 1,
      accountCommitment: row.account_commitment,
    };
  } catch (error) {
    logError(error, `Error while fetching account by commitment`);
    return null;
  }
}

export function getAccountCode(
  dbId: string,
  commitment: string
): AccountCodeRow | null {
  try {
    const db = getDatabase(dbId);
    const row = db.get<{ commitment: string; code: Uint8Array }>(
      "SELECT commitment, code FROM account_code WHERE commitment = ?",
      [commitment]
    );

    if (!row) return null;

    const codeBytes =
      row.code instanceof Uint8Array
        ? row.code
        : new Uint8Array(row.code as ArrayBuffer);

    return {
      commitment: row.commitment,
      code: uint8ArrayToBase64(codeBytes),
    };
  } catch (error) {
    logError(error, `Error while fetching account code: ${commitment}`);
    return null;
  }
}

export function getAccountStorage(
  dbId: string,
  storageCommitment: string
): AccountStorageRow[] {
  try {
    const db = getDatabase(dbId);
    const rows = db.all<{
      commitment: string;
      slot_name: string;
      slot_value: string;
      slot_type: number;
    }>(
      "SELECT commitment, slot_name, slot_value, slot_type FROM account_storage WHERE commitment = ?",
      [storageCommitment]
    );

    return rows.map((row) => ({
      commitment: row.commitment,
      slotName: row.slot_name,
      slotValue: row.slot_value,
      slotType: row.slot_type,
    }));
  } catch (error) {
    logError(error, `Error while fetching account storage`);
    return [];
  }
}

export function getAccountStorageMaps(
  dbId: string,
  roots: string[]
): StorageMapEntryRow[] {
  try {
    if (roots.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = roots.map(() => "?").join(",");
    const rows = db.all<{ root: string; key: string; value: string }>(
      `SELECT root, key, value FROM storage_map_entries WHERE root IN (${placeholders})`,
      roots
    );
    return rows;
  } catch (error) {
    logError(error, `Error while fetching storage map entries`);
    return [];
  }
}

export function getAccountVaultAssets(
  dbId: string,
  vaultRoot: string
): AccountAssetRow[] {
  try {
    const db = getDatabase(dbId);
    const rows = db.all<{
      root: string;
      vault_key: string;
      faucet_id_prefix: string;
      asset: string;
    }>(
      "SELECT root, vault_key, faucet_id_prefix, asset FROM account_assets WHERE root = ?",
      [vaultRoot]
    );

    return rows.map((row) => ({
      root: row.root,
      vaultKey: row.vault_key,
      faucetIdPrefix: row.faucet_id_prefix,
      asset: row.asset,
    }));
  } catch (error) {
    logError(error, `Error while fetching vault assets`);
    return [];
  }
}

export function getAccountAddresses(
  dbId: string,
  accountId: string
): AddressRow[] {
  try {
    const db = getDatabase(dbId);
    const rows = db.all<{ address: Uint8Array; account_id: number }>(
      "SELECT address, account_id FROM addresses WHERE account_id = ?",
      [accountId]
    );

    return rows.map((row) => {
      const addrBytes =
        row.address instanceof Uint8Array
          ? row.address
          : new Uint8Array(row.address as ArrayBuffer);
      return {
        address: uint8ArrayToBase64(addrBytes),
        id: row.account_id.toString(),
      };
    });
  } catch (error) {
    logError(error, `Error while fetching account addresses`);
    return [];
  }
}

export function upsertAccountCode(
  dbId: string,
  codeCommitment: string,
  code: Uint8Array
): void {
  try {
    const db = getDatabase(dbId);
    db.run(
      "INSERT OR REPLACE INTO account_code (commitment, code) VALUES (?, ?)",
      [codeCommitment, code]
    );
  } catch (error) {
    logError(error, `Error inserting account code`);
  }
}

export function upsertAccountStorage(
  dbId: string,
  slots: Array<{
    commitment: string;
    slotName: string;
    slotValue: string;
    slotType: number;
  }>
): void {
  try {
    const db = getDatabase(dbId);
    db.transaction(() => {
      for (const slot of slots) {
        db.run(
          "INSERT OR REPLACE INTO account_storage (commitment, slot_name, slot_value, slot_type) VALUES (?, ?, ?, ?)",
          [slot.commitment, slot.slotName, slot.slotValue, slot.slotType]
        );
      }
    });
  } catch (error) {
    logError(error, `Error inserting account storage`);
  }
}

export function upsertStorageMapEntries(
  dbId: string,
  entries: Array<{ root: string; key: string; value: string }>
): void {
  try {
    const db = getDatabase(dbId);
    db.transaction(() => {
      for (const entry of entries) {
        db.run(
          "INSERT OR REPLACE INTO storage_map_entries (root, key, value) VALUES (?, ?, ?)",
          [entry.root, entry.key, entry.value]
        );
      }
    });
  } catch (error) {
    logError(error, `Error inserting storage map entries`);
  }
}

export function upsertVaultAssets(
  dbId: string,
  assets: Array<{
    root: string;
    vaultKey: string;
    faucetIdPrefix: string;
    asset: string;
  }>
): void {
  try {
    const db = getDatabase(dbId);
    db.transaction(() => {
      for (const asset of assets) {
        db.run(
          "INSERT OR REPLACE INTO account_assets (root, vault_key, faucet_id_prefix, asset) VALUES (?, ?, ?, ?)",
          [asset.root, asset.vaultKey, asset.faucetIdPrefix, asset.asset]
        );
      }
    });
  } catch (error) {
    logError(error, `Error inserting vault assets`);
  }
}

export function upsertAccountRecord(
  dbId: string,
  accountId: string,
  codeCommitment: string,
  storageCommitment: string,
  vaultRoot: string,
  nonce: string,
  committed: boolean,
  commitment: string,
  accountSeed: Uint8Array | null
): void {
  try {
    const db = getDatabase(dbId);
    db.transaction(() => {
      db.run(
        `INSERT OR REPLACE INTO accounts
         (id, account_commitment, code_commitment, storage_commitment, vault_root, nonce, account_seed, locked)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
        [
          accountId,
          commitment,
          codeCommitment,
          storageCommitment,
          vaultRoot,
          nonce,
          accountSeed,
          0, // not locked
        ]
      );
      db.run("INSERT OR REPLACE INTO tracked_accounts (id) VALUES (?)", [
        accountId,
      ]);
    });
  } catch (error) {
    logError(error, `Error inserting account: ${accountId}`);
  }
}

export function insertAccountAddress(
  dbId: string,
  accountId: string,
  address: Uint8Array
): void {
  try {
    const db = getDatabase(dbId);
    db.run(
      "INSERT OR REPLACE INTO addresses (address, account_id) VALUES (?, ?)",
      [address, accountId]
    );
  } catch (error) {
    logError(error, `Error inserting account address`);
  }
}

export function removeAccountAddress(dbId: string, address: Uint8Array): void {
  try {
    const db = getDatabase(dbId);
    db.run("DELETE FROM addresses WHERE address = ?", [address]);
  } catch (error) {
    logError(error, `Error removing account address`);
  }
}

export function upsertForeignAccountCode(
  dbId: string,
  accountId: string,
  code: Uint8Array,
  codeCommitment: string
): void {
  try {
    const db = getDatabase(dbId);
    db.transaction(() => {
      // Upsert the account code
      db.run(
        "INSERT OR REPLACE INTO account_code (commitment, code) VALUES (?, ?)",
        [codeCommitment, code]
      );
      // Upsert the foreign account reference
      db.run(
        "INSERT OR REPLACE INTO foreign_account_code (account_id, code_commitment) VALUES (?, ?)",
        [accountId, codeCommitment]
      );
    });
  } catch (error) {
    logError(error, `Error upserting foreign account code`);
  }
}

export function getForeignAccountCode(
  dbId: string,
  accountIds: string[]
): ForeignAccountCodeRow[] {
  try {
    if (accountIds.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = accountIds.map(() => "?").join(",");
    const rows = db.all<{
      account_id: string;
      code: Uint8Array;
    }>(
      `SELECT f.account_id, c.code
       FROM foreign_account_code AS f
       JOIN account_code AS c ON f.code_commitment = c.commitment
       WHERE f.account_id IN (${placeholders})`,
      accountIds
    );

    return rows.map((row) => {
      const codeBytes =
        row.code instanceof Uint8Array
          ? row.code
          : new Uint8Array(row.code as ArrayBuffer);
      return {
        accountId: row.account_id,
        code: uint8ArrayToBase64(codeBytes),
      };
    });
  } catch (error) {
    logError(error, `Error fetching foreign account code`);
    return [];
  }
}

export function lockAccount(dbId: string, accountId: string): void {
  try {
    const db = getDatabase(dbId);
    db.run("UPDATE accounts SET locked = 1 WHERE id = ?", [accountId]);
  } catch (error) {
    logError(error, `Error locking account: ${accountId}`);
  }
}

export function undoAccountStates(
  dbId: string,
  accountCommitments: string[]
): void {
  try {
    if (accountCommitments.length === 0) return;
    const db = getDatabase(dbId);
    const placeholders = accountCommitments.map(() => "?").join(",");
    db.run(
      `DELETE FROM accounts WHERE account_commitment IN (${placeholders})`,
      accountCommitments
    );
  } catch (error) {
    logError(error, `Error undoing account states`);
  }
}
