use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_objects::account::{Account, AccountCode, AccountHeader, AccountId, AccountStorage};
use miden_objects::address::AccountIdAddress;
use miden_objects::asset::{Asset, AssetVault};
use miden_objects::utils::Deserializable;
use miden_objects::{Felt, Word};
use miden_tx::utils::Serializable;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use super::js_bindings::{
    idxdb_get_account_auth_by_pub_key,
    idxdb_insert_account_asset_vault,
    idxdb_insert_account_auth,
    idxdb_insert_account_code,
    idxdb_insert_account_record,
    idxdb_insert_account_storage,
};
use super::models::{AccountAuthIdxdbObject, AccountRecordIdxdbObject};
use crate::store::web_store::account::js_bindings::idxdb_insert_account_address;
use crate::store::web_store::account::models::AccountIdAddressIdxdbObject;
use crate::store::{AccountStatus, StoreError};

pub async fn insert_account_code(account_code: &AccountCode) -> Result<(), JsValue> {
    let root = account_code.commitment().to_string();
    let code = account_code.to_bytes();

    let promise = idxdb_insert_account_code(root, code);
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn insert_account_storage(account_storage: &AccountStorage) -> Result<(), JsValue> {
    let root = account_storage.commitment().to_string();

    let storage = account_storage.to_bytes();

    let promise = idxdb_insert_account_storage(root, storage);
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn insert_account_asset_vault(asset_vault: &AssetVault) -> Result<(), JsValue> {
    let commitment = asset_vault.root().to_string();
    let assets = asset_vault.assets().collect::<Vec<Asset>>().to_bytes();

    let promise = idxdb_insert_account_asset_vault(commitment, assets);
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn insert_account_auth(pub_key: String, secret_key: String) -> Result<(), JsValue> {
    let promise = idxdb_insert_account_auth(pub_key, secret_key);
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn get_account_auth_by_pub_key(pub_key: String) -> Result<String, JsValue> {
    let promise = idxdb_get_account_auth_by_pub_key(pub_key.clone());
    let js_secret_key = JsFuture::from(promise).await?;

    let account_auth_idxdb: Option<AccountAuthIdxdbObject> =
        from_value(js_secret_key).map_err(|err| {
            JsValue::from_str(&format!("error: failed to deserialize secret key: {err}"))
        })?;

    match account_auth_idxdb {
        Some(account_auth) => Ok(account_auth.secret_key),
        None => Err(JsValue::from_str(&format!("pub key {pub_key} not found in the store"))),
    }
}

pub async fn insert_account_record(
    account: &Account,
    account_seed: Option<Word>,
) -> Result<(), JsValue> {
    let account_id_str = account.id().to_string();
    let code_root = account.code().commitment().to_string();
    let storage_root = account.storage().commitment().to_string();
    let vault_root = account.vault().root().to_string();
    let committed = account.is_public();
    let nonce = account.nonce().to_string();
    let account_seed = account_seed.map(|seed| seed.to_bytes());
    let commitment = account.commitment().to_string();

    let promise = idxdb_insert_account_record(
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

pub async fn insert_account_addresses(
    account: &Account,
    addresses: Vec<AccountIdAddress>,
) -> Result<(), JsValue> {
    for address in addresses {
        let account_id_str = account.id().to_string();
        let serialized_address: [u8; AccountIdAddress::SERIALIZED_SIZE] = address.into();
        let promise = idxdb_insert_account_address(account_id_str, serialized_address.to_vec());
        JsFuture::from(promise).await?;
    }

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
        (_, true) => AccountStatus::Locked,
        (Some(seed), _) => AccountStatus::New { seed },
        _ => AccountStatus::Tracked,
    };

    Ok((account_header, status))
}

pub fn parse_account_address_idxdb_object(
    account_address_idxdb: AccountIdAddressIdxdbObject,
) -> Result<(AccountIdAddress, AccountId), StoreError> {
    let native_account_id: AccountId = AccountId::from_hex(&account_address_idxdb.id)?;

    let address: [u8; AccountIdAddress::SERIALIZED_SIZE] = account_address_idxdb
        .address
        .try_into()
        .map_err(|_| StoreError::ParsingError("invalid address length".to_string()))?;
    let native_address = AccountIdAddress::try_from(address).map_err(|err| {
        StoreError::ParsingError(format!("failed to parse address from bytes: {err}"))
    })?;
    Ok((native_address, native_account_id))
}

pub async fn update_account(new_account_state: &Account) -> Result<(), JsValue> {
    insert_account_storage(new_account_state.storage()).await?;
    insert_account_asset_vault(new_account_state.vault()).await?;
    insert_account_record(new_account_state, None).await
}
