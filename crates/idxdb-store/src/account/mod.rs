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
    StorageSlotType,
};
use miden_client::asset::{Asset, AssetVault};
use miden_client::store::{AccountRecord, AccountStatus, StoreError};
use miden_client::utils::Serializable;
use miden_client::{Felt, Word};

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
        let promise = idxdb_get_account_ids();
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
        let promise = idxdb_get_account_headers();
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
        let promise = idxdb_get_account_header(account_id_str);
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

        let promise = idxdb_get_account_header_by_commitment(account_commitment_str);
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

        let promise = idxdb_get_account_addresses(account_id_str);

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

        let account_storage = self.get_storage(account_header.storage_commitment(), None).await?;
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

        Ok(Some(AccountRecord::new(account, status, addresses)))
    }

    pub(super) async fn get_account_code(&self, root: Word) -> Result<AccountCode, StoreError> {
        let root_serialized = root.to_string();

        let promise = idxdb_get_account_code(root_serialized);
        let account_code_idxdb: AccountCodeIdxdbObject =
            await_js(promise, "failed to fetch account code").await?;

        let code =
            AccountCode::from_bytes(&account_code_idxdb.code).map_err(StoreError::AccountError)?;

        Ok(code)
    }

    pub(super) async fn get_storage(
        &self,
        commitment: Word,
        map_root: Option<Word>,
    ) -> Result<AccountStorage, StoreError> {
        let commitment_serialized = commitment.to_string();

        let promise = idxdb_get_account_storage(commitment_serialized);
        let account_storage_idxdb: Vec<AccountStorageIdxdbObject> =
            await_js(promise, "failed to fetch account storage").await?;

        let roots = match map_root {
            Some(map_root) => {
                if !account_storage_idxdb.iter().any(|a| a.slot_value == map_root.to_hex()) {
                    return Err(StoreError::AccountStorageNotFound(map_root));
                }
                vec![map_root.to_hex()]
            },
            None => account_storage_idxdb
                .iter()
                .map(|s| s.slot_value.clone())
                .collect::<Vec<String>>(),
        };

        let promise = idxdb_get_account_storage_maps(roots);
        let account_maps_idxdb: Vec<StorageMapEntryIdxdbObject> =
            await_js(promise, "failed to fetch account storage maps").await?;

        let mut maps = BTreeMap::new();
        for entry in account_maps_idxdb {
            let map = maps.entry(entry.root).or_insert_with(StorageMap::new);
            map.insert(Word::try_from(entry.key)?, Word::try_from(entry.value)?)?;
        }

        let slots: Vec<StorageSlot> = account_storage_idxdb
            .into_iter()
            .map(|slot| {
                let slot_type = StorageSlotType::try_from(Felt::new(slot.slot_type))
                    .map_err(StoreError::DatabaseError)?;
                Ok(match slot_type {
                    StorageSlotType::Value => StorageSlot::Value(Word::try_from(&slot.slot_value)?),
                    StorageSlotType::Map => {
                        StorageSlot::Map(maps.remove(&slot.slot_value).unwrap_or_default())
                    },
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?;

        Ok(AccountStorage::new(slots)?)
    }

    pub(super) async fn get_vault_assets(&self, root: Word) -> Result<Vec<Asset>, StoreError> {
        let promise = idxdb_get_account_vault_assets(root.to_hex());
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
        upsert_account_code(account.code()).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to insert account code: {js_error:?}",))
        })?;

        upsert_account_storage(account.storage()).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to insert account storage:{js_error:?}",))
        })?;

        upsert_account_asset_vault(account.vault()).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to insert account vault:{js_error:?}",))
        })?;

        upsert_account_record(account).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to insert account record: {js_error:?}",))
        })?;

        insert_account_address(&account.id(), initial_address)
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
        let promise = idxdb_get_account_header(account_id_str);
        let account_header_idxdb: Option<AccountRecordIdxdbObject> =
            await_js(promise, "failed to fetch account header").await?;

        if account_header_idxdb.is_none() {
            return Err(StoreError::AccountDataNotFound(new_account_state.id()));
        }

        update_account(new_account_state)
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
        map_root: Option<Word>,
    ) -> Result<AccountStorage, StoreError> {
        let account_header = self
            .get_account_header(account_id)
            .await?
            .ok_or(StoreError::AccountDataNotFound(account_id))?
            .0;

        self.get_storage(account_header.storage_commitment(), map_root).await
    }

    pub(crate) async fn upsert_foreign_account_code(
        &self,
        account_id: AccountId,
        code: AccountCode,
    ) -> Result<(), StoreError> {
        let root = code.commitment().to_string();
        let code = code.to_bytes();
        let account_id = account_id.to_string();

        let promise = idxdb_upsert_foreign_account_code(account_id, code, root);
        await_js_value(promise, "failed to upsert foreign account code").await?;

        Ok(())
    }

    pub(crate) async fn get_foreign_account_code(
        &self,
        account_ids: Vec<AccountId>,
    ) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
        let account_ids = account_ids.iter().map(ToString::to_string).collect::<Vec<_>>();
        let promise = idxdb_get_foreign_account_code(account_ids);
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
        let promise = idxdb_undo_account_states(account_commitments);
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
        let promise = idxdb_lock_account(account_id_str);
        await_js_value(promise, "failed to lock account").await?;

        Ok(())
    }

    pub(crate) async fn insert_address(
        &self,
        address: Address,
        account_id: &AccountId,
    ) -> Result<(), StoreError> {
        insert_account_address(account_id, address).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to insert account addresses: {js_error:?}",))
        })?;

        Ok(())
    }

    pub(crate) async fn remove_address(&self, address: Address) -> Result<(), StoreError> {
        remove_account_address(address).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to remove account address: {js_error:?}"))
        })
    }
}
