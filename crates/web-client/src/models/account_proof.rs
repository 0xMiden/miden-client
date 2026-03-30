use alloc::string::String;
use alloc::vec::Vec;

use miden_client::Word as NativeWord;
use miden_client::account::StorageSlotName;
use miden_client::block::BlockNumber;
use miden_client::rpc::domain::account::{AccountProof as NativeAccountProof, StorageMapEntries};
use miden_protocol::account::AccountStorageHeader;
use wasm_bindgen::prelude::*;

use super::account_code::AccountCode;
use super::account_header::AccountHeader;
use super::account_id::AccountId;
use super::word::Word;
use crate::js_error_with_context;

/// Proof of existence of an account's state at a specific block number, as returned by the node.
///
/// For public accounts, this includes the account header, storage slot values, account code,
/// and optionally storage map entries for the requested storage maps.
/// For private accounts, only the account commitment and merkle proof are available.
#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountProof {
    inner: NativeAccountProof,
    block_num: BlockNumber,
}

#[wasm_bindgen]
impl AccountProof {
    /// Returns the account ID.
    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.inner.account_id().into()
    }

    /// Returns the block number at which this proof was retrieved.
    #[wasm_bindgen(js_name = "blockNum")]
    pub fn block_num(&self) -> u32 {
        self.block_num.as_u32()
    }

    /// Returns the account commitment (hash of the full state).
    #[wasm_bindgen(js_name = "accountCommitment")]
    pub fn account_commitment(&self) -> Word {
        self.inner.account_commitment().into()
    }

    /// Returns the account header, if available (public accounts only).
    #[wasm_bindgen(js_name = "accountHeader")]
    pub fn account_header(&self) -> Option<AccountHeader> {
        self.inner.account_header().map(Into::into)
    }

    /// Returns the account code, if available (public accounts only).
    #[wasm_bindgen(js_name = "accountCode")]
    pub fn account_code(&self) -> Option<AccountCode> {
        self.inner.account_code().map(Into::into)
    }

    /// Returns the value of a storage slot by name, if available.
    ///
    /// For `Value` slots, this returns the stored word.
    /// For `Map` slots, this returns the map root commitment.
    ///
    /// Returns `undefined` if the account is private or the slot name is not found.
    #[wasm_bindgen(js_name = "getStorageSlotValue")]
    pub fn get_storage_slot_value(&self, slot_name: &str) -> Result<Option<Word>, JsValue> {
        let Some(storage_header) = self.inner.storage_header() else {
            return Ok(None);
        };

        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| js_error_with_context(err, "invalid slot name"))?;

        Ok(storage_header
            .find_slot_header_by_name(&slot_name)
            .map(|slot| slot.value().into()))
    }

    /// Returns the number of storage slots, if available (public accounts only).
    #[wasm_bindgen(js_name = "numStorageSlots")]
    pub fn num_storage_slots(&self) -> Option<u8> {
        self.inner.storage_header().map(AccountStorageHeader::num_slots)
    }

    /// Returns storage map entries for a given slot name, if available.
    ///
    /// Returns `undefined` if the account is private, the slot was not requested in the
    /// storage requirements, or the slot is not a map.
    ///
    /// Each entry contains a `key` and `value` as `Word` objects.
    #[wasm_bindgen(js_name = "getStorageMapEntries")]
    pub fn get_storage_map_entries(
        &self,
        slot_name: &str,
    ) -> Result<Option<Vec<StorageMapEntryJs>>, JsValue> {
        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| js_error_with_context(err, "invalid slot name"))?;

        let Some(map_details) = self.inner.find_map_details(&slot_name) else {
            return Ok(None);
        };

        let entries = match &map_details.entries {
            StorageMapEntries::AllEntries(entries) => entries
                .iter()
                .map(|e| StorageMapEntryJs {
                    key: Word::from(NativeWord::from(e.key)),
                    value: Word::from(e.value),
                })
                .collect(),
            // EntriesWithProofs contains raw SMT proofs without enumerable key-value
            // pairs. The proofs are used for verification only; entries cannot be
            // reconstructed from them.
            StorageMapEntries::EntriesWithProofs(_) => Vec::new(),
        };

        Ok(Some(entries))
    }

    /// Returns whether a storage map slot had too many entries to return inline.
    ///
    /// When this returns `true`, use `RpcClient.syncStorageMaps()` to fetch the full
    /// storage map data.
    ///
    /// Returns `undefined` if the slot was not found or the account is private.
    #[wasm_bindgen(js_name = "hasStorageMapTooManyEntries")]
    pub fn has_storage_map_too_many_entries(
        &self,
        slot_name: &str,
    ) -> Result<Option<bool>, JsValue> {
        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| js_error_with_context(err, "invalid slot name"))?;

        Ok(self.inner.find_map_details(&slot_name).map(|d| d.too_many_entries))
    }

    /// Returns the names of all storage slots that have map details available.
    ///
    /// This can be used to discover which storage maps were included in the proof response.
    /// Returns `undefined` if the account is private.
    #[wasm_bindgen(js_name = "getStorageMapSlotNames")]
    pub fn get_storage_map_slot_names(&self) -> Option<Vec<String>> {
        self.inner
            .storage_details()
            .map(|details| details.map_details.iter().map(|d| d.slot_name.to_string()).collect())
    }
}

// STORAGE MAP ENTRY
// ================================================================================================

/// A key-value entry from a storage map.
#[derive(Clone)]
#[wasm_bindgen(js_name = "StorageMapEntry")]
pub struct StorageMapEntryJs {
    key: Word,
    value: Word,
}

#[wasm_bindgen(js_class = "StorageMapEntry")]
impl StorageMapEntryJs {
    /// Returns the storage map key.
    pub fn key(&self) -> Word {
        self.key.clone()
    }

    /// Returns the storage map value.
    pub fn value(&self) -> Word {
        self.value.clone()
    }
}

// CONVERSIONS
// ================================================================================================

impl AccountProof {
    pub fn new(inner: NativeAccountProof, block_num: BlockNumber) -> Self {
        Self { inner, block_num }
    }
}
