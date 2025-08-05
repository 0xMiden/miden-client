#![allow(clippy::items_after_statements)]

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use std::{collections::BTreeMap, rc::Rc};

use miden_objects::{
    AccountError, Felt, Word, WordError,
    account::{
        Account, AccountCode, AccountHeader, AccountId, AccountStorage, StorageMap, StorageSlot,
    },
    asset::{Asset, AssetVault, FungibleAsset, NonFungibleAsset},
};
use miden_tx::utils::{Deserializable, DeserializationError, Serializable};
use rusqlite::{Connection, Transaction, named_params, params, types::Value};

use super::{SqliteStore, column_value_as_u64, u64_to_value};
use crate::{
    insert_sql,
    store::{AccountRecord, AccountStatus, StoreError},
    subst,
};

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

type SerializedAccountCodeData = (String, Vec<u8>);

struct SerializedStorageSlotData {
    index: u64,
    value: Option<String>,
    map_root: Option<String>,
    map_entries: Vec<(String, String)>,
}

struct SerializedAssetData {
    fungible_faucet_id: Option<String>,
    fungible_faucet_amount: Option<u64>,
    non_fungible_asset: Option<Vec<u8>>,
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
        const QUERY: &str = "SELECT a.id, a.nonce, a.vault_root, a.storage_commitment, a.code_commitment, a.account_seed, a.locked \
            FROM accounts a \
            WHERE a.nonce = (SELECT MAX(b.nonce) FROM accounts b WHERE b.id = a.id)";

        conn.prepare(QUERY)?
            .query_map([], parse_account_header_columns)
            .expect("no binding parameters used in query")
            .map(|result| Ok(result?).and_then(parse_accounts))
            .collect()
    }

    pub(crate) fn get_account_header(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        const QUERY: &str = "SELECT id, nonce, vault_root, storage_commitment, code_commitment, account_seed, locked \
            FROM accounts WHERE id = ? \
            ORDER BY nonce DESC \
            LIMIT 1";
        conn.prepare(QUERY)?
            .query_map(params![account_id.to_hex()], parse_account_header_columns)?
            .map(|result| Ok(result?).and_then(parse_accounts))
            .next()
            .transpose()
    }

    pub(crate) fn get_account_header_by_commitment(
        conn: &mut Connection,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError> {
        let account_commitment_str: String = account_commitment.to_string();
        const QUERY: &str = "SELECT id, nonce, vault_root, storage_commitment, code_commitment, account_seed, locked \
            FROM accounts WHERE account_commitment = ?";

        conn.prepare(QUERY)?
            .query_map(params![account_commitment_str], parse_account_header_columns)?
            .map(|result| {
                let result = result?;
                Ok(parse_accounts(result)?.0)
            })
            .next()
            .map_or(Ok(None), |result| result.map(Some))
    }

    pub(crate) fn get_account(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        let Some((header, status)) = Self::get_account_header(conn, account_id)? else {
            return Ok(None);
        };

        const VAULT_QUERY: &str = "SELECT  fungible_faucet_id, fungible_faucet_amount, non_fungible_asset FROM account_vaults WHERE root = ?";
        let assets = conn
            .prepare(VAULT_QUERY)?
            .query_map(params![header.vault_root().to_hex()], parse_asset_columns)?
            .map(|result| Ok(result?).and_then(parse_asset))
            .collect::<Result<Vec<Asset>, StoreError>>()?;

        let vault = AssetVault::new(&assets)?;

        let storage = build_account_storage(conn, header.storage_commitment())?;

        const CODE_QUERY: &str = "SELECT code FROM account_code WHERE commitment = ?";
        let code = conn
            .prepare(CODE_QUERY)?
            .query_map(params![header.code_commitment().to_hex()], |row| {
                let code: Vec<u8> = row.get(0)?;
                Ok(code)
            })?
            .next()
            .transpose()?
            .unwrap(); // TODO: Remove
        let account_code = AccountCode::from_bytes(&code)?;

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

        insert_account_code(&tx, account.code())?;
        insert_account_storage(&tx, account.storage())?;
        insert_account_asset_vault(&tx, account.vault())?;
        insert_account_record(&tx, account, account_seed)?;

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
        update_account(&tx, new_account_state)?;
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

        insert_account_code(&tx, code)?;
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
            FROM foreign_account_code JOIN account_code ON code_commitment = code_commitment
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
}

// HELPERS
// ================================================================================================

/// Update previously-existing account after a transaction execution.
///
/// Because the Client retrieves the account by account ID before applying the delta, we don't
/// need to check that it exists here. This inserts a new row into the accounts table.
/// We can later identify the proper account state by looking at the nonce.
pub(crate) fn update_account(
    tx: &Transaction<'_>,
    new_account_state: &Account,
) -> Result<(), StoreError> {
    insert_account_storage(tx, new_account_state.storage())?;
    insert_account_asset_vault(tx, new_account_state.vault())?;
    insert_account_record(tx, new_account_state, None)
}

pub(super) fn insert_account_record(
    tx: &Transaction<'_>,
    account: &Account,
    account_seed: Option<Word>,
) -> Result<(), StoreError> {
    let id: String = account.id().to_hex();
    let code_commitment = account.code().commitment().to_string();
    let storage_commitment = account.storage().commitment().to_string();
    let vault_root = account.vault().root().to_string();
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
fn insert_account_code(tx: &Transaction<'_>, account_code: &AccountCode) -> Result<(), StoreError> {
    let (code_commitment, code) = serialize_account_code(account_code);
    const QUERY: &str = insert_sql!(account_code { commitment, code } | IGNORE);
    tx.execute(QUERY, params![code_commitment, code])?;
    Ok(())
}

/// Inserts an [`AccountStorage`].
pub(super) fn insert_account_storage(
    tx: &Transaction<'_>,
    account_storage: &AccountStorage,
) -> Result<(), StoreError> {
    for (index, slot) in account_storage.slots().iter().enumerate() {
        let SerializedStorageSlotData { index, value, map_root, map_entries } =
            serialize_account_storage_slot(
                u64::try_from(index).expect("There are at most 255 slots"),
                slot,
            );
        const QUERY: &str = insert_sql!(
            account_storage {
                commitment,
                slot_index,
                slot_value,
                slot_map_root
            } | IGNORE
        );

        tx.execute(
            QUERY,
            params![account_storage.commitment().to_string(), index, value, map_root,],
        )?;

        for (key, value) in map_entries {
            const MAP_ENTRY_QUERY: &str =
                insert_sql!(storage_map_entries { root, key, value } | IGNORE);
            tx.execute(MAP_ENTRY_QUERY, params![slot.value().to_string(), key, value])?;
        }
    }

    Ok(())
}

/// Inserts an [`AssetVault`].
pub(super) fn insert_account_asset_vault(
    tx: &Transaction<'_>,
    asset_vault: &AssetVault,
) -> Result<(), StoreError> {
    for asset in asset_vault.assets() {
        let serialized_asset = serialize_asset(&asset);
        const QUERY: &str = insert_sql!(
            account_vaults {
                root,
                faucet_id_prefix,
                fungible_faucet_id,
                fungible_faucet_amount,
                non_fungible_asset
            } | IGNORE
        );
        tx.execute(
            QUERY,
            params![
                asset_vault.root().to_string(),
                asset.faucet_id_prefix().to_string(),
                serialized_asset.fungible_faucet_id,
                serialized_asset.fungible_faucet_amount,
                serialized_asset.non_fungible_asset,
            ],
        )?;
    }

    Ok(())
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

/// Parse the account header columns from the provided row into its serialized form.
fn parse_account_header_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedHeaderData, rusqlite::Error> {
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
}

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

/// Serialize the provided `account_code` into database compatible types.
fn serialize_account_code(account_code: &AccountCode) -> SerializedAccountCodeData {
    let commitment = account_code.commitment().to_string();
    let code = account_code.to_bytes();

    (commitment, code)
}

/// Parse the storage slot columns from the provided row into its serialized form. This function
/// needs map entries to be retrieved first to properly parse the storage slot.
fn parse_storage_slot_columns(
    row: &rusqlite::Row<'_>,
    map_entries: &mut BTreeMap<String, Vec<(String, String)>>,
) -> Result<SerializedStorageSlotData, rusqlite::Error> {
    let index: u64 = column_value_as_u64(row, 0)?;
    let value: Option<String> = row.get(1)?;
    let map_root: Option<String> = row.get(2)?;
    let map_entries = if let Some(root) = &map_root {
        map_entries.remove(root).unwrap_or_default()
    } else {
        Vec::new()
    };

    Ok(SerializedStorageSlotData { index, value, map_root, map_entries })
}

/// Serialize the provided storage slot into its serialized form.
fn serialize_account_storage_slot(index: u64, slot: &StorageSlot) -> SerializedStorageSlotData {
    match slot {
        StorageSlot::Value(value) => SerializedStorageSlotData {
            index,
            value: Some(value.to_string()),
            map_root: None,
            map_entries: Vec::new(),
        },
        StorageSlot::Map(map) => SerializedStorageSlotData {
            index,
            value: None,
            map_root: Some(map.root().to_string()),
            map_entries: map
                .entries()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        },
    }
}

/// Serialize the provided asset into its serialized form.
fn serialize_asset(asset: &Asset) -> SerializedAssetData {
    match asset {
        Asset::Fungible(fungible) => {
            let id = fungible.faucet_id().to_hex();
            let amount = fungible.amount();
            SerializedAssetData {
                fungible_faucet_id: Some(id),
                fungible_faucet_amount: Some(amount),
                non_fungible_asset: None,
            }
        },
        Asset::NonFungible(non_fungible) => SerializedAssetData {
            fungible_faucet_id: None,
            fungible_faucet_amount: None,
            non_fungible_asset: Some(non_fungible.to_bytes()),
        },
    }
}

/// Parse the asset columns from the provided row into its serialized form.
fn parse_asset_columns(row: &rusqlite::Row<'_>) -> Result<SerializedAssetData, rusqlite::Error> {
    let fungible_faucet_id: Option<String> = row.get(0)?;
    let fungible_faucet_amount: Option<u64> = row.get(1)?;
    let non_fungible_asset: Option<Vec<u8>> = row.get(2)?;

    Ok(SerializedAssetData {
        fungible_faucet_id,
        fungible_faucet_amount,
        non_fungible_asset,
    })
}

/// Parse the serialized asset data into an `Asset`.
fn parse_asset(serialized_asset: SerializedAssetData) -> Result<Asset, StoreError> {
    let SerializedAssetData {
        fungible_faucet_id,
        fungible_faucet_amount,
        non_fungible_asset,
    } = serialized_asset;

    match (fungible_faucet_id, fungible_faucet_amount, non_fungible_asset) {
        (Some(faucet_id), Some(amount), None) => {
            let faucet_id = AccountId::from_hex(&faucet_id)?;
            let fungible_asset = FungibleAsset::new(faucet_id, amount)?;
            Ok(Asset::Fungible(fungible_asset))
        },
        (None, None, Some(non_fungible_asset)) => {
            let non_fungible_asset = NonFungibleAsset::read_from_bytes(&non_fungible_asset)?;
            Ok(Asset::NonFungible(non_fungible_asset))
        },
        _ => Err(StoreError::DataDeserializationError(DeserializationError::InvalidValue(
            "Invalid asset data".to_string(),
        ))),
    }
}

/// Removes account states with the specified hashes from the database.
///
/// This is used to rollback account changes when a transaction is discarded,
/// effectively undoing the account state changes that were applied by the transaction.
///
/// Note: This is not part of the Store trait and is only used internally by the `SQLite` store
/// implementation to handle transaction rollbacks.
pub(crate) fn undo_account_state(
    tx: &Transaction<'_>,
    account_hashes: &[Word],
) -> Result<(), StoreError> {
    const QUERY: &str = "DELETE FROM accounts WHERE account_commitment = ?";
    for account_id in account_hashes {
        tx.execute(QUERY, params![account_id.to_hex()])?;
    }
    Ok(())
}

/// Builds the [`AccountStorage`] with the provided storage commitment from the database.
pub(crate) fn build_account_storage(
    conn: &Connection,
    storage_commitment: Word,
) -> Result<AccountStorage, StoreError> {
    const STORAGE_MAP_QUERY: &str =
        "SELECT root, key, value FROM storage_map_entries WHERE root in (
                SELECT slot_map_root FROM account_storage WHERE root = ?
            )";

    let map_entries = conn
        .prepare(STORAGE_MAP_QUERY)?
        .query_map(params![storage_commitment.to_hex()], |row| {
            let root: String = row.get(0)?;
            let key: String = row.get(1)?;
            let value: String = row.get(2)?;
            Ok((root, key, value))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut map_entries: BTreeMap<String, Vec<(String, String)>> =
        map_entries.into_iter().fold(BTreeMap::new(), |mut acc, (root, key, value)| {
            acc.entry(root).or_default().push((key, value));
            acc
        });

    const STORAGE_QUERY: &str =
        "SELECT slot_index, slot_value, slot_map_root FROM account_storage WHERE commitment = ?";
    let storage_slots = conn
        .prepare(STORAGE_QUERY)?
        .query_map(params![storage_commitment.to_hex()], |row| {
            parse_storage_slot_columns(row, &mut map_entries)
        })?
        .map(|result| Ok(result?))
        .collect::<Result<Vec<SerializedStorageSlotData>, StoreError>>()?;

    let mut slots = vec![];
    for slot in storage_slots {
        let slot = match (slot.value, slot.map_root) {
            (Some(value), None) => StorageSlot::Value(Word::try_from(value)?),
            (None, Some(_)) => {
                let entries = slot
                    .map_entries
                    .into_iter()
                    .map(|(key, value)| -> Result<(Word, Word), WordError> {
                        Ok((Word::try_from(key)?, Word::try_from(value)?))
                    })
                    .collect::<Result<Vec<(Word, Word)>, _>>()?;

                StorageSlot::Map(StorageMap::with_entries(entries).map_err(|_| {
                    DeserializationError::InvalidValue("Duplicate storage map entries".to_string())
                })?)
            },
            _ => {
                return Err(StoreError::DataDeserializationError(
                    DeserializationError::InvalidValue("Invalid storage slot data".to_string()),
                ));
            },
        };
        slots.push(slot);
    }

    Ok(AccountStorage::new(slots)?)
}

#[cfg(test)]
mod tests {
    use miden_lib::account::auth::AuthRpoFalcon512;
    use miden_objects::{
        EMPTY_WORD,
        account::{AccountCode, AccountComponent},
        crypto::dsa::rpo_falcon512::PublicKey,
        testing::account_component::BASIC_WALLET_CODE,
    };

    use crate::store::sqlite_store::{account::insert_account_code, tests::create_test_store};

    #[tokio::test]
    async fn account_code_insertion_no_duplicates() {
        let store = create_test_store().await;
        let assembler = miden_lib::transaction::TransactionKernel::assembler();
        let account_component = AccountComponent::compile(BASIC_WALLET_CODE, assembler, vec![])
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
                insert_account_code(&tx, &account_code).unwrap();
                actual = tx
                    .query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0))
                    .unwrap();
                assert_eq!(actual, 1);

                // Second insertion passes but does not generate a new row
                assert!(insert_account_code(&tx, &account_code).is_ok());
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
