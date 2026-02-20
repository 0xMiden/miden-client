use crate::prelude::*;

#[cfg(feature = "wasm")]
use idxdb_store::WebStore;
use miden_client::account::{
    AccountId as NativeAccountId,
    AccountReader as NativeAccountReader,
    StorageSlotName,
};
#[cfg(feature = "napi")]
use miden_client::store::Store;

use super::account_header::AccountHeader;
use super::account_id::AccountId;
use super::account_status::AccountStatus;
use super::address::Address;
use super::felt::Felt;
use super::word::Word;

/// Provides lazy access to account data.
///
/// `AccountReader` executes queries lazily - each method call fetches fresh data
/// from storage, ensuring you always see the current state.
#[bindings]
pub struct AccountReader(NativeAccountReader);

// wasm: new takes Arc<WebStore>
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AccountReader {
    /// Creates a new `AccountReader` for the given account.
    pub(crate) fn new(store: Arc<WebStore>, account_id: NativeAccountId) -> Self {
        let inner = NativeAccountReader::new(store, account_id);
        Self(inner)
    }
}

// napi: new takes Arc<dyn Store>
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl AccountReader {
    /// Creates a new `AccountReader` for the given account.
    pub(crate) fn new(store: Arc<dyn Store>, account_id: NativeAccountId) -> Self {
        let inner = NativeAccountReader::new(store, account_id);
        Self(inner)
    }
}

// Shared methods with identical implementations across both platforms
#[bindings]
impl AccountReader {
    /// Returns the account ID.
    pub fn account_id(&self) -> AccountId {
        self.0.account_id().into()
    }

    /// Retrieves the current account nonce.
    pub async fn nonce(&self) -> JsResult<Felt> {
        self.0
            .nonce()
            .await
            .map(Into::into)
            .map_err(|err| platform::error_with_context(err, "failed to get account nonce"))
    }

    /// Retrieves the account commitment (hash of the full state).
    pub async fn commitment(&self) -> JsResult<Word> {
        self.0
            .commitment()
            .await
            .map(Into::into)
            .map_err(|err| platform::error_with_context(err, "failed to get account commitment"))
    }

    /// Retrieves the storage commitment (root of the storage tree).
    pub async fn storage_commitment(&self) -> JsResult<Word> {
        self.0
            .storage_commitment()
            .await
            .map(Into::into)
            .map_err(|err| platform::error_with_context(err, "failed to get storage commitment"))
    }

    /// Retrieves the vault root (root of the asset vault tree).
    pub async fn vault_root(&self) -> JsResult<Word> {
        self.0
            .vault_root()
            .await
            .map(Into::into)
            .map_err(|err| platform::error_with_context(err, "failed to get vault root"))
    }

    /// Retrieves the code commitment (hash of the account code).
    pub async fn code_commitment(&self) -> JsResult<Word> {
        self.0
            .code_commitment()
            .await
            .map(Into::into)
            .map_err(|err| platform::error_with_context(err, "failed to get code commitment"))
    }

    /// Retrieves the account header.
    pub async fn header(&self) -> JsResult<AccountHeader> {
        let (header, _) = self
            .0
            .header()
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get account header"))?;
        Ok(header.into())
    }

    /// Retrieves the account status.
    pub async fn status(&self) -> JsResult<AccountStatus> {
        self.0
            .status()
            .await
            .map(Into::into)
            .map_err(|err| platform::error_with_context(err, "failed to get account status"))
    }

    /// Retrieves the addresses associated with this account.
    pub async fn addresses(&self) -> JsResult<Vec<Address>> {
        self.0
            .addresses()
            .await
            .map(|addrs| addrs.into_iter().map(Into::into).collect())
            .map_err(|err| platform::error_with_context(err, "failed to get account addresses"))
    }

    /// Retrieves the balance of a fungible asset in the account's vault.
    ///
    /// Returns 0 if the asset is not present in the vault.
    pub async fn get_balance(&self, faucet_id: &AccountId) -> JsResult<i64> {
        self.0
            .get_balance(faucet_id.into())
            .await
            .map(|v| v as i64)
            .map_err(|err| platform::error_with_context(err, "failed to get balance"))
    }

    /// Retrieves a storage slot value by name.
    ///
    /// For `Value` slots, returns the stored word.
    /// For `Map` slots, returns the map root.
    
    pub async fn get_storage_item(&self, slot_name: String) -> JsResult<Word> {
        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| platform::error_with_context(err, "invalid slot name"))?;

        self.0
            .get_storage_item(slot_name)
            .await
            .map(Into::into)
            .map_err(|err| platform::error_with_context(err, "failed to get storage item"))
    }

    /// Retrieves a value from a storage map slot by name and key.
    pub async fn get_storage_map_item(&self, slot_name: String, key: &Word) -> JsResult<Word> {
        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| platform::error_with_context(err, "invalid slot name"))?;

        self.0
            .get_storage_map_item(slot_name, *key.as_native())
            .await
            .map(Into::into)
            .map_err(|err| platform::error_with_context(err, "failed to get storage map item"))
    }
}
