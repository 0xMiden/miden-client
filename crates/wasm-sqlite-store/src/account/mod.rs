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

use super::WasmSqliteStore;
use crate::account::utils::{
    insert_account_address,
    parse_account_address_object,
    parse_account_record_object,
    remove_account_address,
    update_account,
    upsert_account_asset_vault,
    upsert_account_code,
    upsert_account_record,
    upsert_account_storage,
};

mod js_bindings;
use js_bindings::{
    js_get_account_addresses,
    js_get_account_code,
    js_get_account_header,
    js_get_account_header_by_commitment,
    js_get_account_headers,
    js_get_account_ids,
    js_get_account_storage,
    js_get_account_storage_maps,
    js_get_account_vault_assets,
    js_get_foreign_account_code,
    js_lock_account,
    js_undo_account_states,
    js_upsert_foreign_account_code,
};

mod models;
use models::{
    AccountAssetObject,
    AccountCodeObject,
    AccountRecordObject,
    AccountStorageObject,
    AddressObject,
    ForeignAccountCodeObject,
    StorageMapEntryObject,
};

pub(crate) mod utils;

impl WasmSqliteStore {
    #[allow(clippy::unused_async)]
    pub(super) async fn get_account_ids(&self) -> Result<Vec<AccountId>, StoreError> {
        let js_value = js_get_account_ids(self.db_id());
        let account_ids_as_strings: Vec<String> = serde_wasm_bindgen::from_value(js_value)
            .map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize account ids: {err:?}"))
            })?;

        let native_account_ids: Vec<AccountId> = account_ids_as_strings
            .into_iter()
            .map(|id| AccountId::from_hex(&id))
            .collect::<Result<Vec<_>, AccountIdError>>()?;

        Ok(native_account_ids)
    }

    #[allow(clippy::unused_async)]
    pub(super) async fn get_account_headers(
        &self,
    ) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
        let js_value = js_get_account_headers(self.db_id());
        let account_headers: Vec<AccountRecordObject> = serde_wasm_bindgen::from_value(js_value)
            .map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize account headers: {err:?}"))
            })?;
        account_headers
            .into_iter()
            .map(parse_account_record_object)
            .collect::<Result<Vec<_>, StoreError>>()
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_account_header(
        &self,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        let account_id_str = account_id.to_string();
        let js_value = js_get_account_header(self.db_id(), account_id_str);
        if js_value.is_null() || js_value.is_undefined() {
            return Ok(None);
        }

        let account_header: AccountRecordObject = serde_wasm_bindgen::from_value(js_value)
            .map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize account header: {err:?}"))
            })?;

        Ok(Some(parse_account_record_object(account_header)?))
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_account_header_by_commitment(
        &self,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError> {
        let commitment_str = account_commitment.to_string();
        let js_value = js_get_account_header_by_commitment(self.db_id(), commitment_str);
        if js_value.is_null() || js_value.is_undefined() {
            return Ok(None);
        }

        let account_header: AccountRecordObject = serde_wasm_bindgen::from_value(js_value)
            .map_err(|err| {
                StoreError::DatabaseError(format!(
                    "failed to deserialize account header by commitment: {err:?}"
                ))
            })?;

        let (header, _status) = parse_account_record_object(account_header)?;
        Ok(Some(header))
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_account_addresses(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<Address>, StoreError> {
        let account_id_str = account_id.to_string();
        let js_value = js_get_account_addresses(self.db_id(), account_id_str);
        let addresses: Vec<AddressObject> =
            serde_wasm_bindgen::from_value(js_value).map_err(|err| {
                StoreError::DatabaseError(format!(
                    "failed to deserialize account addresses: {err:?}"
                ))
            })?;

        addresses
            .iter()
            .map(|obj| parse_account_address_object(obj).map(|(addr, _)| addr))
            .collect::<Result<Vec<Address>, StoreError>>()
    }

    pub(crate) async fn get_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        let (account_header, status) = match self.get_account_header(account_id).await? {
            None => return Ok(None),
            Some((header, status)) => (header, status),
        };
        let account_code =
            self.get_account_code_by_commitment(account_header.code_commitment()).await?;

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

        let account_data = AccountRecordData::Full(account);
        Ok(Some(AccountRecord::new(account_data, status)))
    }

    pub(crate) async fn get_minimal_partial_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        let (account_header, status) = match self.get_account_header(account_id).await? {
            None => return Ok(None),
            Some((header, status)) => (header, status),
        };
        let account_code =
            self.get_account_code_by_commitment(account_header.code_commitment()).await?;

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

        let account_data = AccountRecordData::Partial((&account).into());
        Ok(Some(AccountRecord::new(account_data, status)))
    }

    #[allow(clippy::unused_async)]
    pub(super) async fn get_account_code_by_commitment(
        &self,
        commitment: Word,
    ) -> Result<AccountCode, StoreError> {
        let commitment_str = commitment.to_string();
        let js_value = js_get_account_code(self.db_id(), commitment_str);
        if js_value.is_null() || js_value.is_undefined() {
            return Err(StoreError::DatabaseError("account code not found".to_string()));
        }

        let account_code_obj: AccountCodeObject = serde_wasm_bindgen::from_value(js_value)
            .map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize account code: {err:?}"))
            })?;

        AccountCode::from_bytes(&account_code_obj.code).map_err(StoreError::AccountError)
    }

    #[allow(clippy::unused_async, clippy::too_many_lines)]
    pub(super) async fn get_storage(
        &self,
        commitment: Word,
        filter: AccountStorageFilter,
    ) -> Result<AccountStorage, StoreError> {
        let commitment_str = commitment.to_string();
        let js_value = js_get_account_storage(self.db_id(), commitment_str);
        let account_storage: Vec<AccountStorageObject> = serde_wasm_bindgen::from_value(js_value)
            .map_err(|err| {
            StoreError::DatabaseError(format!("failed to deserialize account storage: {err:?}"))
        })?;

        if account_storage.iter().any(|s| s.slot_name.is_empty()) {
            return Err(StoreError::DatabaseError(
                "account storage entries are missing `slotName`; clear database and re-sync"
                    .to_string(),
            ));
        }

        let filtered_slots: Vec<AccountStorageObject> = match filter {
            AccountStorageFilter::All => account_storage,
            AccountStorageFilter::Root(map_root) => {
                let map_root_hex = map_root.to_hex();
                let slot = account_storage.into_iter().find(|s| {
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
                    account_storage.into_iter().find(|s| s.slot_name.as_str() == wanted_name);
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

        let js_maps = js_get_account_storage_maps(self.db_id(), roots);
        let storage_map_entries: Vec<StorageMapEntryObject> =
            serde_wasm_bindgen::from_value(js_maps).map_err(|err| {
                StoreError::DatabaseError(format!(
                    "failed to deserialize storage map entries: {err:?}"
                ))
            })?;

        let mut maps = BTreeMap::new();
        for entry in storage_map_entries {
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

    #[allow(clippy::unused_async)]
    pub(super) async fn get_vault_assets(&self, root: Word) -> Result<Vec<Asset>, StoreError> {
        let js_value = js_get_account_vault_assets(self.db_id(), root.to_hex());
        let vault_assets: Vec<AccountAssetObject> = serde_wasm_bindgen::from_value(js_value)
            .map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize vault assets: {err:?}"))
            })?;

        vault_assets
            .into_iter()
            .map(|asset| {
                let word = Word::try_from(&asset.asset)?;
                Ok(Asset::try_from(word)?)
            })
            .collect::<Result<Vec<_>, StoreError>>()
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn insert_account(
        &self,
        account: &Account,
        initial_address: Address,
    ) -> Result<(), StoreError> {
        upsert_account_code(self.db_id(), account.code());
        upsert_account_storage(self.db_id(), account.storage());
        upsert_account_asset_vault(self.db_id(), account.vault());
        upsert_account_record(self.db_id(), account);
        insert_account_address(self.db_id(), &account.id(), initial_address);
        Ok(())
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn update_account(
        &self,
        new_account_state: &Account,
    ) -> Result<(), StoreError> {
        let account_id_str = new_account_state.id().to_string();
        let js_value = js_get_account_header(self.db_id(), account_id_str);
        if js_value.is_null() || js_value.is_undefined() {
            return Err(StoreError::AccountDataNotFound(new_account_state.id()));
        }

        update_account(self.db_id(), new_account_state);
        Ok(())
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

    #[allow(clippy::unused_async)]
    pub(crate) async fn upsert_foreign_account_code(
        &self,
        account_id: AccountId,
        code: AccountCode,
    ) -> Result<(), StoreError> {
        let commitment = code.commitment().to_string();
        let code_bytes = code.to_bytes();
        let account_id_str = account_id.to_string();

        js_upsert_foreign_account_code(self.db_id(), account_id_str, code_bytes, commitment);
        Ok(())
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_foreign_account_code(
        &self,
        account_ids: Vec<AccountId>,
    ) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
        let account_id_strings = account_ids.iter().map(ToString::to_string).collect::<Vec<_>>();
        let js_value = js_get_foreign_account_code(self.db_id(), account_id_strings);

        let foreign_code: Vec<ForeignAccountCodeObject> = serde_wasm_bindgen::from_value(js_value)
            .map_err(|err| {
                StoreError::DatabaseError(format!(
                    "failed to deserialize foreign account code: {err:?}"
                ))
            })?;

        foreign_code
            .into_iter()
            .map(|obj| {
                let account_id =
                    AccountId::from_hex(&obj.account_id).map_err(StoreError::AccountIdError)?;
                let code = AccountCode::from_bytes(&obj.code).map_err(StoreError::AccountError)?;
                Ok((account_id, code))
            })
            .collect::<Result<BTreeMap<AccountId, AccountCode>, StoreError>>()
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn undo_account_states(
        &self,
        account_states: &[Word],
    ) -> Result<(), StoreError> {
        let account_commitments =
            account_states.iter().map(ToString::to_string).collect::<Vec<_>>();
        js_undo_account_states(self.db_id(), account_commitments);
        Ok(())
    }

    pub(crate) async fn lock_account_on_unexpected_commitment(
        &self,
        account_id: &AccountId,
        mismatched_digest: &Word,
    ) -> Result<(), StoreError> {
        if let Some(account) = self.get_account_header_by_commitment(*mismatched_digest).await?
            && account.id() == *account_id
        {
            return Ok(());
        }

        let account_id_str = account_id.to_string();
        js_lock_account(self.db_id(), account_id_str);
        Ok(())
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn insert_address_record(
        &self,
        address: Address,
        account_id: &AccountId,
    ) -> Result<(), StoreError> {
        insert_account_address(self.db_id(), account_id, address);
        Ok(())
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn remove_address_record(&self, address: Address) -> Result<(), StoreError> {
        remove_account_address(self.db_id(), address);
        Ok(())
    }
}
