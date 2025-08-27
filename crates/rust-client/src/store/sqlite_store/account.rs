#![allow(clippy::items_after_statements)]

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::collections::BTreeMap;
use std::rc::Rc;

use miden_objects::account::{
    Account, AccountCode, AccountDelta, AccountHeader, AccountId, AccountIdPrefix, AccountStorage,
    NonFungibleDeltaAction, StorageMap, StorageSlot, StorageSlotType,
};
use miden_objects::asset::{Asset, AssetVault, FungibleAsset};
use miden_objects::{AccountError, Felt, Word};
use miden_tx::utils::{Deserializable, Serializable};
use rusqlite::types::Value;
use rusqlite::{Connection, Params, Transaction, named_params, params};

use super::{SqliteStore, column_value_as_u64, u64_to_value};
use crate::store::{AccountRecord, AccountStatus, StoreError};
use crate::{insert_sql, subst};

// TYPES
// ================================================================================================
struct SerializedHeaderData {
    id: String,
    nonce: u64,
    vault_root: String,
    storage_commitment: String,
    code_commitment: String,
    account_seed: Option<Vec<u8>>,
    locked: bool,
}

impl SqliteStore {
    // ACCOUNTS
    // --------------------------------------------------------------------------------------------

    pub(super) fn get_account_ids(conn: &mut Connection) -> Result<Vec<AccountId>, StoreError> {
        const QUERY: &str = "SELECT DISTINCT id FROM accounts";

        conn.prepare(QUERY)?
            .query_map([], |row| row.get(0))
            .expect("no binding parameters used in query")
            .map(|result| {
                Ok(result
                    .map(|id: String| AccountId::from_hex(&id).expect("account id is valid"))?)
            })
            .collect::<Result<Vec<AccountId>, StoreError>>()
    }

    pub(super) fn get_account_headers(
        conn: &mut Connection,
    ) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
        query_account_headers(
            conn,
            "nonce = (SELECT MAX(max.nonce) FROM accounts max WHERE max.id = accounts.id)",
            params![],
        )
    }

    pub(crate) fn get_account_header(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        Ok(query_account_headers(
            conn,
            "id = ? ORDER BY nonce DESC LIMIT 1",
            params![account_id.to_hex()],
        )?
        .pop())
    }

    pub(crate) fn get_account_header_by_commitment(
        conn: &mut Connection,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError> {
        let account_commitment_str: String = account_commitment.to_string();
        Ok(
            query_account_headers(conn, "account_commitment = ?", params![account_commitment_str])?
                .pop()
                .map(|(header, _)| header),
        )
    }

    pub(crate) fn get_account(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        let Some((header, status)) = Self::get_account_header(conn, account_id)? else {
            return Ok(None);
        };

        let assets = query_vault_assets(conn, "root = ?", params![header.vault_root().to_hex()])?;
        let vault = AssetVault::new(&assets)?;

        let slots = query_storage_slots(
            conn,
            "commitment = ?",
            params![header.storage_commitment().to_hex()],
        )?
        .into_values()
        .collect();

        let storage = AccountStorage::new(slots)?;

        let Some(account_code) = query_account_code(conn, header.code_commitment())? else {
            return Ok(None);
        };

        Ok(Some(AccountRecord::new(
            Account::from_parts(header.id(), vault, storage, account_code, header.nonce()),
            status,
        )))
    }

    pub(crate) fn insert_account(
        conn: &mut Connection,
        account: &Account,
        account_seed: Option<Word>,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction()?;

        Self::insert_account_code(&tx, account.code())?;
        Self::insert_storage_slots(
            &tx,
            account.storage().commitment(),
            account.storage().slots().iter().enumerate(),
        )?;
        Self::insert_assets(&tx, account.vault().root(), account.vault().assets())?;
        Self::insert_account_header(&tx, &account.into(), account_seed)?;

        Ok(tx.commit()?)
    }

    pub(crate) fn update_account(
        conn: &mut Connection,
        new_account_state: &Account,
    ) -> Result<(), StoreError> {
        const QUERY: &str = "SELECT id FROM accounts WHERE id = ?";
        if conn
            .prepare(QUERY)?
            .query_map(params![new_account_state.id().to_hex()], |row| row.get(0))?
            .map(|result| {
                result.map_err(|err| StoreError::ParsingError(err.to_string())).and_then(
                    |id: String| {
                        AccountId::from_hex(&id).map_err(|err| {
                            StoreError::AccountError(
                                AccountError::FinalAccountHeaderIdParsingFailed(err),
                            )
                        })
                    },
                )
            })
            .next()
            .is_none()
        {
            return Err(StoreError::AccountDataNotFound(new_account_state.id()));
        }

        let tx = conn.transaction()?;
        Self::update_account_state(&tx, new_account_state)?;
        Ok(tx.commit()?)
    }

    pub fn upsert_foreign_account_code(
        conn: &mut Connection,
        account_id: AccountId,
        code: &AccountCode,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction()?;

        const QUERY: &str =
            insert_sql!(foreign_account_code { account_id, code_commitment } | REPLACE);

        tx.execute(QUERY, params![account_id.to_hex(), code.commitment().to_string()])?;

        Self::insert_account_code(&tx, code)?;
        Ok(tx.commit()?)
    }

    pub fn get_foreign_account_code(
        conn: &mut Connection,
        account_ids: Vec<AccountId>,
    ) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
        let params: Vec<Value> =
            account_ids.into_iter().map(|id| Value::from(id.to_hex())).collect();
        const QUERY: &str = "
            SELECT account_id, code
            FROM foreign_account_code JOIN account_code ON foreign_account_code.code_commitment = account_code.commitment
            WHERE account_id IN rarray(?)";

        conn.prepare(QUERY)?
            .query_map([Rc::new(params)], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("no binding parameters used in query")
            .map(|result| {
                result.map_err(|err| StoreError::ParsingError(err.to_string())).and_then(
                    |(id, code): (String, Vec<u8>)| {
                        Ok((
                            AccountId::from_hex(&id).map_err(|err| {
                                StoreError::AccountError(
                                    AccountError::FinalAccountHeaderIdParsingFailed(err),
                                )
                            })?,
                            AccountCode::from_bytes(&code).map_err(StoreError::AccountError)?,
                        ))
                    },
                )
            })
            .collect::<Result<BTreeMap<AccountId, AccountCode>, _>>()
    }

    // ACCOUNT DELTA HELPERS
    // --------------------------------------------------------------------------------------------

    /// Applies the account delta to the account state, updating the vault and storage maps.
    ///
    /// The apply delta operation strats by copying over the initial account state (vault and
    /// storage) and then applying the delta on top of it. The storage and vault elements are
    /// overwritten in the new state. In the cases where the delta depends on previous state (e.g.
    /// adding or subtracting fungible assets), the previous state needs to be provided via the
    /// `updated_fungible_assets` and `updated_storage_maps` parameters.
    pub(super) fn apply_account_delta(
        tx: &Transaction<'_>,
        init_account_state: &AccountHeader,
        final_account_state: &AccountHeader,
        mut updated_fungible_assets: BTreeMap<AccountIdPrefix, FungibleAsset>,
        mut updated_storage_maps: BTreeMap<u8, StorageMap>,
        delta: &AccountDelta,
    ) -> Result<(), StoreError> {
        // Copy over the storage and vault from the previous state. Non-relevant data will not be
        // modified.
        Self::copy_account_state(tx, init_account_state, final_account_state)?;

        // Apply vault delta. This map will contain all updated assets, both fungible and
        // non-fungible.
        let mut updated_assets: BTreeMap<AccountIdPrefix, Asset> = BTreeMap::new();

        // We first process the fungible assets. Adding or subtracting them from the vault as
        // requested.
        for (faucet_id, delta) in delta.vault().fungible().iter() {
            let delta_asset = FungibleAsset::new(*faucet_id, delta.unsigned_abs())?;

            match updated_fungible_assets.remove(&faucet_id.prefix()) {
                Some(asset) => {
                    // If the asset exists, update it accordingly.
                    if *delta >= 0 {
                        updated_assets
                            .insert(faucet_id.prefix(), Asset::Fungible(asset.add(delta_asset)?));
                    } else {
                        updated_assets
                            .insert(faucet_id.prefix(), Asset::Fungible(asset.sub(delta_asset)?));
                    }
                },
                None => {
                    // If the asset doesn't exist, we add it to the map to be inserted.
                    if *delta > 0 {
                        updated_assets.insert(faucet_id.prefix(), Asset::Fungible(delta_asset));
                    }
                },
            }
        }

        // Process non-fungible assets. Here additions or removals don't depend on previous state as
        // each asset is unique.
        let (added_nonfungible_assets, removed_nonfungible_assets) =
            delta.vault().non_fungible().iter().partition::<Vec<_>, _>(|(_, action)| {
                matches!(action, NonFungibleDeltaAction::Add)
            });

        updated_assets.extend(
            added_nonfungible_assets
                .into_iter()
                .map(|(asset, _)| (asset.faucet_id_prefix(), Asset::NonFungible(*asset))),
        );

        const DELETE_QUERY: &str =
            "DELETE FROM account_vaults WHERE root = ? AND faucet_id_prefix IN rarray(?)";

        tx.execute(
            DELETE_QUERY,
            params![
                final_account_state.vault_root().to_hex(),
                Rc::new(
                    removed_nonfungible_assets
                        .iter()
                        .map(|(asset, _)| Value::Text(asset.faucet_id_prefix().to_hex()))
                        .collect::<Vec<Value>>(),
                ),
            ],
        )?;

        Self::insert_assets(tx, final_account_state.vault_root(), updated_assets.into_values())?;

        // Apply storage delta. This map will contain all updated storage slots, both values and
        // maps. It gets initialized with value type updates which contain the new value and
        // don't depend on previous state.
        let mut updated_storage_slots: BTreeMap<u8, StorageSlot> = delta
            .storage()
            .values()
            .iter()
            .map(|(index, slot)| (*index, StorageSlot::Value(*slot)))
            .collect();

        // For storage map deltas, we only updated the keys in the delta, this is why we need the
        // previously retrieved storage maps.
        for (index, map_delta) in delta.storage().maps() {
            let mut map = updated_storage_maps.remove(index).unwrap_or_default();

            for (key, value) in map_delta.entries() {
                map.insert((*key).into(), *value);
            }

            updated_storage_slots.insert(*index, StorageSlot::Map(map));
        }

        Self::insert_storage_slots(
            tx,
            final_account_state.storage_commitment(),
            updated_storage_slots.iter().map(|(index, slot)| (*index as usize, slot)),
        )?;

        Ok(())
    }

    /// Fetches the relevant fungible assets of an account that will be updated by the account
    /// delta.
    pub(super) fn get_account_fungible_assets_for_delta(
        conn: &Connection,
        header: &AccountHeader,
        delta: &AccountDelta,
    ) -> Result<BTreeMap<AccountIdPrefix, FungibleAsset>, StoreError> {
        let fungible_faucet_prefixes = delta
            .vault()
            .fungible()
            .iter()
            .map(|(faucet_id, _)| Value::Text(faucet_id.prefix().to_hex()))
            .collect::<Vec<Value>>();

        Ok(query_vault_assets(
                    conn,
                    "root = ? AND faucet_id_prefix IN rarray(?)",
                    params![header.vault_root().to_hex(), Rc::new(fungible_faucet_prefixes)]
                )?
                .into_iter()
                // SAFETY: all retrieved assets should be fungible
                .map(|asset| (asset.faucet_id_prefix(), asset.unwrap_fungible()))
                .collect())
    }

    /// Fetches the relevant storage maps inside the account's storage that will be updated by the
    /// account delta.
    pub(super) fn get_account_storage_maps_for_delta(
        conn: &Connection,
        header: &AccountHeader,
        delta: &AccountDelta,
    ) -> Result<BTreeMap<u8, StorageMap>, StoreError> {
        let updated_map_indexes = delta
            .storage()
            .maps()
            .keys()
            .map(|k| Value::Integer(i64::from(*k)))
            .collect::<Vec<Value>>();

        query_storage_slots(
            conn,
            "commitment = ? AND slot_index IN rarray(?)",
            params![header.storage_commitment().to_hex(), Rc::new(updated_map_indexes)],
        )?
        .into_iter()
        .map(|(index, slot)| {
            let StorageSlot::Map(map) = slot else {
                return Err(StoreError::AccountError(AccountError::StorageSlotNotMap(index)));
            };

            Ok((index, map))
        })
        .collect()
    }

    /// Inserts the new `final_account_header` to the store and copies over the previous account
    /// state (vault and storage). This isn't meant to be the whole account update, just the first
    /// step. The account delta should then be applied to the copied data.
    fn copy_account_state(
        tx: &Transaction<'_>,
        init_account_header: &AccountHeader,
        final_account_header: &AccountHeader,
    ) -> Result<(), StoreError> {
        Self::insert_account_header(tx, final_account_header, None)?;

        if init_account_header.vault_root() != final_account_header.vault_root() {
            const VAULT_QUERY: &str = "
                INSERT OR IGNORE INTO account_vaults (
                    root,
                    faucet_id_prefix,
                    asset
                )
                SELECT
                    ?, --new root
                    faucet_id_prefix,
                    asset
                FROM account_vaults
                WHERE root = (SELECT vault_root FROM accounts WHERE account_commitment = ?)
                ";
            tx.execute(
                VAULT_QUERY,
                params![
                    final_account_header.vault_root().to_hex(),
                    init_account_header.commitment().to_hex()
                ],
            )?;
        }

        if init_account_header.storage_commitment() != final_account_header.storage_commitment() {
            const STORAGE_QUERY: &str = "
                INSERT OR IGNORE INTO account_storage (
                    commitment,
                    slot_index,
                    slot_value,
                    slot_type
                )
                SELECT
                    ?, -- new commitment
                    slot_index,
                    slot_value,
                    slot_type
                FROM account_storage
                WHERE commitment = (SELECT storage_commitment FROM accounts WHERE account_commitment = ?)
                ";

            tx.execute(
                STORAGE_QUERY,
                params![
                    final_account_header.storage_commitment().to_hex(),
                    init_account_header.commitment().to_hex()
                ],
            )?;
        }

        Ok(())
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

    /// Update previously-existing account after a transaction execution.
    ///
    /// Because the Client retrieves the account by account ID before applying the delta, we don't
    /// need to check that it exists here. This inserts a new row into the accounts table.
    /// We can later identify the proper account state by looking at the nonce.
    pub(super) fn update_account_state(
        tx: &Transaction<'_>,
        new_account_state: &Account,
    ) -> Result<(), StoreError> {
        Self::insert_storage_slots(
            tx,
            new_account_state.storage().commitment(),
            new_account_state.storage().slots().iter().enumerate(),
        )?;
        Self::insert_assets(
            tx,
            new_account_state.vault().root(),
            new_account_state.vault().assets(),
        )?;
        Self::insert_account_header(tx, &new_account_state.into(), None)
    }

    /// Locks the account if the mismatched digest doesn't belong to a previous account state (stale
    /// data).
    pub(super) fn lock_account_on_unexpected_commitment(
        tx: &Transaction<'_>,
        account_id: &AccountId,
        mismatched_digest: &Word,
    ) -> Result<(), StoreError> {
        // Mismatched digests may be due to stale network data. If the mismatched digest is
        // tracked in the db and corresponds to the mismatched account, it means we
        // got a past update and shouldn't lock the account.
        const QUERY: &str = "UPDATE accounts SET locked = true WHERE id = :account_id AND NOT EXISTS (SELECT 1 FROM accounts WHERE id = :account_id AND account_commitment = :digest)";
        tx.execute(
            QUERY,
            named_params! {
                ":account_id": account_id.to_hex(),
                ":digest": mismatched_digest.to_string()
            },
        )?;
        Ok(())
    }

    /// Removes account states with the specified hashes from the database.
    ///
    /// This is used to rollback account changes when a transaction is discarded,
    /// effectively undoing the account state changes that were applied by the transaction.
    ///
    /// Note: This is not part of the Store trait and is only used internally by the `SQLite` store
    /// implementation to handle transaction rollbacks.
    pub(super) fn undo_account_state(
        tx: &Transaction<'_>,
        account_hashes: &[Word],
    ) -> Result<(), StoreError> {
        const QUERY: &str = "DELETE FROM accounts WHERE account_commitment IN rarray(?)";

        let params = account_hashes.iter().map(|h| Value::from(h.to_hex())).collect::<Vec<_>>();
        tx.execute(QUERY, params![Rc::new(params)])?;

        Ok(())
    }

    /// Inserts a new account record into the database.
    fn insert_account_header(
        tx: &Transaction<'_>,
        account: &AccountHeader,
        account_seed: Option<Word>,
    ) -> Result<(), StoreError> {
        let id: String = account.id().to_hex();
        let code_commitment = account.code_commitment().to_string();
        let storage_commitment = account.storage_commitment().to_string();
        let vault_root = account.vault_root().to_string();
        let nonce = u64_to_value(account.nonce().as_int());
        let commitment = account.commitment().to_string();

        let account_seed = account_seed.map(|seed| seed.to_bytes());

        const QUERY: &str = insert_sql!(
            accounts {
                id,
                code_commitment,
                storage_commitment,
                vault_root,
                nonce,
                account_seed,
                account_commitment,
                locked
            } | REPLACE
        );

        tx.execute(
            QUERY,
            params![
                id,
                code_commitment,
                storage_commitment,
                vault_root,
                nonce,
                account_seed,
                commitment,
                false,
            ],
        )?;
        Ok(())
    }

    /// Inserts an [`AccountCode`].
    fn insert_account_code(
        tx: &Transaction<'_>,
        account_code: &AccountCode,
    ) -> Result<(), StoreError> {
        const QUERY: &str = insert_sql!(account_code { commitment, code } | IGNORE);
        tx.execute(QUERY, params![account_code.commitment().to_hex(), account_code.to_bytes()])?;
        Ok(())
    }

    fn insert_storage_slots<'a>(
        tx: &Transaction<'_>,
        commitment: Word,
        account_storage: impl Iterator<Item = (usize, &'a StorageSlot)>,
    ) -> Result<(), StoreError> {
        for (index, slot) in account_storage {
            const QUERY: &str = insert_sql!(
                account_storage {
                    commitment,
                    slot_index,
                    slot_value,
                    slot_type
                } | REPLACE
            );

            tx.execute(
                QUERY,
                params![
                    commitment.to_hex(),
                    index,
                    slot.value().to_hex(),
                    slot.slot_type().to_bytes()
                ],
            )?;

            if let StorageSlot::Map(map) = slot {
                const MAP_QUERY: &str =
                    insert_sql!(storage_map_entries { root, key, value } | REPLACE);
                for (key, value) in map.entries() {
                    // Insert each entry of the storage map
                    tx.execute(
                        MAP_QUERY,
                        params![map.root().to_hex(), key.to_hex(), value.to_hex()],
                    )?;
                }
            }
        }

        Ok(())
    }

    fn insert_assets(
        tx: &Transaction<'_>,
        root: Word,
        assets: impl Iterator<Item = Asset>,
    ) -> Result<(), StoreError> {
        for asset in assets {
            const QUERY: &str =
                insert_sql!(account_vaults { root, faucet_id_prefix, asset } | REPLACE);
            tx.execute(
                QUERY,
                params![
                    root.to_hex(),
                    asset.faucet_id_prefix().to_hex(),
                    Word::from(asset).to_hex(),
                ],
            )?;
        }

        Ok(())
    }
}

// HELPERS
// ================================================================================================

/// Parse an account header from the provided serialized data.
fn parse_accounts(
    serialized_account_parts: SerializedHeaderData,
) -> Result<(AccountHeader, AccountStatus), StoreError> {
    let SerializedHeaderData {
        id,
        nonce,
        vault_root,
        storage_commitment,
        code_commitment,
        account_seed,
        locked,
    } = serialized_account_parts;
    let account_seed = account_seed.map(|seed| Word::read_from_bytes(&seed)).transpose()?;

    let status = match (account_seed, locked) {
        (_, true) => AccountStatus::Locked,
        (Some(seed), _) => AccountStatus::New { seed },
        _ => AccountStatus::Tracked,
    };

    Ok((
        AccountHeader::new(
            AccountId::from_hex(&id).expect("Conversion from stored AccountID should not panic"),
            Felt::new(nonce),
            Word::try_from(&vault_root)?,
            Word::try_from(&storage_commitment)?,
            Word::try_from(&code_commitment)?,
        ),
        status,
    ))
}

fn query_storage_slots(
    conn: &Connection,
    where_clause: &str,
    params: impl Params,
) -> Result<BTreeMap<u8, StorageSlot>, StoreError> {
    const STORAGE_QUERY: &str = "SELECT slot_index, slot_value, slot_type FROM account_storage";

    let query = format!("{STORAGE_QUERY} WHERE {where_clause}");
    let storage_values = conn
        .prepare(&query)?
        .query_map(params, |row| {
            let index: u8 = row.get(0)?;
            let value: String = row.get(1)?;
            let slot_type: Vec<u8> = row.get(2)?;
            Ok((index, value, slot_type))
        })?
        .map(|result| {
            let (index, value, slot_type) = result?;
            Ok((index, Word::try_from(value)?, StorageSlotType::read_from_bytes(&slot_type)?))
        })
        .collect::<Result<Vec<(u8, Word, StorageSlotType)>, StoreError>>()?;

    let possible_roots: Vec<Value> =
        storage_values.iter().map(|(_, value, _)| Value::from(value.to_hex())).collect();

    let mut storage_maps =
        query_storage_maps(conn, "root IN rarray(?)", [Rc::new(possible_roots)])?;

    Ok(storage_values
        .into_iter()
        .map(|(index, value, slot_type)| {
            let slot = match slot_type {
                StorageSlotType::Value => StorageSlot::Value(value),
                StorageSlotType::Map => {
                    StorageSlot::Map(storage_maps.remove(&value).unwrap_or_default())
                },
            };
            (index, slot)
        })
        .collect())
}

fn query_storage_maps(
    conn: &Connection,
    where_clause: &str,
    params: impl Params,
) -> Result<BTreeMap<Word, StorageMap>, StoreError> {
    const STORAGE_MAP_SELECT: &str = "SELECT root, key, value FROM storage_map_entries";
    let query = format!("{STORAGE_MAP_SELECT} WHERE {where_clause}");

    let map_entries = conn
        .prepare(&query)?
        .query_map(params, |row| {
            let root: String = row.get(0)?;
            let key: String = row.get(1)?;
            let value: String = row.get(2)?;

            Ok((root, key, value))
        })?
        .map(|result| {
            let (root, key, value) = result?;
            Ok((Word::try_from(root)?, Word::try_from(key)?, Word::try_from(value)?))
        })
        .collect::<Result<Vec<(Word, Word, Word)>, StoreError>>()?;

    let mut maps = BTreeMap::new();
    for (root, key, value) in map_entries {
        let map = maps.entry(root).or_insert_with(StorageMap::new);
        map.insert(key, value);
    }

    Ok(maps)
}

fn query_vault_assets(
    conn: &Connection,
    where_clause: &str,
    params: impl Params,
) -> Result<Vec<Asset>, StoreError> {
    const VAULT_QUERY: &str = "SELECT asset FROM account_vaults";

    let query = format!("{VAULT_QUERY} WHERE {where_clause}");
    conn.prepare(&query)?
        .query_map(params, |row| {
            let asset: String = row.get(0)?;
            Ok(asset)
        })?
        .map(|result| {
            let word = Word::try_from(result?)?;
            Ok(Asset::try_from(word)?)
        })
        .collect::<Result<Vec<Asset>, StoreError>>()
}

fn query_account_code(
    conn: &Connection,
    commitment: Word,
) -> Result<Option<AccountCode>, StoreError> {
    // TODO: this function will probably be refactored to receive more complex where clauses and
    // return multiple mast forests
    const CODE_QUERY: &str = "SELECT code FROM account_code WHERE commitment = ?";

    conn.prepare(CODE_QUERY)?
        .query_map(params![commitment.to_hex()], |row| {
            let code: Vec<u8> = row.get(0)?;
            Ok(code)
        })?
        .map(|result| {
            let bytes = result?;
            Ok(AccountCode::from_bytes(&bytes)?)
        })
        .next()
        .transpose()
}

fn query_account_headers(
    conn: &Connection,
    where_clause: &str,
    params: impl Params,
) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
    const SELECT_QUERY: &str = "SELECT id, nonce, vault_root, storage_commitment, code_commitment, account_seed, locked \
        FROM accounts";
    let query = format!("{SELECT_QUERY} WHERE {where_clause}");
    conn.prepare(&query)?
        .query_map(params, |row| {
            let id: String = row.get(0)?;
            let nonce: u64 = column_value_as_u64(row, 1)?;
            let vault_root: String = row.get(2)?;
            let storage_commitment: String = row.get(3)?;
            let code_commitment: String = row.get(4)?;
            let account_seed: Option<Vec<u8>> = row.get(5)?;
            let locked: bool = row.get(6)?;

            Ok(SerializedHeaderData {
                id,
                nonce,
                vault_root,
                storage_commitment,
                code_commitment,
                account_seed,
                locked,
            })
        })?
        .map(|result| parse_accounts(result?))
        .collect::<Result<Vec<(AccountHeader, AccountStatus)>, StoreError>>()
}

#[cfg(test)]
mod tests {
    use miden_lib::account::auth::AuthRpoFalcon512;
    use miden_objects::EMPTY_WORD;
    use miden_objects::account::{AccountCode, AccountComponent};
    use miden_objects::crypto::dsa::rpo_falcon512::PublicKey;

    use crate::store::sqlite_store::SqliteStore;
    use crate::store::sqlite_store::tests::create_test_store;

    #[tokio::test]
    async fn account_code_insertion_no_duplicates() {
        let store = create_test_store().await;
        let assembler = miden_lib::transaction::TransactionKernel::assembler();
        let account_component = AccountComponent::compile(
            "
                export.::miden::contracts::wallets::basic::receive_asset
                export.::miden::contracts::wallets::basic::move_asset_to_note
            ",
            assembler,
            vec![],
        )
        .unwrap()
        .with_supports_all_types();
        let account_code = AccountCode::from_components(
            &[AuthRpoFalcon512::new(PublicKey::new(EMPTY_WORD)).into(), account_component],
            miden_objects::account::AccountType::RegularAccountUpdatableCode,
        )
        .unwrap();
        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().unwrap();

                // Table is empty at the beginning
                let mut actual: usize = tx
                    .query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0))
                    .unwrap();
                assert_eq!(actual, 0);

                // First insertion generates a new row
                SqliteStore::insert_account_code(&tx, &account_code).unwrap();
                actual = tx
                    .query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0))
                    .unwrap();
                assert_eq!(actual, 1);

                // Second insertion passes but does not generate a new row
                assert!(SqliteStore::insert_account_code(&tx, &account_code).is_ok());
                actual = tx
                    .query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0))
                    .unwrap();
                assert_eq!(actual, 1);

                Ok(())
            })
            .await
            .unwrap();
    }
}
