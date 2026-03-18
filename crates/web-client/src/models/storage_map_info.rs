use alloc::string::String;
use alloc::vec::Vec;

use miden_client::rpc::domain::storage_map::StorageMapInfo as NativeStorageMapInfo;
use wasm_bindgen::prelude::*;

use super::word::Word;

/// Information about storage map updates for an account, as returned by the
/// `syncStorageMaps` RPC endpoint.
///
/// Contains the list of storage map updates within the requested block range,
/// along with the chain tip and last processed block number.
#[wasm_bindgen(js_name = "StorageMapInfo")]
pub struct StorageMapInfo {
    chain_tip: u32,
    block_number: u32,
    updates: Vec<StorageMapUpdate>,
}

#[wasm_bindgen(js_class = "StorageMapInfo")]
impl StorageMapInfo {
    /// Returns the current chain tip block number.
    #[wasm_bindgen(js_name = "chainTip")]
    pub fn chain_tip(&self) -> u32 {
        self.chain_tip
    }

    /// Returns the block number of the last check included in this response.
    #[wasm_bindgen(js_name = "blockNumber")]
    pub fn block_number(&self) -> u32 {
        self.block_number
    }

    /// Returns the list of storage map updates.
    pub fn updates(&self) -> Vec<StorageMapUpdate> {
        self.updates.clone()
    }
}

// STORAGE MAP UPDATE
// ================================================================================================

/// A single storage map update entry, containing the block number, slot name,
/// key, and new value.
#[derive(Clone)]
#[wasm_bindgen(js_name = "StorageMapUpdate")]
pub struct StorageMapUpdate {
    block_num: u32,
    slot_name: String,
    key: Word,
    value: Word,
}

#[wasm_bindgen(js_class = "StorageMapUpdate")]
impl StorageMapUpdate {
    /// Returns the block number in which this update occurred.
    #[wasm_bindgen(js_name = "blockNum")]
    pub fn block_num(&self) -> u32 {
        self.block_num
    }

    /// Returns the name of the storage slot that was updated.
    #[wasm_bindgen(js_name = "slotName")]
    pub fn slot_name(&self) -> String {
        self.slot_name.clone()
    }

    /// Returns the storage map key that was updated.
    pub fn key(&self) -> Word {
        self.key.clone()
    }

    /// Returns the new value for this storage map key.
    pub fn value(&self) -> Word {
        self.value.clone()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeStorageMapInfo> for StorageMapInfo {
    fn from(native: NativeStorageMapInfo) -> Self {
        let updates = native
            .updates
            .iter()
            .map(|u| StorageMapUpdate {
                block_num: u.block_num.as_u32(),
                slot_name: u.slot_name.to_string(),
                key: miden_protocol::Word::from(u.key).into(),
                value: u.value.into(),
            })
            .collect();

        Self {
            chain_tip: native.chain_tip.as_u32(),
            block_number: native.block_number.as_u32(),
            updates,
        }
    }
}
