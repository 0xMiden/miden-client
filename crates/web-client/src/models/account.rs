use miden_client::account::{Account as NativeAccount, AccountType as NativeAccountType};
use miden_client::utils::get_public_keys_from_account;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::account_code::AccountCode;
use crate::models::account_id::AccountId;
use crate::models::account_storage::AccountStorage;
use crate::models::asset_vault::AssetVault;
use crate::models::felt::Felt;
use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// JavaScript wrapper around [`miden_client::account::Account`].
///
/// Exposes read-only accessors and serialization helpers for account data inside the web client.
#[derive(Clone)]
#[wasm_bindgen]
pub struct Account(NativeAccount);

#[wasm_bindgen]
impl Account {
    /// Returns the account identifier.
    pub fn id(&self) -> AccountId {
        self.0.id().into()
    }

    /// Returns the account commitment.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns the account nonce as a field element.
    pub fn nonce(&self) -> Felt {
        self.0.nonce().into()
    }

    /// Returns the asset vault associated with this account.
    pub fn vault(&self) -> AssetVault {
        self.0.vault().into()
    }

    /// Returns the storage associated with this account.
    pub fn storage(&self) -> AccountStorage {
        self.0.storage().into()
    }

    /// Returns the executable code stored in this account.
    pub fn code(&self) -> AccountCode {
        self.0.code().into()
    }

    #[wasm_bindgen(js_name = "isFaucet")]
    /// Returns `true` if the account is a faucet account.
    pub fn is_faucet(&self) -> bool {
        self.0.is_faucet()
    }

    #[wasm_bindgen(js_name = "isRegularAccount")]
    /// Returns `true` if the account is a regular account.
    pub fn is_regular_account(&self) -> bool {
        self.0.is_regular_account()
    }

    #[wasm_bindgen(js_name = "isUpdatable")]
    /// Returns `true` if the account supports updating its code.
    pub fn is_updatable(&self) -> bool {
        matches!(self.0.account_type(), NativeAccountType::RegularAccountUpdatableCode)
    }

    #[wasm_bindgen(js_name = "isPublic")]
    /// Returns `true` if the account is public.
    pub fn is_public(&self) -> bool {
        self.0.is_public()
    }

    #[wasm_bindgen(js_name = "isNew")]
    /// Returns `true` if the account has not been initialized yet.
    pub fn is_new(&self) -> bool {
        self.0.is_new()
    }

    /// Serializes this account into raw bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes an account from its byte representation.
    ///
    /// @param bytes - Serialized account bytes.
    /// @throws Throws if the bytes cannot be parsed into a valid account.
    pub fn deserialize(bytes: &Uint8Array) -> Result<Account, JsValue> {
        deserialize_from_uint8array::<NativeAccount>(bytes).map(Account)
    }

    #[wasm_bindgen(js_name = "getPublicKeys")]
    /// Returns the public keys associated with this account.
    pub fn get_public_keys(&self) -> Vec<Word> {
        let mut key_pairs = vec![];

        for pub_key in get_public_keys_from_account(&self.0) {
            key_pairs.push(pub_key.into());
        }

        key_pairs
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccount> for Account {
    fn from(native_account: NativeAccount) -> Self {
        Account(native_account)
    }
}

impl From<&NativeAccount> for Account {
    fn from(native_account: &NativeAccount) -> Self {
        Account(native_account.clone())
    }
}

impl From<Account> for NativeAccount {
    fn from(account: Account) -> Self {
        account.0
    }
}

impl From<&Account> for NativeAccount {
    fn from(account: &Account) -> Self {
        account.0.clone()
    }
}
