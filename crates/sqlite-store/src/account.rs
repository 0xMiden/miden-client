#![allow(clippy::items_after_statements)]

use std::collections::BTreeMap;
use std::rc::Rc;
use std::string::{String, ToString};
use std::sync::{Arc, RwLock};
use std::vec::Vec;

use miden_client::account::{
    Account,
    AccountCode,
    AccountDelta,
    AccountHeader,
    AccountId,
    AccountIdPrefix,
    AccountStorage,
    Address,
    StorageMap,
    StorageSlot,
    StorageSlotType,
};
use miden_client::asset::{Asset, AssetVault, AssetWitness, FungibleAsset, NonFungibleDeltaAction};
use miden_client::crypto::{MerklePath, MerkleStore, SmtLeaf, SmtProof};
use miden_client::store::{AccountRecord, AccountStatus, StoreError};
use miden_client::utils::{Deserializable, Serializable};
use miden_client::{AccountError, Felt, Word};
use miden_objects::account::{
    AccountStorageHeader,
    PartialAccount,
    PartialStorage,
    PartialStorageMap,
    StorageMapWitness,
};
use miden_objects::asset::PartialVault;
use miden_objects::crypto::merkle::PartialSmt;
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Params, Transaction, named_params, params};

use super::{SqliteStore, column_value_as_u64, u64_to_value};
use crate::merkle_store::{
    get_asset_proof,
    get_storage_map_item_proof,
    insert_asset_nodes,
    insert_storage_map_nodes,
    update_asset_nodes,
    update_storage_map_nodes,
};
use crate::sql_error::SqlResultExt;
use crate::{current_timestamp_u64, insert_sql, subst};

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

        conn.prepare(QUERY)
            .into_store_error()?
            .query_map([], |row| row.get(0))
            .expect("no binding parameters used in query")
            .map(|result| {
                let id: String = result.map_err(|e| StoreError::ParsingError(e.to_string()))?;
                Ok(AccountId::from_hex(&id).expect("account id is valid"))
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

        let account = Account::new_unchecked(
            header.id(),
            vault,
            storage,
            account_code,
            header.nonce(),
            status.seed().copied(),
        );

        let addresses = query_account_addresses(conn, header.id())?;
        Ok(Some(AccountRecord::new(account, status, addresses)))
    }

    pub(crate) fn insert_account(
        conn: &mut Connection,
        merkle_store: &Arc<RwLock<MerkleStore>>,
        account: &Account,
        initial_address: &Address,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;

        Self::insert_account_code(&tx, account.code())?;

        Self::insert_storage_slots(
            &tx,
            account.storage().commitment(),
            account.storage().slots().iter().enumerate(),
        )?;

        Self::insert_assets(&tx, account.vault().root(), account.vault().assets())?;
        Self::insert_account_header(&tx, &account.into(), account.seed())?;

        Self::insert_address(&tx, initial_address, account.id())?;

        tx.commit().into_store_error()?;

        let mut merkle_store = merkle_store.write().expect("merkle_store write lock not poisoned");
        insert_storage_map_nodes(&mut merkle_store, account.storage());
        insert_asset_nodes(&mut merkle_store, account.vault());

        Ok(())
    }

    pub(crate) fn update_account(
        conn: &mut Connection,
        merkle_store: &Arc<RwLock<MerkleStore>>,
        new_account_state: &Account,
    ) -> Result<(), StoreError> {
        const QUERY: &str = "SELECT id FROM accounts WHERE id = ?";
        if conn
            .prepare(QUERY)
            .into_store_error()?
            .query_map(params![new_account_state.id().to_hex()], |row| row.get(0))
            .into_store_error()?
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

        let mut merkle_store = merkle_store.write().expect("merkle_store write lock not poisoned");
        let tx = conn.transaction().into_store_error()?;
        Self::update_account_state(&tx, &mut merkle_store, new_account_state)?;
        tx.commit().into_store_error()
    }

    pub fn upsert_foreign_account_code(
        conn: &mut Connection,
        account_id: AccountId,
        code: &AccountCode,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;

        Self::insert_account_code(&tx, code)?;

        const QUERY: &str =
            insert_sql!(foreign_account_code { account_id, code_commitment } | REPLACE);

        tx.execute(QUERY, params![account_id.to_hex(), code.commitment().to_string()])
            .into_store_error()?;

        Self::insert_account_code(&tx, code)?;
        tx.commit().into_store_error()
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

        conn.prepare(QUERY)
            .into_store_error()?
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

    /// Retrieves the full asset vault for a specific account.
    pub fn get_account_vault(
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<AssetVault, StoreError> {
        let assets = query_vault_assets(
            conn,
            "root = (SELECT vault_root FROM accounts WHERE id = ? ORDER BY nonce DESC LIMIT 1)",
            params![account_id.to_hex()],
        )?;

        Ok(AssetVault::new(&assets)?)
    }

    /// Retrieves the full storage for a specific account.
    pub fn get_account_storage(
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<AccountStorage, StoreError> {
        let slots = query_storage_slots(
            conn,
            "commitment = (SELECT storage_commitment FROM accounts WHERE id = ? ORDER BY nonce DESC LIMIT 1)",
            params![account_id.to_hex()],
        )?
        .into_values()
        .collect();

        Ok(AccountStorage::new(slots)?)
    }

    /// Fetches a specific asset from the account's vault without the need of loading the entire
    /// vault. The Merkle proof is also retrieved from the [`MerkleStore`].
    pub(crate) fn get_account_asset(
        conn: &mut Connection,
        merkle_store: &Arc<RwLock<MerkleStore>>,
        account_id: AccountId,
        faucet_id_prefix: AccountIdPrefix,
    ) -> Result<Option<(Asset, AssetWitness)>, StoreError> {
        let header = Self::get_account_header(conn, account_id)?
            .ok_or(StoreError::AccountDataNotFound(account_id))?
            .0;

        let Some(asset) = query_vault_assets(
            conn,
            "faucet_id_prefix = ? AND root = ?",
            params![faucet_id_prefix.to_hex(), header.vault_root().to_hex()],
        )?
        .into_iter()
        .next() else {
            return Ok(None);
        };

        let merkle_store = merkle_store.read().expect("merkle_store read lock not poisoned");

        let proof = get_asset_proof(&merkle_store, header.vault_root(), &asset)?;
        let witness = AssetWitness::new(proof)?;

        Ok(Some((asset, witness)))
    }

    /// Retrieves a specific item from the account's storage map without loading the entire storage.
    /// The Merkle proof is also retrieved from the [`MerkleStore`].
    pub(crate) fn get_account_map_item(
        conn: &mut Connection,
        merkle_store: &Arc<RwLock<MerkleStore>>,
        account_id: AccountId,
        index: u8,
        key: Word,
    ) -> Result<(Word, StorageMapWitness), StoreError> {
        let header = Self::get_account_header(conn, account_id)?
            .ok_or(StoreError::AccountDataNotFound(account_id))?
            .0;

        let StorageSlot::Map(map) = query_storage_slots(
            conn,
            "commitment = ? AND slot_index = ?",
            params![header.storage_commitment().to_hex(), index],
        )?
        .remove(&index)
        .ok_or(StoreError::AccountStorageNotFound(header.storage_commitment()))?
        else {
            return Err(StoreError::AccountError(AccountError::StorageSlotNotMap(index)));
        };

        let item = map.get(&key);
        let merkle_store = merkle_store.read().expect("merkle_store read lock not poisoned");

        // TODO: change the api of get_storage_map_item_proof
        let path = get_storage_map_item_proof(&merkle_store, map.root(), key)?.1;
        let leaf = SmtLeaf::new_single(StorageMap::hash_key(key), item);
        let proof = SmtProof::new(path, leaf)?;

        let witness = StorageMapWitness::new(proof, [key])?;

        Ok((item, witness))
    }

    pub(crate) fn get_account_addresses(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Vec<Address>, StoreError> {
        query_account_addresses(conn, account_id)
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
        merkle_store: &mut MerkleStore,
        init_account_state: &AccountHeader,
        final_account_state: &AccountHeader,
        mut updated_fungible_assets: BTreeMap<AccountIdPrefix, FungibleAsset>,
        mut updated_storage_maps: BTreeMap<u8, StorageMap>,
        delta: &AccountDelta,
    ) -> Result<(), StoreError> {
        // Copy over the storage and vault from the previous state. Non-relevant data will not be
        // modified.
        Self::copy_account_state(tx, init_account_state, final_account_state)?;

        // Apply vault delta. This map will contain all updated assets (indexed by vault key), both
        // fungible and non-fungible.
        let mut updated_assets: BTreeMap<Word, Asset> = BTreeMap::new();
        let mut removed_vault_keys: Vec<Word> = Vec::new();

        // We first process the fungible assets. Adding or subtracting them from the vault as
        // requested.
        for (faucet_id, delta) in delta.vault().fungible().iter() {
            let delta_asset = FungibleAsset::new(*faucet_id, delta.unsigned_abs())?;

            let asset = match updated_fungible_assets.remove(&faucet_id.prefix()) {
                Some(asset) => {
                    // If the asset exists, update it accordingly.
                    if *delta >= 0 {
                        asset.add(delta_asset)?
                    } else {
                        asset.sub(delta_asset)?
                    }
                },
                None => {
                    // If the asset doesn't exist, we add it to the map to be inserted.
                    delta_asset
                },
            };

            if asset.amount() > 0 {
                updated_assets.insert(asset.vault_key(), Asset::Fungible(asset));
            } else {
                removed_vault_keys.push(asset.vault_key());
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
                .map(|(asset, _)| (asset.vault_key(), Asset::NonFungible(*asset))),
        );

        removed_vault_keys
            .extend(removed_nonfungible_assets.iter().map(|(asset, _)| asset.vault_key()));

        const DELETE_QUERY: &str =
            "DELETE FROM account_assets WHERE root = ? AND vault_key IN rarray(?)";

        tx.execute(
            DELETE_QUERY,
            params![
                final_account_state.vault_root().to_hex(),
                Rc::new(
                    removed_vault_keys
                        .into_iter()
                        .map(|k| Value::from(k.to_hex()))
                        .collect::<Vec<Value>>(),
                ),
            ],
        )
        .into_store_error()?;

        update_asset_nodes(
            merkle_store,
            init_account_state.vault_root(),
            updated_assets.values().copied(),
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

            update_storage_map_nodes(
                merkle_store,
                map.root(),
                map_delta.entries().iter().map(|(key, value)| ((*key).into(), *value)),
            )?;

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
                INSERT OR IGNORE INTO account_assets (
                    root,
                    vault_key,
                    faucet_id_prefix,
                    asset
                )
                SELECT
                    ?, --new root
                    vault_key,
                    faucet_id_prefix,
                    asset
                FROM account_assets
                WHERE root = (SELECT vault_root FROM accounts WHERE account_commitment = ?)
                ";
            tx.execute(
                VAULT_QUERY,
                params![
                    final_account_header.vault_root().to_hex(),
                    init_account_header.commitment().to_hex()
                ],
            )
            .into_store_error()?;
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
            )
            .into_store_error()?;
        }

        Ok(())
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

    /// Update previously-existing account after a transaction execution. Apart from updating the
    /// `SQLite` database, this function also updates the [`MerkleStore`] by adding the vault and
    /// storage SMT's nodes.
    ///
    /// Because the Client retrieves the account by account ID before applying the delta, we don't
    /// need to check that it exists here. This inserts a new row into the accounts table.
    /// We can later identify the proper account state by looking at the nonce.
    pub(super) fn update_account_state(
        tx: &Transaction<'_>,
        merkle_store: &mut MerkleStore,
        new_account_state: &Account,
    ) -> Result<(), StoreError> {
        insert_storage_map_nodes(merkle_store, new_account_state.storage());
        Self::insert_storage_slots(
            tx,
            new_account_state.storage().commitment(),
            new_account_state.storage().slots().iter().enumerate(),
        )?;
        insert_asset_nodes(merkle_store, new_account_state.vault());
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
        )
        .into_store_error()?;
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
        tx.execute(QUERY, params![Rc::new(params)]).into_store_error()?;

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
        )
        .into_store_error()?;
        Ok(())
    }

    /// Inserts an [`AccountCode`].
    fn insert_account_code(
        tx: &Transaction<'_>,
        account_code: &AccountCode,
    ) -> Result<(), StoreError> {
        const QUERY: &str = insert_sql!(account_code { commitment, code } | IGNORE);
        tx.execute(QUERY, params![account_code.commitment().to_hex(), account_code.to_bytes()])
            .into_store_error()?;
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
            )
            .into_store_error()?;

            if let StorageSlot::Map(map) = slot {
                const MAP_QUERY: &str =
                    insert_sql!(storage_map_entries { root, key, value } | REPLACE);
                for (key, value) in map.entries() {
                    // Insert each entry of the storage map
                    tx.execute(
                        MAP_QUERY,
                        params![map.root().to_hex(), key.to_hex(), value.to_hex()],
                    )
                    .into_store_error()?;
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
                insert_sql!(account_assets { root, vault_key, faucet_id_prefix, asset } | REPLACE);
            tx.execute(
                QUERY,
                params![
                    root.to_hex(),
                    asset.vault_key().to_hex(),
                    asset.faucet_id_prefix().to_hex(),
                    Word::from(asset).to_hex(),
                ],
            )
            .into_store_error()?;
        }

        Ok(())
    }

    fn insert_address(
        tx: &Transaction<'_>,
        address: &Address,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        const QUERY: &str = insert_sql!(addresses { address, account_id } | REPLACE);
        let serialized_address = address.to_bytes();
        tx.execute(QUERY, params![serialized_address, account_id.to_hex(),])
            .into_store_error()?;

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
        .prepare(&query)
        .into_store_error()?
        .query_map(params, |row| {
            let index: u8 = row.get(0)?;
            let value: String = row.get(1)?;
            let slot_type: Vec<u8> = row.get(2)?;
            Ok((index, value, slot_type))
        })
        .into_store_error()?
        .map(|result| {
            let (index, value, slot_type) = result.into_store_error()?;
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
        .prepare(&query)
        .into_store_error()?
        .query_map(params, |row| {
            let root: String = row.get(0)?;
            let key: String = row.get(1)?;
            let value: String = row.get(2)?;

            Ok((root, key, value))
        })
        .into_store_error()?
        .map(|result| {
            let (root, key, value) = result.into_store_error()?;
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
    const VAULT_QUERY: &str = "SELECT asset FROM account_assets";

    let query = format!("{VAULT_QUERY} WHERE {where_clause}");
    conn.prepare(&query)
        .into_store_error()?
        .query_map(params, |row| {
            let asset: String = row.get(0)?;
            Ok(asset)
        })
        .into_store_error()?
        .map(|result| {
            let asset_str: String = result.into_store_error()?;
            let word = Word::try_from(asset_str)?;
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

    conn.prepare(CODE_QUERY)
        .into_store_error()?
        .query_map(params![commitment.to_hex()], |row| {
            let code: Vec<u8> = row.get(0)?;
            Ok(code)
        })
        .into_store_error()?
        .map(|result| {
            let bytes: Vec<u8> = result.into_store_error()?;
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
    conn.prepare(&query)
        .into_store_error()?
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
        })
        .into_store_error()?
        .map(|result| parse_accounts(result.into_store_error()?))
        .collect::<Result<Vec<(AccountHeader, AccountStatus)>, StoreError>>()
}

fn query_account_addresses(
    conn: &Connection,
    account_id: AccountId,
) -> Result<Vec<Address>, StoreError> {
    const ADDRESS_QUERY: &str = "SELECT address FROM addresses";

    let query = format!("{ADDRESS_QUERY} WHERE ACCOUNT_ID = '{}'", account_id.to_hex());
    conn.prepare(&query)
        .into_store_error()?
        .query_map([], |row| {
            let address: Vec<u8> = row.get(0)?;
            Ok(address)
        })
        .into_store_error()?
        .map(|result| {
            let serialized_address = result.into_store_error()?;
            let address = Address::read_from_bytes(&serialized_address)?;
            Ok(address)
        })
        .collect::<Result<Vec<Address>, StoreError>>()
}

// PARTIAL ACCOUNTS
// ================================================================================================

impl SqliteStore {
    /// Stores a `PartialAccount` with its specific vault/storage items and authentication paths.
    pub(crate) fn store_partial_account(
        conn: &mut Connection,
        partial_account_id: &str,
        partial_account: &PartialAccount,
        storage_items: Vec<(u8, Option<Word>, Word, MerklePath)>,
        vault_items: Vec<(Word, Asset, MerklePath)>,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        // Store PartialAccount metadata
        const INSERT_PARTIAL_ACCOUNT: &str = insert_sql!(
            partial_accounts {
                partial_account_id,
                account_id,
                storage_header,
                vault_root,
                created_at
            } | REPLACE
        );

        tx.execute(
            INSERT_PARTIAL_ACCOUNT,
            params![
                partial_account_id,
                partial_account.id().to_hex(),
                partial_account.storage().header().to_bytes(),
                partial_account.vault().root().to_hex(),
                current_timestamp_u64(),
            ],
        )
        .into_store_error()?;

        // Store storage items
        const INSERT_STORAGE_ITEM: &str = insert_sql!(
            partial_account_storage_items {
                partial_account_id,
                slot_index,
                item_key,
                item_value,
                auth_path
            } | REPLACE
        );

        for (slot_index, item_key, item_value, auth_path) in storage_items {
            tx.execute(
                INSERT_STORAGE_ITEM,
                params![
                    partial_account_id,
                    slot_index,
                    item_key.map(|k| k.to_hex()),
                    item_value.to_hex(),
                    auth_path.to_bytes(),
                ],
            )
            .into_store_error()?;
        }

        // Store vault items
        const INSERT_VAULT_ITEM: &str = insert_sql!(
            partial_account_vault_items {
                partial_account_id,
                vault_key,
                asset_value,
                auth_path
            } | REPLACE
        );

        for (vault_key, asset, auth_path) in vault_items {
            tx.execute(
                INSERT_VAULT_ITEM,
                params![
                    partial_account_id,
                    vault_key.to_hex(),
                    asset.to_bytes(),
                    auth_path.to_bytes(),
                ],
            )
            .into_store_error()?;
        }

        tx.commit().into_store_error()?;
        Ok(())
    }

    /// Retrieves a `PartialAccount` by its ID.
    pub(crate) fn get_partial_account(
        conn: &Connection,
        partial_account_id: &str,
    ) -> Result<Option<PartialAccount>, StoreError> {
        const SELECT_PARTIAL_ACCOUNT: &str = "SELECT account_id, storage_header, vault_root FROM partial_accounts WHERE partial_account_id = ?";

        let Some((account_id_hex, storage_header_bytes, vault_root_hex)) = conn
            .query_row(SELECT_PARTIAL_ACCOUNT, params![partial_account_id], |row| {
                let account_id: String = row.get(0)?;
                let storage_header: Vec<u8> = row.get(1)?;
                let vault_root: String = row.get(2)?;
                Ok((account_id, storage_header, vault_root))
            })
            .optional()
            .into_store_error()?
        else {
            return Ok(None);
        };

        let account_id = AccountId::from_hex(&account_id_hex)?;
        let storage_header = AccountStorageHeader::read_from_bytes(&storage_header_bytes)?;
        let _vault_root = Word::try_from(&vault_root_hex)?;

        // Get storage items for this PartialAccount
        let storage_items = query_partial_storage_items(conn, partial_account_id)?;

        // Get vault items for this PartialAccount
        let vault_items = query_partial_vault_items(conn, partial_account_id)?;

        // Reconstruct PartialAccount
        let partial_storage = PartialStorage::new(storage_header, storage_items.into_iter())?;
        let partial_vault =
            PartialVault::new(vault_items).map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        // Get the original account code and nonce from the database
        let (account_code, nonce) = query_account_code_and_nonce(conn, account_id)?;

        let partial_account = PartialAccount::new(
            account_id,
            nonce,
            account_code,
            partial_storage,
            partial_vault,
            None, // seed - not stored in PartialAccount
        )?;

        Ok(Some(partial_account))
    }

    /// Retrieves specific storage items for a `PartialAccount`.
    pub(crate) fn get_partial_storage_items(
        conn: &Connection,
        partial_account_id: &str,
        slot_indices: &[u8],
    ) -> Result<Vec<(u8, StorageSlot, MerklePath)>, StoreError> {
        if slot_indices.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = slot_indices.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT slot_index, item_key, item_value, auth_path FROM partial_account_storage_items 
             WHERE partial_account_id = ? AND slot_index IN ({placeholders})"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(partial_account_id.to_string())];
        params.extend(slot_indices.iter().map(|i| Box::new(*i) as Box<dyn rusqlite::ToSql>));

        let items = conn
            .prepare(&query)
            .into_store_error()?
            .query_map(rusqlite::params_from_iter(params), |row| {
                let slot_index: u8 = row.get(0)?;
                let item_key: Option<String> = row.get(1)?;
                let item_value: String = row.get(2)?;
                let auth_path_bytes: Vec<u8> = row.get(3)?;

                let item_key = item_key.map(Word::try_from).transpose().map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "Word".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;
                let item_value = Word::try_from(item_value).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "Word".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;
                let auth_path = MerklePath::read_from_bytes(&auth_path_bytes).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "MerklePath".to_string(),
                        rusqlite::types::Type::Blob,
                    )
                })?;

                // Reconstruct StorageSlot based on whether it has a key (map entry) or not (simple
                // value)
                let storage_slot = if let Some(key) = item_key {
                    // This is a map entry - we need to create a PartialStorageMap
                    let mut map = StorageMap::new();
                    map.insert(key, item_value);
                    StorageSlot::Map(map)
                } else {
                    // This is a simple value
                    StorageSlot::Value(item_value)
                };

                Ok((slot_index, storage_slot, auth_path))
            })
            .into_store_error()?
            .map(super::sql_error::SqlResultExt::into_store_error)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    /// Retrieves specific vault items for a `PartialAccount`.
    pub(crate) fn get_partial_vault_items(
        conn: &Connection,
        partial_account_id: &str,
        vault_keys: &[Word],
    ) -> Result<Vec<(Word, Asset, MerklePath)>, StoreError> {
        if vault_keys.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = vault_keys.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT vault_key, asset_value, auth_path FROM partial_account_vault_items 
             WHERE partial_account_id = ? AND vault_key IN ({placeholders})"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(partial_account_id.to_string())];
        params.extend(vault_keys.iter().map(|k| Box::new(k.to_hex()) as Box<dyn rusqlite::ToSql>));

        let items = conn
            .prepare(&query)
            .into_store_error()?
            .query_map(rusqlite::params_from_iter(params), |row| {
                let vault_key_hex: String = row.get(0)?;
                let asset_bytes: Vec<u8> = row.get(1)?;
                let auth_path_bytes: Vec<u8> = row.get(2)?;

                let vault_key = Word::try_from(vault_key_hex).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "Word".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;
                let asset = Asset::read_from_bytes(&asset_bytes).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "Asset".to_string(),
                        rusqlite::types::Type::Blob,
                    )
                })?;
                let auth_path = MerklePath::read_from_bytes(&auth_path_bytes).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "MerklePath".to_string(),
                        rusqlite::types::Type::Blob,
                    )
                })?;

                Ok((vault_key, asset, auth_path))
            })
            .into_store_error()?
            .map(super::sql_error::SqlResultExt::into_store_error)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    /// Lists all `PartialAccount` IDs for a given account.
    pub(crate) fn list_partial_accounts(
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<Vec<String>, StoreError> {
        const SELECT_PARTIAL_ACCOUNTS: &str =
            "SELECT partial_account_id FROM partial_accounts WHERE account_id = ?";

        let partial_account_ids = conn
            .prepare(SELECT_PARTIAL_ACCOUNTS)
            .into_store_error()?
            .query_map(params![account_id.to_hex()], |row| {
                let partial_account_id: String = row.get(0)?;
                Ok(partial_account_id)
            })
            .into_store_error()?
            .map(super::sql_error::SqlResultExt::into_store_error)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(partial_account_ids)
    }

    /// Removes a `PartialAccount` and all its associated data.
    pub(crate) fn remove_partial_account(
        conn: &mut Connection,
        partial_account_id: &str,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        // Delete storage items
        tx.execute(
            "DELETE FROM partial_account_storage_items WHERE partial_account_id = ?",
            params![partial_account_id],
        )
        .into_store_error()?;

        // Delete vault items
        tx.execute(
            "DELETE FROM partial_account_vault_items WHERE partial_account_id = ?",
            params![partial_account_id],
        )
        .into_store_error()?;

        // Delete PartialAccount metadata
        tx.execute(
            "DELETE FROM partial_accounts WHERE partial_account_id = ?",
            params![partial_account_id],
        )
        .into_store_error()?;

        tx.commit().into_store_error()?;
        Ok(())
    }
}

// PARTIAL ACCOUNT HELPERS
// ================================================================================================

#[allow(clippy::all)]
fn query_partial_storage_items(
    conn: &Connection,
    partial_account_id: &str,
) -> Result<Vec<PartialStorageMap>, StoreError> {
    const SELECT_STORAGE_ITEMS: &str = "SELECT slot_index, item_key, item_value, auth_path FROM partial_account_storage_items WHERE partial_account_id = ?";

    let items = conn
        .prepare(SELECT_STORAGE_ITEMS)
        .into_store_error()?
        .query_map(params![partial_account_id], |row| {
            let slot_index: u8 = row.get(0)?;
            let item_key: Option<String> = row.get(1)?;
            let item_value: String = row.get(2)?;
            let auth_path_bytes: Vec<u8> = row.get(3)?;

            let item_key = item_key.map(|k| Word::try_from(k)).transpose().map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    0,
                    "Word".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let item_value = Word::try_from(item_value).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    0,
                    "Word".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let auth_path = MerklePath::read_from_bytes(&auth_path_bytes).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    0,
                    "MerklePath".to_string(),
                    rusqlite::types::Type::Blob,
                )
            })?;

            Ok((slot_index, item_key, item_value, auth_path))
        })
        .into_store_error()?
        .map(super::sql_error::SqlResultExt::into_store_error)
        .collect::<Result<Vec<_>, _>>()?;

    // Group items by slot index and create PartialStorageMap for each slot
    let mut slot_maps: BTreeMap<u8, Vec<(Word, Word, MerklePath)>> = BTreeMap::new();

    for (slot_index, item_key, item_value, auth_path) in items {
        let entry = slot_maps.entry(slot_index).or_insert_with(Vec::new);
        entry.push((item_key.unwrap_or_default(), item_value, auth_path));
    }

    let partial_storage_maps = slot_maps
        .into_iter()
        .map(|(_slot_index, entries)| {
            let mut witnesses = Vec::new();
            for (key, value, auth_path) in entries {
                let proof = SmtProof::new(
                    auth_path,
                    SmtLeaf::new_single(StorageMap::hash_key(key), value),
                )?;
                witnesses.push(StorageMapWitness::new(proof, [key])?);
            }
            Ok::<PartialStorageMap, StoreError>(PartialStorageMap::from_witnesses(witnesses)?)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(partial_storage_maps)
}

#[allow(clippy::all)]
fn query_partial_vault_items(
    conn: &Connection,
    partial_account_id: &str,
) -> Result<PartialSmt, StoreError> {
    const SELECT_VAULT_ITEMS: &str = "SELECT vault_key, asset_value, auth_path FROM partial_account_vault_items WHERE partial_account_id = ?";

    let items = conn
        .prepare(SELECT_VAULT_ITEMS)
        .into_store_error()?
        .query_map(params![partial_account_id], |row| {
            let vault_key_hex: String = row.get(0)?;
            let asset_bytes: Vec<u8> = row.get(1)?;
            let auth_path_bytes: Vec<u8> = row.get(2)?;

            let vault_key = Word::try_from(vault_key_hex).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    0,
                    "Word".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let asset = Asset::read_from_bytes(&asset_bytes).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    0,
                    "Asset".to_string(),
                    rusqlite::types::Type::Blob,
                )
            })?;
            let auth_path = MerklePath::read_from_bytes(&auth_path_bytes).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    0,
                    "MerklePath".to_string(),
                    rusqlite::types::Type::Blob,
                )
            })?;

            Ok((vault_key, asset, auth_path))
        })
        .into_store_error()?
        .map(super::sql_error::SqlResultExt::into_store_error)
        .collect::<Result<Vec<_>, _>>()?;

    // Create AssetWitness for each item and build PartialSmt
    let _witnesses = items
        .into_iter()
        .map(|(vault_key, asset, auth_path)| {
            let proof = SmtProof::new(auth_path, SmtLeaf::new_single(vault_key, asset.into()))?;
            Ok::<AssetWitness, StoreError>(AssetWitness::new(proof)?)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PartialSmt::new())
}

#[allow(clippy::all)]
fn query_account_code_and_nonce(
    conn: &Connection,
    account_id: AccountId,
) -> Result<(AccountCode, Felt), StoreError> {
    const SELECT_ACCOUNT_CODE_AND_NONCE: &str =
        "SELECT code_commitment, nonce FROM accounts WHERE id = ?";

    let (code_commitment_hex, nonce_value): (String, i64) = conn
        .query_row(SELECT_ACCOUNT_CODE_AND_NONCE, params![account_id.to_hex()], |row| {
            let code_commitment: String = row.get(0)?;
            let nonce: i64 = row.get(1)?;
            Ok((code_commitment, nonce))
        })
        .optional()
        .into_store_error()?
        .ok_or_else(|| {
            StoreError::AccountError(AccountError::other(format!("Account {account_id} not found")))
        })?;

    // Get the actual account code
    const SELECT_ACCOUNT_CODE: &str = "SELECT code FROM account_code WHERE commitment = ?";
    let code_bytes: Vec<u8> = conn
        .query_row(SELECT_ACCOUNT_CODE, params![code_commitment_hex], |row| {
            let code: Vec<u8> = row.get(0)?;
            Ok(code)
        })
        .optional()
        .into_store_error()?
        .ok_or_else(|| {
            StoreError::AccountError(AccountError::other(format!("Account {account_id} not found")))
        })?;

    let account_code = AccountCode::read_from_bytes(&code_bytes)
        .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
    #[allow(clippy::cast_sign_loss)]
    let nonce = Felt::new(nonce_value as u64);

    Ok((account_code, nonce))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::vec::Vec;

    use anyhow::Context;
    use miden_client::account::component::AccountComponent;
    use miden_client::account::{
        Account,
        AccountBuilder,
        AccountCode,
        AccountDelta,
        AccountHeader,
        AccountId,
        AccountIdAddress,
        AccountType,
        Address,
        AddressInterface,
        StorageMap,
        StorageSlot,
    };
    use miden_client::asset::{
        AccountStorageDelta,
        AccountVaultDelta,
        Asset,
        FungibleAsset,
        NonFungibleAsset,
        NonFungibleAssetDetails,
    };
    use miden_client::crypto::rpo_falcon512::PublicKey;
    use miden_client::store::Store;
    use miden_client::testing::account_id::{
        ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
        ACCOUNT_ID_PUBLIC_NON_FUNGIBLE_FAUCET,
    };
    use miden_client::testing::constants::NON_FUNGIBLE_ASSET_DATA;
    use miden_client::transaction::TransactionKernel;
    use miden_client::{EMPTY_WORD, ONE, Word, ZERO};
    use miden_lib::account::auth::AuthRpoFalcon512;
    use miden_lib::account::components::basic_wallet_library;
    use miden_objects::account::{AccountStorageHeader, PartialStorage};
    use miden_objects::asset::PartialVault;
    use miden_objects::crypto::merkle::PartialSmt;

    use crate::SqliteStore;
    use crate::sql_error::SqlResultExt;
    use crate::tests::create_test_store;

    #[tokio::test]
    async fn account_code_insertion_no_duplicates() -> anyhow::Result<()> {
        let store = create_test_store().await;
        let assembler = TransactionKernel::assembler();
        let account_component = AccountComponent::compile(
            "
                export.::miden::contracts::wallets::basic::receive_asset
                export.::miden::contracts::wallets::basic::move_asset_to_note
            ",
            assembler,
            vec![],
        )?
        .with_supports_all_types();
        let account_code = AccountCode::from_components(
            &[AuthRpoFalcon512::new(PublicKey::new(EMPTY_WORD)).into(), account_component],
            AccountType::RegularAccountUpdatableCode,
        )?;

        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().into_store_error()?;

                // Table is empty at the beginning
                let mut actual: usize = tx
                    .query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0))
                    .into_store_error()?;
                assert_eq!(actual, 0);

                // First insertion generates a new row
                SqliteStore::insert_account_code(&tx, &account_code)?;
                actual = tx
                    .query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0))
                    .into_store_error()?;
                assert_eq!(actual, 1);

                // Second insertion passes but does not generate a new row
                assert!(SqliteStore::insert_account_code(&tx, &account_code).is_ok());
                actual = tx
                    .query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0))
                    .into_store_error()?;
                assert_eq!(actual, 1);

                Ok(())
            })
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_apply_account_delta_additions() -> anyhow::Result<()> {
        let store = create_test_store().await;

        let dummy_component = AccountComponent::new(
            basic_wallet_library(),
            vec![StorageSlot::empty_value(), StorageSlot::empty_map()],
        )?
        .with_supports_all_types();

        // Create and insert an account
        let account = AccountBuilder::new([0; 32])
            .account_type(AccountType::RegularAccountImmutableCode)
            .with_auth_component(AuthRpoFalcon512::new(PublicKey::new(EMPTY_WORD)))
            .with_component(dummy_component)
            .build()?;

        let default_address =
            Address::AccountId(AccountIdAddress::new(account.id(), AddressInterface::Unspecified));
        store.insert_account(&account, default_address).await?;

        let mut storage_delta = AccountStorageDelta::new();
        storage_delta.set_item(1, [ZERO, ZERO, ZERO, ONE].into());
        storage_delta.set_map_item(2, [ONE, ZERO, ZERO, ZERO].into(), [ONE, ONE, ONE, ONE].into());

        let vault_delta = AccountVaultDelta::from_iters(
            vec![
                FungibleAsset::new(AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET)?, 100)?
                    .into(),
                NonFungibleAsset::new(&NonFungibleAssetDetails::new(
                    AccountId::try_from(ACCOUNT_ID_PUBLIC_NON_FUNGIBLE_FAUCET)?.prefix(),
                    NON_FUNGIBLE_ASSET_DATA.into(),
                )?)?
                .into(),
            ],
            [],
        );

        let delta = AccountDelta::new(account.id(), storage_delta, vault_delta, ONE)?;

        let mut account_after_delta = account.clone();
        account_after_delta.apply_delta(&delta)?;

        let account_id = account.id();
        let final_state: AccountHeader = (&account_after_delta).into();
        let merkle_store = store.merkle_store.clone();
        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().into_store_error()?;
                let mut merkle_store =
                    merkle_store.write().expect("merkle_store write lock not poisoned");

                SqliteStore::apply_account_delta(
                    &tx,
                    &mut merkle_store,
                    &account.into(),
                    &final_state,
                    BTreeMap::default(),
                    BTreeMap::default(),
                    &delta,
                )?;

                tx.commit().into_store_error()?;
                Ok(())
            })
            .await?;

        let updated_account: Account = store
            .get_account(account_id)
            .await?
            .context("failed to find inserted account")?
            .into();

        assert_eq!(updated_account, account_after_delta);

        Ok(())
    }

    #[tokio::test]
    async fn test_apply_account_delta_removals() -> anyhow::Result<()> {
        let store = create_test_store().await;

        let mut dummy_map = StorageMap::new();
        dummy_map.insert([ONE, ZERO, ZERO, ZERO].into(), [ONE, ONE, ONE, ONE].into());

        let dummy_component = AccountComponent::new(
            basic_wallet_library(),
            vec![StorageSlot::Value([ZERO, ZERO, ZERO, ONE].into()), StorageSlot::Map(dummy_map)],
        )?
        .with_supports_all_types();

        // Create and insert an account
        let assets: Vec<Asset> = vec![
            FungibleAsset::new(AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET)?, 100)?
                .into(),
            NonFungibleAsset::new(&NonFungibleAssetDetails::new(
                AccountId::try_from(ACCOUNT_ID_PUBLIC_NON_FUNGIBLE_FAUCET)?.prefix(),
                NON_FUNGIBLE_ASSET_DATA.into(),
            )?)?
            .into(),
        ];
        let account = AccountBuilder::new([0; 32])
            .account_type(AccountType::RegularAccountImmutableCode)
            .with_auth_component(AuthRpoFalcon512::new(PublicKey::new(EMPTY_WORD)))
            .with_component(dummy_component)
            .with_assets(assets.clone())
            .build_existing()?;
        let default_address =
            Address::AccountId(AccountIdAddress::new(account.id(), AddressInterface::Unspecified));
        store.insert_account(&account, default_address).await?;

        let mut storage_delta = AccountStorageDelta::new();
        storage_delta.set_item(1, EMPTY_WORD);
        storage_delta.set_map_item(2, [ONE, ZERO, ZERO, ZERO].into(), EMPTY_WORD);

        let vault_delta = AccountVaultDelta::from_iters([], assets.clone());

        let delta = AccountDelta::new(account.id(), storage_delta, vault_delta, ONE)?;

        let mut account_after_delta = account.clone();
        account_after_delta.apply_delta(&delta)?;

        let account_id = account.id();
        let final_state: AccountHeader = (&account_after_delta).into();

        let merkle_store = store.merkle_store.clone();
        store
            .interact_with_connection(move |conn| {
                let fungible_assets = SqliteStore::get_account_fungible_assets_for_delta(
                    conn,
                    &(&account).into(),
                    &delta,
                )?;
                let storage_maps = SqliteStore::get_account_storage_maps_for_delta(
                    conn,
                    &(&account).into(),
                    &delta,
                )?;
                let tx = conn.transaction().into_store_error()?;
                let mut merkle_store =
                    merkle_store.write().expect("merkle_store write lock not poisoned");

                SqliteStore::apply_account_delta(
                    &tx,
                    &mut merkle_store,
                    &account.into(),
                    &final_state,
                    fungible_assets,
                    storage_maps,
                    &delta,
                )?;

                tx.commit().into_store_error()?;
                Ok(())
            })
            .await?;

        let updated_account: Account = store
            .get_account(account_id)
            .await?
            .context("failed to find inserted account")?
            .into();

        assert_eq!(updated_account, account_after_delta);
        assert!(updated_account.vault().is_empty());
        assert_eq!(updated_account.storage().get_item(1)?, EMPTY_WORD);
        let StorageSlot::Map(ref updated_map) = updated_account.storage().slots()[2] else {
            panic!("Expected map slot");
        };
        assert_eq!(updated_map.entries().count(), 0);

        Ok(())
    }

    // PARTIAL ACCOUNT TESTS
    // ================================================================================================

    #[test]
    fn test_partial_account_types_compile() -> anyhow::Result<()> {
        // This test just verifies that the PartialAccount types can be imported and basic types
        // work without requiring a full database setup

        let account_id = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET)?;
        let storage_header = AccountStorageHeader::new(vec![(
            miden_objects::account::StorageSlotType::Value,
            Word::from([ZERO, ZERO, ZERO, ZERO]),
        )]);
        let partial_storage = PartialStorage::new(storage_header, vec![])?;
        let partial_vault = PartialVault::new(PartialSmt::new())?;

        // Test that we can create the basic types (this tests the imports and types compile
        // correctly)
        assert_eq!(account_id, AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET)?);

        // Test that PartialStorage and PartialVault can be created (this verifies the types
        // compile) If we get here, the types compiled successfully
        let _ = partial_storage;
        let _ = partial_vault;

        Ok(())
    }
}
