use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

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
    StorageSlotContent,
    StorageSlotName,
    StorageSlotType,
};
use miden_client::asset::{
    Asset, AssetVault, AssetVaultKey, FungibleAsset, NonFungibleDeltaAction,
};
use miden_client::store::{AccountSmtForest, AccountStatus, StoreError};
use miden_client::utils::{Deserializable, Serializable};
use miden_client::{EMPTY_WORD, Felt, Word};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use super::js_bindings::{
    JsStorageMapEntry,
    JsStorageSlot,
    JsVaultAsset,
    idxdb_apply_full_account_state,
    idxdb_apply_transaction_delta,
    idxdb_upsert_account_code,
    idxdb_upsert_account_record,
    idxdb_upsert_account_storage,
    idxdb_upsert_storage_map_entries,
    idxdb_upsert_vault_assets,
};
use crate::account::js_bindings::idxdb_insert_account_address;
use crate::account::models::{AccountRecordIdxdbObject, AddressIdxdbObject};
use crate::sync::JsAccountUpdate;

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

/// Computes new storage slot values from the delta, using the SMT forest for map root computation.
///
/// Value-type slots take their new value directly from the delta.
/// Map-type slots use the old root + delta entries to compute a new root via the SMT forest.
pub fn compute_storage_delta(
    smt_forest: &mut AccountSmtForest,
    old_map_roots: &BTreeMap<StorageSlotName, Word>,
    delta: &AccountDelta,
) -> Result<BTreeMap<StorageSlotName, (Word, StorageSlotType)>, StoreError> {
    let mut updated_slots: BTreeMap<StorageSlotName, (Word, StorageSlotType)> = delta
        .storage()
        .values()
        .map(|(slot_name, value)| (slot_name.clone(), (*value, StorageSlotType::Value)))
        .collect();

    let default_map_root = StorageMap::default().root();

    for (slot_name, map_delta) in delta.storage().maps() {
        let old_root = old_map_roots.get(slot_name).copied().unwrap_or(default_map_root);
        let entries: Vec<(Word, Word)> =
            map_delta.entries().iter().map(|(key, value)| ((*key).into(), *value)).collect();

        let new_root = smt_forest.update_storage_map_nodes(old_root, entries.into_iter())?;
        updated_slots.insert(slot_name.clone(), (new_root, StorageSlotType::Map));
    }

    Ok(updated_slots)
}

/// Computes the new vault state from old assets and the vault delta.
///
/// Returns (`updated_assets`, `removed_vault_keys`) where:
/// - `updated_assets` contains assets with their new values (for DB insertion and SMT update)
/// - `removed_vault_keys` contains vault keys for assets removed from the vault
pub fn compute_vault_delta(
    old_vault_assets: &[Asset],
    delta: &AccountDelta,
) -> Result<(Vec<Asset>, Vec<AssetVaultKey>), StoreError> {
    let mut updated_assets = Vec::new();
    let mut removed_vault_keys = Vec::new();

    // Build lookup map from faucet ID prefix to FungibleAsset
    let mut fungible_map: BTreeMap<AccountIdPrefix, FungibleAsset> = old_vault_assets
        .iter()
        .filter_map(|asset| match asset {
            Asset::Fungible(fa) => Some((fa.faucet_id_prefix(), *fa)),
            Asset::NonFungible(_) => None,
        })
        .collect();

    // Process fungible deltas
    for (faucet_id, delta_amount) in delta.vault().fungible().iter() {
        let delta_asset = FungibleAsset::new(*faucet_id, delta_amount.unsigned_abs())?;

        let asset = match fungible_map.remove(&faucet_id.prefix()) {
            Some(existing) => {
                if *delta_amount >= 0 {
                    existing.add(delta_asset)?
                } else {
                    existing.sub(delta_asset)?
                }
            },
            None => delta_asset,
        };

        if asset.amount() > 0 {
            updated_assets.push(Asset::Fungible(asset));
        } else {
            removed_vault_keys.push(asset.vault_key());
        }
    }

    // Process non-fungible deltas
    for (nft, action) in delta.vault().non_fungible().iter() {
        match action {
            NonFungibleDeltaAction::Add => {
                updated_assets.push(Asset::NonFungible(*nft));
            },
            NonFungibleDeltaAction::Remove => {
                removed_vault_keys.push(nft.vault_key());
            },
        }
    }

    Ok((updated_assets, removed_vault_keys))
}

/// Applies a transaction's account delta atomically in a single Dexie transaction.
///
/// Takes pre-computed values (storage roots from SMT forest, vault changes) instead of
/// the full Account object. This avoids loading account code and full storage map entries.
pub async fn apply_transaction_delta(
    db_id: &str,
    account_id: AccountId,
    final_header: &AccountHeader,
    account_seed: Option<Word>,
    updated_storage_slots: &BTreeMap<StorageSlotName, (Word, StorageSlotType)>,
    updated_assets: &[Asset],
    removed_vault_keys: &[AssetVaultKey],
    delta: &AccountDelta,
) -> Result<(), JsValue> {
    let account_id_str = account_id.to_string();
    let nonce_str = final_header.nonce().to_string();

    // Build updated slot JS objects from pre-computed storage roots
    let mut js_slots = Vec::new();
    for (slot_name, (value, slot_type)) in updated_storage_slots {
        js_slots.push(JsStorageSlot {
            slot_name: slot_name.to_string(),
            slot_value: value.to_hex(),
            slot_type: *slot_type as u8,
        });
    }

    // Build changed map entries from delta
    let mut changed_map_entries = Vec::new();
    for (slot_name, map_delta) in delta.storage().maps() {
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

    // Build changed assets: updated assets + removal markers
    let mut changed_assets: Vec<JsVaultAsset> =
        updated_assets.iter().map(JsVaultAsset::from_asset).collect();

    for vault_key in removed_vault_keys {
        changed_assets.push(JsVaultAsset {
            vault_key: Word::from(*vault_key).to_hex(),
            faucet_id_prefix: vault_key.faucet_id_prefix().to_hex(),
            asset: String::new(),
        });
    }

    // Account record fields from final header
    let code_root = final_header.code_commitment().to_string();
    let storage_root = final_header.storage_commitment().to_string();
    let vault_root = final_header.vault_root().to_string();
    let committed = account_id.is_public();
    let commitment = final_header.commitment().to_string();
    let account_seed_bytes = account_seed.map(|seed| seed.to_bytes());

    JsFuture::from(idxdb_apply_transaction_delta(
        db_id,
        account_id_str,
        nonce_str,
        js_slots,
        changed_map_entries,
        changed_assets,
        code_root,
        storage_root,
        vault_root,
        committed,
        commitment,
        account_seed_bytes,
    ))
    .await?;

    Ok(())
}

/// Writes the full account state atomically in a single Dexie transaction.
/// Combines storage upsert + map entries upsert + vault assets upsert + account record upsert.
pub async fn apply_full_account_state(db_id: &str, account: &Account) -> Result<(), JsValue> {
    let account_state = JsAccountUpdate::from_account(account, account.seed());

    JsFuture::from(idxdb_apply_full_account_state(db_id, account_state)).await?;

    Ok(())
}
