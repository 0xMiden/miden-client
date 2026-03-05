use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_client::account::{
    AccountId as NativeAccountId,
    AccountReader as NativeAccountReader,
    StorageSlotName,
};
use miden_client::store::Store;
use js_export_macro::js_export;

use super::account_header::AccountHeader;
use super::account_id::AccountId;
use super::account_status::AccountStatus;
use super::address::Address;
use super::felt::Felt;
use super::word::Word;
use crate::js_error_with_context;
use crate::platform::JsErr;

/// Provides lazy access to account data.
///
/// `AccountReader` executes queries lazily - each method call fetches fresh data
/// from storage, ensuring you always see the current state.
#[js_export]
pub struct AccountReader(NativeAccountReader);

#[js_export]
impl AccountReader {
    /// Returns the account ID.
    #[js_export(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.0.account_id().into()
    }

    // HEADER ACCESS
    // --------------------------------------------------------------------------------------------

    /// Retrieves the current account nonce.
    pub async fn nonce(&self) -> Result<Felt, JsErr> {
        self.0
            .nonce()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get account nonce"))
    }

    /// Retrieves the account commitment (hash of the full state).
    pub async fn commitment(&self) -> Result<Word, JsErr> {
        self.0
            .commitment()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get account commitment"))
    }

    /// Retrieves the storage commitment (root of the storage tree).
    #[js_export(js_name = "storageCommitment")]
    pub async fn storage_commitment(&self) -> Result<Word, JsErr> {
        self.0
            .storage_commitment()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get storage commitment"))
    }

    /// Retrieves the vault root (root of the asset vault tree).
    #[js_export(js_name = "vaultRoot")]
    pub async fn vault_root(&self) -> Result<Word, JsErr> {
        self.0
            .vault_root()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get vault root"))
    }

    /// Retrieves the code commitment (hash of the account code).
    #[js_export(js_name = "codeCommitment")]
    pub async fn code_commitment(&self) -> Result<Word, JsErr> {
        self.0
            .code_commitment()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get code commitment"))
    }

    /// Retrieves the account header.
    pub async fn header(&self) -> Result<AccountHeader, JsErr> {
        let (header, _) = self
            .0
            .header()
            .await
            .map_err(|err| js_error_with_context(err, "failed to get account header"))?;
        Ok(header.into())
    }

    /// Retrieves the account status.
    pub async fn status(&self) -> Result<AccountStatus, JsErr> {
        self.0
            .status()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get account status"))
    }

    // ACCOUNT DATA ACCESS
    // --------------------------------------------------------------------------------------------

    /// Retrieves the addresses associated with this account.
    pub async fn addresses(&self) -> Result<Vec<Address>, JsErr> {
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
    #[js_export(js_name = "getBalance")]
    pub async fn get_balance(&self, faucet_id: &AccountId) -> Result<u64, JsErr> {
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
    #[js_export(js_name = "getStorageItem")]
    pub async fn get_storage_item(&self, slot_name: String) -> Result<Word, JsErr> {
        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| js_error_with_context(err, "invalid slot name"))?;

        self.0
            .get_storage_item(slot_name)
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get storage item"))
    }

    /// Retrieves a value from a storage map slot by name and key.
    #[js_export(js_name = "getStorageMapItem")]
    pub async fn get_storage_map_item(&self, slot_name: String, key: &Word) -> Result<Word, JsErr> {
        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| js_error_with_context(err, "invalid slot name"))?;

        self.0
            .get_storage_map_item(slot_name, *key.as_native())
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get storage map item"))
    }
}

impl AccountReader {
    /// Creates a new `AccountReader` for the given account.
    pub(crate) fn new(store: Arc<dyn Store>, account_id: NativeAccountId) -> Self {
        let inner = NativeAccountReader::new(store, account_id);
        Self(inner)
    }
}
