use alloc::string::String;
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::account::{StorageMap, StorageSlot};
use miden_client::asset::Asset;
use miden_client::utils::Serializable;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;

// INDEXED DB BINDINGS
// ================================================================================================

// Account IndexedDB Operations
#[wasm_bindgen(module = "/src/js/accounts.js")]
extern "C" {
    // GETS
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = getAccountIds)]
    pub fn idxdb_get_account_ids(db_id: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAllAccountHeaders)]
    pub fn idxdb_get_account_headers(db_id: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountHeader)]
    pub fn idxdb_get_account_header(db_id: &str, account_id: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountHeaderByCommitment)]
    pub fn idxdb_get_account_header_by_commitment(
        db_id: &str,
        account_commitment: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountCode)]
    pub fn idxdb_get_account_code(db_id: &str, code_root: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountStorage)]
    pub fn idxdb_get_account_storage(db_id: &str, storage_root: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountStorageMaps)]
    pub fn idxdb_get_account_storage_maps(db_id: &str, roots: Vec<String>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountVaultAssets)]
    pub fn idxdb_get_account_vault_assets(db_id: &str, vault_root: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountAuthByPubKeyCommitment)]
    pub fn idxdb_get_account_auth_by_pub_key_commitment(
        db_id: &str,
        pub_key_commitment_hex: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountAddresses)]
    pub fn idxdb_get_account_addresses(db_id: &str, account_id: String) -> js_sys::Promise;

    // INSERTS
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = upsertAccountCode)]
    pub fn idxdb_upsert_account_code(
        db_id: &str,
        code_root: String,
        code: Vec<u8>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertAccountStorage)]
    pub fn idxdb_upsert_account_storage(
        db_id: &str,
        storage_slots: Vec<JsStorageSlot>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertStorageMapEntries)]
    pub fn idxdb_upsert_storage_map_entries(
        db_id: &str,
        entries: Vec<JsStorageMapEntry>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertVaultAssets)]
    pub fn idxdb_upsert_vault_assets(db_id: &str, assets: Vec<JsVaultAsset>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertAccountRecord)]
    pub fn idxdb_upsert_account_record(
        db_id: &str,
        id: String,
        code_root: String,
        storage_root: String,
        vault_root: String,
        nonce: String,
        committed: bool,
        commitment: String,
        account_seed: Option<Vec<u8>>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = insertAccountAddress)]
    pub fn idxdb_insert_account_address(
        db_id: &str,
        account_id: String,
        address: Vec<u8>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = removeAccountAddress)]
    pub fn idxdb_remove_account_address(db_id: &str, address: Vec<u8>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertForeignAccountCode)]
    pub fn idxdb_upsert_foreign_account_code(
        db_id: &str,
        account_id: String,
        code: Vec<u8>,
        code_root: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getForeignAccountCode)]
    pub fn idxdb_get_foreign_account_code(db_id: &str, account_ids: Vec<String>)
    -> js_sys::Promise;

    // UPDATES
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = lockAccount)]
    pub fn idxdb_lock_account(db_id: &str, account_id: String) -> js_sys::Promise;

    // DELETES
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = undoAccountStates)]
    pub fn idxdb_undo_account_states(db_id: &str, account_hashes: Vec<String>) -> js_sys::Promise;

    // ACCOUNT PUBLIC KEYS
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = insertAccountPublicKey)]
    pub fn idxdb_insert_account_public_key(
        db_id: &str,
        pub_key_commitment_hex: String,
        account_id: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = insertAccountPublicKeys)]
    pub fn idxdb_insert_account_public_keys(
        db_id: &str,
        pub_key_commitment_hexes: Vec<String>,
        account_id: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountIdByPublicKey)]
    pub fn idxdb_get_account_id_by_public_key(
        db_id: &str,
        pub_key_commitment_hex: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getPublicKeysByAccountId)]
    pub fn idxdb_get_public_keys_by_account_id(db_id: &str, account_id: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = removeAccountPublicKey)]
    pub fn idxdb_remove_account_public_key(
        db_id: &str,
        pub_key_commitment_hex: String,
    ) -> js_sys::Promise;
}

// VAULT ASSET
// ================================================================================================

/// An object that contains a serialized vault asset
#[wasm_bindgen(getter_with_clone, inspectable)]
#[derive(Clone)]
pub struct JsVaultAsset {
    /// The merkle root of the vault's assets.
    #[wasm_bindgen(js_name = "root")]
    pub root: String,
    /// The vault key associated with the asset.
    #[wasm_bindgen(js_name = "vaultKey")]
    pub vault_key: String,
    /// Asset's faucet ID prefix.
    #[wasm_bindgen(js_name = "faucetIdPrefix")]
    pub faucet_id_prefix: String,
    /// Word representing the asset.
    #[wasm_bindgen(js_name = "asset")]
    pub asset: String,
}

impl JsVaultAsset {
    pub fn from_asset(asset: &Asset, vault_root: Word) -> Self {
        Self {
            root: vault_root.to_hex(),
            vault_key: Word::from(asset.vault_key()).to_hex(),
            faucet_id_prefix: asset.faucet_id_prefix().to_hex(),
            asset: Word::from(asset).to_hex(),
        }
    }
}

// STORAGE SLOT
// ================================================================================================

/// A JavaScript representation of a storage slot in an account.
#[wasm_bindgen(getter_with_clone, inspectable)]
#[derive(Clone)]
pub struct JsStorageSlot {
    /// Commitment of the whole account storage
    #[wasm_bindgen(js_name = "commitment")]
    pub commitment: String,
    /// The name of the storage slot.
    #[wasm_bindgen(js_name = "slotName")]
    pub slot_name: String,
    /// The value stored in the storage slot.
    #[wasm_bindgen(js_name = "slotValue")]
    pub slot_value: String,
    /// The type of the storage slot.
    #[wasm_bindgen(js_name = "slotType")]
    pub slot_type: u8,
}

impl JsStorageSlot {
    pub fn from_slot(slot: &StorageSlot, storage_commitment: Word) -> Self {
        Self {
            commitment: storage_commitment.to_hex(),
            slot_name: slot.name().to_string(),
            slot_value: slot.value().to_hex(),
            slot_type: slot.slot_type().to_bytes()[0],
        }
    }
}

// STORAGE MAP ENTRY
// ================================================================================================

/// A JavaScript representation of a storage map entry in an account.
#[wasm_bindgen(getter_with_clone, inspectable)]
#[derive(Clone)]
pub struct JsStorageMapEntry {
    /// The root of the storage map entry.
    #[wasm_bindgen(js_name = "root")]
    pub root: String,
    /// The key of the storage map entry.
    #[wasm_bindgen(js_name = "key")]
    pub key: String,
    /// The value of the storage map entry.
    #[wasm_bindgen(js_name = "value")]
    pub value: String,
}

impl JsStorageMapEntry {
    pub fn from_map(map: &StorageMap) -> Vec<Self> {
        map.entries()
            .map(|(key, value)| Self {
                root: map.root().to_hex(),
                key: key.to_hex(),
                value: value.to_hex(),
            })
            .collect()
    }
}
