use alloc::sync::Arc;
use alloc::vec::Vec;

use idxdb_store::WebStore;
use miden_client::account::{
    AccountId as NativeAccountId,
    AccountReader as NativeAccountReader,
    StorageSlotName,
};
use wasm_bindgen::prelude::*;

use super::account_header::AccountHeader;
use super::account_id::AccountId;
use super::address::Address;
use super::felt::Felt;
use super::word::Word;
use crate::js_error_with_context;

/// Provides lazy access to account data.
///
/// `AccountReader` executes queries lazily - each method call fetches fresh data
/// from storage, ensuring you always see the current state.
#[wasm_bindgen]
pub struct AccountReader(NativeAccountReader);

#[wasm_bindgen]
impl AccountReader {
    /// Creates a new `AccountReader` for the given account.
    pub(crate) fn new(store: Arc<WebStore>, account_id: NativeAccountId) -> Self {
        let inner = NativeAccountReader::new(store, account_id);
        Self(inner)
    }

    /// Returns the account ID.
    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.0.account_id().into()
    }

    // HEADER ACCESS
    // --------------------------------------------------------------------------------------------

    /// Retrieves the current account nonce.
    pub async fn nonce(&self) -> Result<Felt, JsValue> {
        self.0
            .nonce()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get account nonce"))
    }

    /// Retrieves the account commitment (hash of the full state).
    pub async fn commitment(&self) -> Result<Word, JsValue> {
        self.0
            .commitment()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get account commitment"))
    }

    /// Retrieves the storage commitment (root of the storage tree).
    #[wasm_bindgen(js_name = "storageCommitment")]
    pub async fn storage_commitment(&self) -> Result<Word, JsValue> {
        self.0
            .storage_commitment()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get storage commitment"))
    }

    /// Retrieves the vault root (root of the asset vault tree).
    #[wasm_bindgen(js_name = "vaultRoot")]
    pub async fn vault_root(&self) -> Result<Word, JsValue> {
        self.0
            .vault_root()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get vault root"))
    }

    /// Retrieves the code commitment (hash of the account code).
    #[wasm_bindgen(js_name = "codeCommitment")]
    pub async fn code_commitment(&self) -> Result<Word, JsValue> {
        self.0
            .code_commitment()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get code commitment"))
    }

    /// Returns whether the account is locked.
    #[wasm_bindgen(js_name = "isLocked")]
    pub async fn is_locked(&self) -> Result<bool, JsValue> {
        self.0
            .is_locked()
            .await
            .map_err(|err| js_error_with_context(err, "failed to check if account is locked"))
    }

    /// Returns whether the account is new.
    #[wasm_bindgen(js_name = "isNew")]
    pub async fn is_new(&self) -> Result<bool, JsValue> {
        self.0
            .is_new()
            .await
            .map_err(|err| js_error_with_context(err, "failed to check if account is new"))
    }

    /// Retrieves the account seed (if available for new/locked accounts).
    pub async fn seed(&self) -> Result<Option<Word>, JsValue> {
        self.0
            .seed()
            .await
            .map(|opt| opt.map(Into::into))
            .map_err(|err| js_error_with_context(err, "failed to get account seed"))
    }

    /// Retrieves the account header.
    pub async fn header(&self) -> Result<AccountHeader, JsValue> {
        let (header, _) = self
            .0
            .header()
            .await
            .map_err(|err| js_error_with_context(err, "failed to get account header"))?;
        Ok(header.into())
    }

    // ACCOUNT DATA ACCESS
    // --------------------------------------------------------------------------------------------

    /// Retrieves the addresses associated with this account.
    pub async fn addresses(&self) -> Result<Vec<Address>, JsValue> {
        self.0
            .addresses()
            .await
            .map(|addrs| addrs.into_iter().map(Into::into).collect())
            .map_err(|err| js_error_with_context(err, "failed to get account addresses"))
    }

    // VAULT ACCESS
    // --------------------------------------------------------------------------------------------

    /// Retrieves the balance of a fungible asset in the account's vault.
    ///
    /// Returns 0 if the asset is not present in the vault.
    #[wasm_bindgen(js_name = "getBalance")]
    pub async fn get_balance(&self, faucet_id: &AccountId) -> Result<u64, JsValue> {
        self.0
            .get_balance(faucet_id.into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get balance"))
    }

    // STORAGE ACCESS
    // --------------------------------------------------------------------------------------------

    /// Retrieves a storage slot value by name.
    ///
    /// For `Value` slots, returns the stored word.
    /// For `Map` slots, returns the map root.
    #[wasm_bindgen(js_name = "getStorageItem")]
    pub async fn get_storage_item(&self, slot_name: &str) -> Result<Word, JsValue> {
        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| js_error_with_context(err, "invalid slot name"))?;

        self.0
            .get_storage_item(slot_name)
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get storage item"))
    }

    /// Retrieves a value from a storage map slot by name and key.
    #[wasm_bindgen(js_name = "getStorageMapItem")]
    pub async fn get_storage_map_item(&self, slot_name: &str, key: &Word) -> Result<Word, JsValue> {
        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| js_error_with_context(err, "invalid slot name"))?;

        self.0
            .get_storage_map_item(slot_name, *key.as_native())
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get storage map item"))
    }
}
