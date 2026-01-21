use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::account::{
    Account,
    AccountCode,
    AccountHeader,
    AccountId,
    AccountIdError,
    AccountStorage,
    Address,
    PartialAccount,
    PartialStorage,
    PartialStorageMap,
    StorageMap,
    StorageSlot,
    StorageSlotName,
    StorageSlotType,
};
use miden_client::asset::{Asset, AssetVault};
use miden_client::store::{
    AccountRecord,
    AccountRecordData,
    AccountStatus,
    AccountStorageFilter,
    StoreError,
};
use miden_client::utils::Serializable;
use miden_client::{AccountError, Word};
use miden_protocol::account::{AccountStorageHeader, StorageSlotHeader};
use miden_protocol::asset::PartialVault;

use super::WebStore;
use crate::account::js_bindings::idxdb_get_account_addresses;
use crate::account::models::AddressIdxdbObject;
use crate::account::utils::{
    insert_account_address,
    parse_account_address_idxdb_object,
    remove_account_address,
};
use crate::promise::{await_js, await_js_value};

mod js_bindings;
pub use js_bindings::{JsStorageMapEntry, JsStorageSlot, JsVaultAsset};
use js_bindings::{
    idxdb_get_account_code,
    idxdb_get_account_header,
    idxdb_get_account_header_by_commitment,
    idxdb_get_account_headers,
    idxdb_get_account_ids,
    idxdb_get_account_storage,
    idxdb_get_account_storage_maps,
    idxdb_get_account_vault_assets,
    idxdb_get_foreign_account_code,
    idxdb_lock_account,
    idxdb_undo_account_states,
    idxdb_upsert_foreign_account_code,
};

mod models;
use models::{
    AccountAssetIdxdbObject,
    AccountCodeIdxdbObject,
    AccountRecordIdxdbObject,
    AccountStorageIdxdbObject,
    ForeignAccountCodeIdxdbObject,
    StorageMapEntryIdxdbObject,
};

pub(crate) mod utils;
use utils::{
    parse_account_record_idxdb_object,
    update_account,
    upsert_account_asset_vault,
    upsert_account_code,
    upsert_account_record,
    upsert_account_storage,
};

impl WebStore {
    pub(super) async fn get_account_ids(&self) -> Result<Vec<AccountId>, StoreError> {
        let promise = idxdb_get_account_ids(self.db_id());
        let account_ids_as_strings: Vec<String> =
            await_js(promise, "failed to fetch account ids").await?;

        let native_account_ids: Vec<AccountId> = account_ids_as_strings
            .into_iter()
            .map(|id| AccountId::from_hex(&id))
            .collect::<Result<Vec<_>, AccountIdError>>()?;

        Ok(native_account_ids)
    }

    pub(super) async fn get_account_headers(
        &self,
    ) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
        let promise = idxdb_get_account_headers(self.db_id());
        let account_headers_idxdb: Vec<AccountRecordIdxdbObject> =
            await_js(promise, "failed to fetch account headers").await?;
        let account_headers: Vec<(AccountHeader, AccountStatus)> = account_headers_idxdb
            .into_iter()
            .map(parse_account_record_idxdb_object)
            .collect::<Result<Vec<_>, StoreError>>()?;

        Ok(account_headers)
    }

    pub(crate) async fn get_account_header(
        &self,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        let account_id_str = account_id.to_string();
        let promise = idxdb_get_account_header(self.db_id(), account_id_str);
        let account_header_idxdb: Option<AccountRecordIdxdbObject> =
            await_js(promise, "failed to fetch account header").await?;

        match account_header_idxdb {
            None => Ok(None),
            Some(account_header_idxdb) => {
                let parsed_account_record =
                    parse_account_record_idxdb_object(account_header_idxdb)?;

                Ok(Some(parsed_account_record))
            },
        }
    }

    pub(crate) async fn get_account_header_by_commitment(
        &self,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError> {
        let account_commitment_str = account_commitment.to_string();

        let promise = idxdb_get_account_header_by_commitment(self.db_id(), account_commitment_str);
        let account_header_idxdb: Option<AccountRecordIdxdbObject> =
            await_js(promise, "failed to fetch account header by commitment").await?;

        let account_header: Result<Option<AccountHeader>, StoreError> = account_header_idxdb
            .map_or(Ok(None), |account_record| {
                let result = parse_account_record_idxdb_object(account_record);

                result.map(|(account_header, _status)| Some(account_header))
            });

        account_header
    }

    pub(crate) async fn get_account_addresses(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<Address>, StoreError> {
        let account_id_str = account_id.to_string();

        let promise = idxdb_get_account_addresses(self.db_id(), account_id_str);

        let account_addresses_idxdb: Vec<AddressIdxdbObject> =
            await_js(promise, "failed to fetch account addresses").await?;

        account_addresses_idxdb
            .into_iter()
            .map(|obj| parse_account_address_idxdb_object(&obj).map(|(addr, _)| addr))
            .collect::<Result<Vec<Address>, StoreError>>()
    }

    pub(crate) async fn get_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        let (account_header, status) = match self.get_account_header(account_id).await? {
            None => return Ok(None),
            Some((account_header, status)) => (account_header, status),
        };
        let account_code = self.get_account_code(account_header.code_commitment()).await?;

        let account_storage = self
            .get_storage(account_header.storage_commitment(), AccountStorageFilter::All)
            .await?;
        let assets = self.get_vault_assets(account_header.vault_root()).await?;
        let account_vault = AssetVault::new(&assets)?;

        let account = Account::new(
            account_header.id(),
            account_vault,
            account_storage,
            account_code,
            account_header.nonce(),
            status.seed().copied(),
        )?;

        let addresses = self.get_account_addresses(account_id).await?;
        let account_data = AccountRecordData::Full(account);
        Ok(Some(AccountRecord::new(account_data, status, addresses)))
    }

    // NOTE: This implementation creates partial accounts with storage map entries but without
    // Merkle witnesses. Full witness support requires an SMT forest similar to the sqlite-store.
    pub(crate) async fn get_minimal_partial_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        let (account_header, status) = match self.get_account_header(account_id).await? {
            None => return Ok(None),
            Some((account_header, status)) => (account_header, status),
        };

        let partial_vault = PartialVault::new(account_header.vault_root());

        let storage_slot_headers =
            self.get_storage_slot_headers(account_header.storage_commitment()).await?;

        let mut storage_header_vec = Vec::new();

        // Collect all map roots that need to be fetched
        let map_roots_to_fetch: Vec<String> = storage_slot_headers
            .iter()
            .filter(|(_, slot_type, _)| slot_type == &StorageSlotType::Map)
            .map(|(_, _, value)| value.to_hex())
            .collect();

        // Fetch all map entries for the partial account
        let mut storage_maps_data = BTreeMap::new();
        if !map_roots_to_fetch.is_empty() {
            let promise = idxdb_get_account_storage_maps(self.db_id(), map_roots_to_fetch);
            let account_maps_idxdb: Vec<StorageMapEntryIdxdbObject> =
                await_js(promise, "failed to fetch account storage maps").await?;

            for entry in account_maps_idxdb {
                let map = storage_maps_data.entry(entry.root).or_insert_with(StorageMap::new);
                map.insert(
                    Word::try_from(entry.key.as_str())?,
                    Word::try_from(entry.value.as_str())?,
                )?;
            }
        }

        let mut maps = Vec::new();
        for (slot_name, slot_type, value) in storage_slot_headers {
            storage_header_vec.push(StorageSlotHeader::new(slot_name, slot_type, value));
            if slot_type == StorageSlotType::Map {
                let full_map =
                    storage_maps_data.get(&value.to_hex()).cloned().unwrap_or_else(StorageMap::new);
                let partial_storage_map = PartialStorageMap::new_full(full_map);
                maps.push(partial_storage_map);
            }
        }

        storage_header_vec.sort_by_key(StorageSlotHeader::id);
        let storage_header =
            AccountStorageHeader::new(storage_header_vec).map_err(StoreError::AccountError)?;
        let partial_storage =
            PartialStorage::new(storage_header, maps).map_err(StoreError::AccountError)?;

        let account_code = self.get_account_code(account_header.code_commitment()).await?;

        let partial_account = PartialAccount::new(
            account_header.id(),
            account_header.nonce(),
            account_code,
            partial_storage,
            partial_vault,
            status.seed().copied(),
        )?;

        let account_data = AccountRecordData::Partial(partial_account);
        let addresses = self.get_account_addresses(account_id).await?;
        Ok(Some(AccountRecord::new(account_data, status, addresses)))
    }

    pub(super) async fn get_account_code(&self, root: Word) -> Result<AccountCode, StoreError> {
        let root_serialized = root.to_string();

        let promise = idxdb_get_account_code(self.db_id(), root_serialized);
        let account_code_idxdb: AccountCodeIdxdbObject =
            await_js(promise, "failed to fetch account code").await?;

        let code =
            AccountCode::from_bytes(&account_code_idxdb.code).map_err(StoreError::AccountError)?;

        Ok(code)
    }

    /// Retrieves storage slot headers without fetching full map entries.
    async fn get_storage_slot_headers(
        &self,
        commitment: Word,
    ) -> Result<Vec<(StorageSlotName, StorageSlotType, Word)>, StoreError> {
        let commitment_serialized = commitment.to_string();

        let promise = idxdb_get_account_storage(self.db_id(), commitment_serialized);
        let account_storage_idxdb: Vec<AccountStorageIdxdbObject> =
            await_js(promise, "failed to fetch account storage").await?;

        if account_storage_idxdb.iter().any(|s| s.slot_name.is_empty()) {
            return Err(StoreError::DatabaseError(
                "account storage entries are missing `slotName`; clear IndexedDB and re-sync"
                    .to_string(),
            ));
        }

        account_storage_idxdb
            .into_iter()
            .map(|slot| {
                let slot_name = StorageSlotName::new(slot.slot_name).map_err(|err| {
                    StoreError::DatabaseError(format!("invalid storage slot name in db: {err}"))
                })?;
                let slot_type = StorageSlotType::try_from(slot.slot_type)?;
                let value = Word::try_from(slot.slot_value.as_str())?;
                Ok((slot_name, slot_type, value))
            })
            .collect()
    }

    pub(super) async fn get_storage(
        &self,
        commitment: Word,
        filter: AccountStorageFilter,
    ) -> Result<AccountStorage, StoreError> {
        let commitment_serialized = commitment.to_string();

        let promise = idxdb_get_account_storage(self.db_id(), commitment_serialized);
        let account_storage_idxdb: Vec<AccountStorageIdxdbObject> =
            await_js(promise, "failed to fetch account storage").await?;

        if account_storage_idxdb.iter().any(|s| s.slot_name.is_empty()) {
            return Err(StoreError::DatabaseError(
                "account storage entries are missing `slotName`; clear IndexedDB and re-sync"
                    .to_string(),
            ));
        }

        let filtered_slots: Vec<AccountStorageIdxdbObject> = match filter {
            AccountStorageFilter::All => account_storage_idxdb,
            AccountStorageFilter::Root(map_root) => {
                let map_root_hex = map_root.to_hex();
                let slot = account_storage_idxdb.into_iter().find(|s| {
                    s.slot_value == map_root_hex
                        && StorageSlotType::try_from(s.slot_type).ok() == Some(StorageSlotType::Map)
                });
                match slot {
                    Some(slot) => vec![slot],
                    None => return Err(StoreError::AccountStorageRootNotFound(map_root)),
                }
            },
            AccountStorageFilter::SlotName(name) => {
                let wanted_name = name.as_str();
                let slot =
                    account_storage_idxdb.into_iter().find(|s| s.slot_name.as_str() == wanted_name);
                match slot {
                    Some(slot) => vec![slot],
                    None => {
                        return Err(StoreError::AccountError(
                            AccountError::StorageSlotNameNotFound { slot_name: name },
                        ));
                    },
                }
            },
        };

        let mut roots = Vec::new();
        for slot in &filtered_slots {
            let slot_type = StorageSlotType::try_from(slot.slot_type)?;
            if slot_type == StorageSlotType::Map {
                roots.push(slot.slot_value.clone());
            }
        }

        let promise = idxdb_get_account_storage_maps(self.db_id(), roots);
        let account_maps_idxdb: Vec<StorageMapEntryIdxdbObject> =
            await_js(promise, "failed to fetch account storage maps").await?;

        let mut maps = BTreeMap::new();
        for entry in account_maps_idxdb {
            let map = maps.entry(entry.root).or_insert_with(StorageMap::new);
            map.insert(Word::try_from(entry.key.as_str())?, Word::try_from(entry.value.as_str())?)?;
        }

        let slots: Vec<StorageSlot> = filtered_slots
            .into_iter()
            .map(|slot| {
                let slot_name = StorageSlotName::new(slot.slot_name).map_err(|err| {
                    StoreError::DatabaseError(format!("invalid storage slot name in db: {err}"))
                })?;

                let slot_type = StorageSlotType::try_from(slot.slot_type)?;

                Ok(match slot_type {
                    StorageSlotType::Value => {
                        StorageSlot::with_value(slot_name, Word::try_from(slot.slot_value.as_str())?)
                    },
                    StorageSlotType::Map => {
                        let map = maps.remove(&slot.slot_value).unwrap_or_else(StorageMap::new);
                        if map.root().to_hex() != slot.slot_value {
                            return Err(StoreError::DatabaseError(format!(
                                "incomplete storage map for slot {slot_name} (expected root {}, got {})",
                                slot.slot_value,
                                map.root().to_hex(),
                            )));
                        }
                        StorageSlot::with_map(slot_name, map)
                    },
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?;

        Ok(AccountStorage::new(slots)?)
    }

    pub(super) async fn get_vault_assets(&self, root: Word) -> Result<Vec<Asset>, StoreError> {
        let promise = idxdb_get_account_vault_assets(self.db_id(), root.to_hex());
        let vault_assets_idxdb: Vec<AccountAssetIdxdbObject> =
            await_js(promise, "failed to fetch vault assets").await?;

        let assets = vault_assets_idxdb
            .into_iter()
            .map(|asset| {
                let word = Word::try_from(&asset.asset)?;
                Ok(Asset::try_from(word)?)
            })
            .collect::<Result<Vec<_>, StoreError>>()?;

        Ok(assets)
    }

    pub(crate) async fn insert_account(
        &self,
        account: &Account,
        initial_address: Address,
    ) -> Result<(), StoreError> {
        upsert_account_code(self.db_id(), account.code()).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to insert account code: {js_error:?}",))
        })?;

        upsert_account_storage(self.db_id(), account.storage())
            .await
            .map_err(|js_error| {
                StoreError::DatabaseError(format!("failed to insert account storage:{js_error:?}",))
            })?;

        upsert_account_asset_vault(self.db_id(), account.vault())
            .await
            .map_err(|js_error| {
                StoreError::DatabaseError(format!("failed to insert account vault:{js_error:?}",))
            })?;

        upsert_account_record(self.db_id(), account).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to insert account record: {js_error:?}",))
        })?;

        insert_account_address(self.db_id(), &account.id(), initial_address)
            .await
            .map_err(|js_error| {
                StoreError::DatabaseError(format!(
                    "failed to insert account addresses: {js_error:?}",
                ))
            })?;

        Ok(())
    }

    pub(crate) async fn update_account(
        &self,
        new_account_state: &Account,
    ) -> Result<(), StoreError> {
        let account_id_str = new_account_state.id().to_string();
        let promise = idxdb_get_account_header(self.db_id(), account_id_str);
        let account_header_idxdb: Option<AccountRecordIdxdbObject> =
            await_js(promise, "failed to fetch account header").await?;

        if account_header_idxdb.is_none() {
            return Err(StoreError::AccountDataNotFound(new_account_state.id()));
        }

        update_account(self.db_id(), new_account_state)
            .await
            .map_err(|_| StoreError::DatabaseError("failed to update account".to_string()))
    }

    pub(crate) async fn get_account_vault(
        &self,
        account_id: AccountId,
    ) -> Result<AssetVault, StoreError> {
        let account_header = self
            .get_account_header(account_id)
            .await?
            .ok_or(StoreError::AccountDataNotFound(account_id))?
            .0;

        let assets = self.get_vault_assets(account_header.vault_root()).await?;
        Ok(AssetVault::new(&assets)?)
    }

    pub(crate) async fn get_account_storage(
        &self,
        account_id: AccountId,
        filter: AccountStorageFilter,
    ) -> Result<AccountStorage, StoreError> {
        let account_header = self
            .get_account_header(account_id)
            .await?
            .ok_or(StoreError::AccountDataNotFound(account_id))?
            .0;

        self.get_storage(account_header.storage_commitment(), filter).await
    }

    pub(crate) async fn upsert_foreign_account_code(
        &self,
        account_id: AccountId,
        code: AccountCode,
    ) -> Result<(), StoreError> {
        let root = code.commitment().to_string();
        let code = code.to_bytes();
        let account_id = account_id.to_string();

        let promise = idxdb_upsert_foreign_account_code(self.db_id(), account_id, code, root);
        await_js_value(promise, "failed to upsert foreign account code").await?;

        Ok(())
    }

    pub(crate) async fn get_foreign_account_code(
        &self,
        account_ids: Vec<AccountId>,
    ) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
        let account_ids = account_ids.iter().map(ToString::to_string).collect::<Vec<_>>();
        let promise = idxdb_get_foreign_account_code(self.db_id(), account_ids);
        let foreign_account_code_idxdb: Option<Vec<ForeignAccountCodeIdxdbObject>> =
            await_js(promise, "failed to fetch foreign account code").await?;

        let foreign_account_code: BTreeMap<AccountId, AccountCode> = foreign_account_code_idxdb
            .unwrap_or_default()
            .into_iter()
            .map(|idxdb_object| {
                let account_id = AccountId::from_hex(&idxdb_object.account_id)
                    .map_err(StoreError::AccountIdError)?;
                let code = AccountCode::from_bytes(&idxdb_object.code)
                    .map_err(StoreError::AccountError)?;

                Ok((account_id, code))
            })
            .collect::<Result<BTreeMap<AccountId, AccountCode>, StoreError>>()?;

        Ok(foreign_account_code)
    }

    pub(crate) async fn undo_account_states(
        &self,
        account_states: &[Word],
    ) -> Result<(), StoreError> {
        let account_commitments =
            account_states.iter().map(ToString::to_string).collect::<Vec<_>>();
        let promise = idxdb_undo_account_states(self.db_id(), account_commitments);
        await_js_value(promise, "failed to undo account states").await?;

        Ok(())
    }

    /// Locks the account if the mismatched digest doesn't belong to a previous account state (stale
    /// data).
    pub(crate) async fn lock_account_on_unexpected_commitment(
        &self,
        account_id: &AccountId,
        mismatched_digest: &Word,
    ) -> Result<(), StoreError> {
        // Mismatched digests may be due to stale network data. If the mismatched digest is
        // tracked in the db and corresponds to the mismatched account, it means we
        // got a past update and shouldn't lock the account.
        if let Some(account) = self.get_account_header_by_commitment(*mismatched_digest).await?
            && account.id() == *account_id
        {
            return Ok(());
        }

        let account_id_str = account_id.to_string();
        let promise = idxdb_lock_account(self.db_id(), account_id_str);
        await_js_value(promise, "failed to lock account").await?;

        Ok(())
    }

    pub(crate) async fn insert_address(
        &self,
        address: Address,
        account_id: &AccountId,
    ) -> Result<(), StoreError> {
        insert_account_address(self.db_id(), account_id, address)
            .await
            .map_err(|js_error| {
                StoreError::DatabaseError(format!(
                    "failed to insert account addresses: {js_error:?}",
                ))
            })?;

        Ok(())
    }

    pub(crate) async fn remove_address(&self, address: Address) -> Result<(), StoreError> {
        remove_account_address(self.db_id(), address).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to remove account address: {js_error:?}"))
        })
    }
}
