use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::account::{
    Account,
    AccountCode,
    AccountDelta,
    AccountHeader,
    AccountId,
    AccountStorage,
    Address,
    StorageSlotContent,
    StorageSlotType,
};
use miden_client::asset::{Asset, AssetVault, FungibleAsset};
use miden_client::store::{AccountStatus, StoreError};
use miden_client::utils::{Deserializable, Serializable};
use miden_client::{EMPTY_WORD, Felt, Word};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use super::js_bindings::{
    JsStorageMapEntry,
    JsStorageSlot,
    JsVaultAsset,
    idxdb_apply_account_delta,
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
        slots.push(JsStorageSlot::from_slot(slot));
        if let StorageSlotContent::Map(map) = slot.content() {
            maps.extend(JsStorageMapEntry::from_map(map, slot.name().as_str()));
        }
    }

    let account_id_str = account_id.to_string();
    let nonce_str = nonce.to_string();
    JsFuture::from(idxdb_upsert_account_storage(
        db_id,
        account_id_str.clone(),
        nonce_str.clone(),
        slots,
    ))
    .await?;
    JsFuture::from(idxdb_upsert_storage_map_entries(db_id, account_id_str, nonce_str, maps))
        .await?;

    Ok(())
}

pub async fn upsert_account_asset_vault(
    db_id: &str,
    account_id: &AccountId,
    nonce: u64,
    asset_vault: &AssetVault,
) -> Result<(), JsValue> {
    let js_assets: Vec<JsVaultAsset> =
        asset_vault.assets().map(|asset| JsVaultAsset::from_asset(&asset)).collect();

    let promise =
        idxdb_upsert_vault_assets(db_id, account_id.to_string(), nonce.to_string(), js_assets);
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

/// Applies the full account delta (storage + vault + header) in a single atomic `IndexedDB`
/// transaction. Value slots and map entries use empty-string sentinels for removals on the JS
/// side.
pub async fn apply_account_delta(
    db_id: &str,
    account: &Account,
    delta: &AccountDelta,
) -> Result<(), JsValue> {
    let account_id_str = account.id().to_string();
    let nonce_str = account.nonce().to_string();

    // --- Storage delta ---

    let mut updated_slots = Vec::new();
    let mut changed_map_entries = Vec::new();

    // Value slots: delta gives (StorageSlotName, Word)
    for (slot_name, value) in delta.storage().values() {
        updated_slots.push(JsStorageSlot {
            slot_name: slot_name.to_string(),
            slot_value: value.to_hex(),
            slot_type: StorageSlotType::Value as u8,
        });
    }

    // Map slots: delta gives (StorageSlotName, StorageMapDelta)
    for (slot_name, map_delta) in delta.storage().maps() {
        // Get the new root value from the final account state for the slot itself
        if let Some(slot) = account.storage().get(slot_name) {
            updated_slots.push(JsStorageSlot::from_slot(slot));
        }

        // Extract changed map entries from the delta
        for (key, value) in map_delta.entries() {
            let value_str = if *value == EMPTY_WORD {
                // Removal sentinel — the JS side interprets "" as "remove from latest,
                // write null tombstone to historical"
                String::new()
            } else {
                value.to_hex()
            };

            changed_map_entries.push(JsStorageMapEntry {
                slot_name: slot_name.to_string(),
                key: key.inner().to_hex(),
                value: value_str,
            });
        }
    }

    // --- Vault delta ---

    let mut changed_assets = Vec::new();

    // Process fungible asset changes
    for (faucet_id, _amount_delta) in delta.vault().fungible().iter() {
        let balance = account
            .vault()
            .get_balance(*faucet_id)
            .expect("faucet_id from delta should be valid");

        if balance > 0 {
            // Asset still exists in the final state — write its current value
            let asset = FungibleAsset::new(*faucet_id, balance)
                .expect("balance from vault should be valid");
            changed_assets.push(JsVaultAsset::from_asset(&Asset::Fungible(asset)));
        } else {
            // Asset was removed — construct a removal sentinel
            // We need vault_key and faucet_id_prefix for the tombstone; create a dummy asset
            // with amount 1 just to derive these identifiers.
            let dummy =
                FungibleAsset::new(*faucet_id, 1).expect("faucet_id from delta should be valid");
            changed_assets.push(JsVaultAsset {
                vault_key: Word::from(dummy.vault_key()).to_hex(),
                faucet_id_prefix: dummy.faucet_id_prefix().to_hex(),
                asset: String::new(),
            });
        }
    }

    // Process non-fungible asset changes
    for (nft, action) in delta.vault().non_fungible().iter() {
        use miden_client::asset::NonFungibleDeltaAction;
        match action {
            NonFungibleDeltaAction::Add => {
                changed_assets.push(JsVaultAsset::from_asset(&Asset::NonFungible(*nft)));
            },
            NonFungibleDeltaAction::Remove => {
                changed_assets.push(JsVaultAsset {
                    vault_key: Word::from(nft.vault_key()).to_hex(),
                    faucet_id_prefix: nft.faucet_id_prefix().to_hex(),
                    asset: String::new(),
                });
            },
        }
    }

    // --- Account header ---

    let code_root = account.code().commitment().to_string();
    let storage_root = account.storage().to_commitment().to_string();
    let vault_root = account.vault().root().to_string();
    let committed = account.is_public();
    let commitment = account.commitment().to_string();
    let account_seed = account.seed().map(|seed| seed.to_bytes());

    JsFuture::from(idxdb_apply_account_delta(
        db_id,
        account_id_str,
        nonce_str,
        updated_slots,
        changed_map_entries,
        changed_assets,
        code_root,
        storage_root,
        vault_root,
        committed,
        commitment,
        account_seed,
    ))
    .await?;

    Ok(())
}

pub async fn update_account(db_id: &str, new_account_state: &Account) -> Result<(), JsValue> {
    let account_id = &new_account_state.id();
    let nonce = new_account_state.nonce().as_int();
    upsert_account_storage(db_id, account_id, nonce, new_account_state.storage()).await?;
    upsert_account_asset_vault(db_id, account_id, nonce, new_account_state.vault()).await?;
    upsert_account_record(db_id, new_account_state).await
}
