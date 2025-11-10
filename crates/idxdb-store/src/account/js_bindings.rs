use alloc::string::String;
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::account::{StorageMap, StorageSlot};
use miden_client::asset::Asset;
use miden_client::utils::Serializable;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, wasm_bindgen};

// INDEXED DB BINDINGS
// ================================================================================================

// Account IndexedDB Operations
include!(concat!(env!("OUT_DIR"), "/generated_js_bindings/account_js_bindings.rs"));

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
    /// The index of the storage slot.
    #[wasm_bindgen(js_name = "slotIndex")]
    pub slot_index: u8,
    /// The value stored in the storage slot.
    #[wasm_bindgen(js_name = "slotValue")]
    pub slot_value: String,
    /// The type of the storage slot.
    #[wasm_bindgen(js_name = "slotType")]
    pub slot_type: u8,
}

impl JsStorageSlot {
    pub fn from_slot(slot: &StorageSlot, index: u8, storage_commitment: Word) -> Self {
        Self {
            commitment: storage_commitment.to_hex(),
            slot_index: index,
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
