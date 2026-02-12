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
use serde::Serialize;

use super::js_bindings::{
    js_insert_account_address,
    js_upsert_account_code,
    js_upsert_account_record,
    js_upsert_account_storage,
    js_upsert_storage_map_entries,
    js_upsert_vault_assets,
};
use crate::account::models::{AccountRecordObject, AddressObject};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsStorageSlot {
    commitment: String,
    slot_name: String,
    slot_value: String,
    slot_type: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsStorageMapEntry {
    root: String,
    key: String,
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsVaultAsset {
    root: String,
    vault_key: String,
    faucet_id_prefix: String,
    asset: String,
}

pub fn upsert_account_code(db_id: &str, account_code: &AccountCode) {
    let commitment = account_code.commitment().to_string();
    let code = account_code.to_bytes();
    js_upsert_account_code(db_id, commitment, code);
}

pub fn upsert_account_storage(db_id: &str, account_storage: &AccountStorage) {
    let mut slots = vec![];
    let mut maps = vec![];

    for slot in account_storage.slots() {
        slots.push(JsStorageSlot {
            commitment: account_storage.to_commitment().to_hex(),
            slot_name: slot.name().to_string(),
            slot_value: slot.value().to_hex(),
            slot_type: slot.slot_type().to_bytes()[0],
        });

        if let StorageSlotContent::Map(map) = slot.content() {
            for (key, value) in map.entries() {
                maps.push(JsStorageMapEntry {
                    root: map.root().to_hex(),
                    key: key.to_hex(),
                    value: value.to_hex(),
                });
            }
        }
    }

    let slots_js = serde_wasm_bindgen::to_value(&slots).expect("serialization should succeed");
    js_upsert_account_storage(db_id, slots_js);

    if !maps.is_empty() {
        let maps_js = serde_wasm_bindgen::to_value(&maps).expect("serialization should succeed");
        js_upsert_storage_map_entries(db_id, maps_js);
    }
}

pub fn upsert_account_asset_vault(db_id: &str, asset_vault: &AssetVault) {
    let js_assets: Vec<JsVaultAsset> = asset_vault
        .assets()
        .map(|asset| JsVaultAsset {
            root: asset_vault.root().to_hex(),
            vault_key: Word::from(asset.vault_key()).to_hex(),
            faucet_id_prefix: asset.faucet_id_prefix().to_hex(),
            asset: Word::from(asset).to_hex(),
        })
        .collect();

    let assets_js = serde_wasm_bindgen::to_value(&js_assets).expect("serialization should succeed");
    js_upsert_vault_assets(db_id, assets_js);
}

pub fn upsert_account_record(db_id: &str, account: &Account) {
    let account_id_str = account.id().to_string();
    let code_commitment = account.code().commitment().to_string();
    let storage_commitment = account.storage().to_commitment().to_string();
    let vault_root = account.vault().root().to_string();
    let committed = account.is_public();
    let nonce = account.nonce().to_string();
    let account_seed = account.seed().map(|seed| seed.to_bytes());
    let commitment = account.commitment().to_string();

    js_upsert_account_record(
        db_id,
        account_id_str,
        code_commitment,
        storage_commitment,
        vault_root,
        nonce,
        committed,
        commitment,
        account_seed,
    );
}

#[allow(clippy::needless_pass_by_value)]
pub fn insert_account_address(db_id: &str, account_id: &AccountId, address: Address) {
    let account_id_str = account_id.to_string();
    let serialized_address = address.to_bytes();
    js_insert_account_address(db_id, account_id_str, serialized_address);
}

#[allow(clippy::needless_pass_by_value)]
pub fn remove_account_address(db_id: &str, address: Address) {
    let serialized_address = address.to_bytes();
    crate::account::js_bindings::js_remove_account_address(db_id, serialized_address);
}

pub fn parse_account_record_object(
    account_header: AccountRecordObject,
) -> Result<(AccountHeader, AccountStatus), StoreError> {
    let native_account_id: AccountId = AccountId::from_hex(&account_header.id)?;
    let native_nonce: u64 = account_header
        .nonce
        .parse::<u64>()
        .map_err(|err| StoreError::ParsingError(err.to_string()))?;
    let account_seed = account_header
        .account_seed
        .map(|seed| Word::read_from_bytes(&seed))
        .transpose()?;

    let header = AccountHeader::new(
        native_account_id,
        Felt::new(native_nonce),
        Word::try_from(&account_header.vault_root)?,
        Word::try_from(&account_header.storage_commitment)?,
        Word::try_from(&account_header.code_commitment)?,
    );

    let status = match (account_seed, account_header.locked) {
        (seed, true) => AccountStatus::Locked { seed },
        (Some(seed), _) => AccountStatus::New { seed },
        _ => AccountStatus::Tracked,
    };

    Ok((header, status))
}

pub fn parse_account_address_object(
    address_obj: &AddressObject,
) -> Result<(Address, AccountId), StoreError> {
    let native_account_id: AccountId = AccountId::from_hex(&address_obj.id)?;
    let address = Address::read_from_bytes(&address_obj.address)?;
    Ok((address, native_account_id))
}

pub fn update_account(db_id: &str, new_account_state: &Account) {
    upsert_account_storage(db_id, new_account_state.storage());
    upsert_account_asset_vault(db_id, new_account_state.vault());
    upsert_account_record(db_id, new_account_state);
}
