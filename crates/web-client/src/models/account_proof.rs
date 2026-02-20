use miden_client::account::StorageSlotName;
use miden_client::block::BlockNumber;
use miden_client::rpc::domain::account::AccountProof as NativeAccountProof;
use miden_protocol::account::AccountStorageHeader;
use wasm_bindgen::prelude::*;

use super::account_code::AccountCode;
use super::account_header::AccountHeader;
use super::account_id::AccountId;
use super::word::Word;
use crate::js_error_with_context;

/// Proof of existence of an account's state at a specific block number, as returned by the node.
///
/// For public accounts, this includes the account header, storage slot values and account code.
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
}

// CONVERSIONS
// ================================================================================================

impl AccountProof {
    pub fn new(inner: NativeAccountProof, block_num: BlockNumber) -> Self {
        Self { inner, block_num }
    }
}
