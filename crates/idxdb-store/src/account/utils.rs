use alloc::string::ToString;
use alloc::vec::Vec;

use miden_client::account::{
    Account,
    AccountCode,
    AccountHeader,
    AccountId,
    AccountStorage,
    Address,
    StorageSlotContent,
};
use miden_client::asset::AssetVault;
use miden_client::store::{AccountStatus, StoreError};
use miden_client::utils::{Deserializable, Serializable};
use miden_client::{Felt, Word};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use super::js_bindings::{
    JsStorageMapEntry,
    JsStorageSlot,
    JsVaultAsset,
    idxdb_upsert_account_code,
    idxdb_upsert_account_record,
    idxdb_upsert_account_storage,
    idxdb_upsert_storage_map_entries,
    idxdb_upsert_vault_assets,
};
use crate::account::js_bindings::idxdb_insert_account_address;
use crate::account::models::{AccountRecordIdxdbObject, AddressIdxdbObject};

pub async fn upsert_account_code(db_id: &str, account_code: &AccountCode) -> Result<(), JsValue> {
    let root = account_code.commitment().to_string();
    let code = account_code.to_bytes();

    let promise = idxdb_upsert_account_code(db_id, root, code);
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn upsert_account_storage(
    db_id: &str,
    account_id: &AccountId,
    nonce: u64,
    account_storage: &AccountStorage,
) -> Result<(), JsValue> {
    let mut slots = vec![];
    let mut maps = vec![];
    for slot in account_storage.slots() {
        slots.push(JsStorageSlot::from_slot(slot, account_id, nonce));
        if let StorageSlotContent::Map(map) = slot.content() {
            maps.extend(JsStorageMapEntry::from_map(map, account_id, nonce, slot.name().as_str()));
        }
    }

    JsFuture::from(idxdb_upsert_account_storage(db_id, slots)).await?;
    JsFuture::from(idxdb_upsert_storage_map_entries(db_id, maps)).await?;

    Ok(())
}

pub async fn upsert_account_asset_vault(
    db_id: &str,
    account_id: &AccountId,
    nonce: u64,
    asset_vault: &AssetVault,
) -> Result<(), JsValue> {
    let js_assets: Vec<JsVaultAsset> = asset_vault
        .assets()
        .map(|asset| JsVaultAsset::from_asset(&asset, account_id, nonce))
        .collect();

    let promise = idxdb_upsert_vault_assets(db_id, js_assets);
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn upsert_account_record(db_id: &str, account: &Account) -> Result<(), JsValue> {
    let account_id_str = account.id().to_string();
    let code_root = account.code().commitment().to_string();
    let storage_root = account.storage().to_commitment().to_string();
    let vault_root = account.vault().root().to_string();
    let committed = account.is_public();
    let nonce = account.nonce().to_string();
    let account_seed = account.seed().map(|seed| seed.to_bytes());
    let commitment = account.commitment().to_string();

    let promise = idxdb_upsert_account_record(
        db_id,
        account_id_str,
        code_root,
        storage_root,
        vault_root,
        nonce,
        committed,
        commitment,
        account_seed,
    );
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn insert_account_address(
    db_id: &str,
    account_id: &AccountId,
    address: Address,
) -> Result<(), JsValue> {
    let account_id_str = account_id.to_string();
    let serialized_address = address.to_bytes();
    let promise = idxdb_insert_account_address(db_id, account_id_str, serialized_address);
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn remove_account_address(db_id: &str, address: Address) -> Result<(), JsValue> {
    let serialized_address = address.to_bytes();
    let promise = crate::account::js_bindings::idxdb_remove_account_address(
        db_id,
        serialized_address.clone(),
    );
    JsFuture::from(promise).await?;

    Ok(())
}

pub fn parse_account_record_idxdb_object(
    account_header_idxdb: AccountRecordIdxdbObject,
) -> Result<(AccountHeader, AccountStatus), StoreError> {
    let native_account_id: AccountId = AccountId::from_hex(&account_header_idxdb.id)?;
    let native_nonce: u64 = account_header_idxdb
        .nonce
        .parse::<u64>()
        .map_err(|err| StoreError::ParsingError(err.to_string()))?;
    let account_seed = account_header_idxdb
        .account_seed
        .map(|seed| Word::read_from_bytes(&seed))
        .transpose()?;

    let account_header = AccountHeader::new(
        native_account_id,
        Felt::new(native_nonce),
        Word::try_from(&account_header_idxdb.vault_root)?,
        Word::try_from(&account_header_idxdb.storage_root)?,
        Word::try_from(&account_header_idxdb.code_root)?,
    );

    let status = match (account_seed, account_header_idxdb.locked) {
        (seed, true) => AccountStatus::Locked { seed },
        (Some(seed), _) => AccountStatus::New { seed },
        _ => AccountStatus::Tracked,
    };

    Ok((account_header, status))
}

pub fn parse_account_address_idxdb_object(
    account_address_idxdb: &AddressIdxdbObject,
) -> Result<(Address, AccountId), StoreError> {
    let native_account_id: AccountId = AccountId::from_hex(&account_address_idxdb.id)?;

    let address = Address::read_from_bytes(&account_address_idxdb.address)?;

    Ok((address, native_account_id))
}

pub async fn update_account(db_id: &str, new_account_state: &Account) -> Result<(), JsValue> {
    let account_id = &new_account_state.id();
    let nonce = new_account_state.nonce().as_int();
    upsert_account_storage(db_id, account_id, nonce, new_account_state.storage()).await?;
    upsert_account_asset_vault(db_id, account_id, nonce, new_account_state.vault()).await?;
    upsert_account_record(db_id, new_account_state).await
}
