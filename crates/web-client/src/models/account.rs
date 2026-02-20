use miden_client::Word as NativeWord;
use miden_client::account::{
    Account as NativeAccount,
    AccountInterfaceExt,
    AccountType as NativeAccountType,
};
use miden_client::transaction::AccountInterface;
use crate::prelude::*;

#[cfg(feature = "wasm")]
use crate::models::account_code::AccountCode;
use crate::models::account_id::AccountId;
#[cfg(feature = "wasm")]
use crate::models::account_storage::AccountStorage;
#[cfg(feature = "wasm")]
use crate::models::asset_vault::AssetVault;
use crate::models::felt::Felt;
use crate::models::word::Word;

/// An account which can store assets and define rules for manipulating them.
///
/// An account consists of the following components:
/// - Account ID, which uniquely identifies the account and also defines basic properties of the
///   account.
/// - Account vault, which stores assets owned by the account.
/// - Account storage, which is a key-value map (both keys and values are words) used to store
///   arbitrary user-defined data.
/// - Account code, which is a set of Miden VM programs defining the public interface of the
///   account.
/// - Account nonce, a value which is incremented whenever account state is updated.
///
/// Out of the above components account ID is always immutable (once defined it can never be
/// changed). Other components may be mutated throughout the lifetime of the account. However,
/// account state can be changed only by invoking one of account interface methods.
///
/// The recommended way to build an account is through an `AccountBuilder`, which can be
/// instantiated directly from a 32-byte seed.
#[derive(Clone)]
#[bindings]
pub struct Account(pub(crate) NativeAccount);

// Shared methods with identical signatures
#[bindings]
impl Account {
    /// Returns the account identifier.
    pub fn id(&self) -> AccountId {
        self.0.id().into()
    }

    /// Returns the commitment to the account header, storage, and code.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns the account nonce, which is incremented on every state update.
    pub fn nonce(&self) -> Felt {
        self.0.nonce().into()
    }

    /// Returns true if the account is a faucet.
    pub fn is_faucet(&self) -> bool {
        self.0.is_faucet()
    }

    /// Returns true if the account is a regular account (immutable or updatable code).
    pub fn is_regular_account(&self) -> bool {
        self.0.is_regular_account()
    }

    /// Returns true if the account can update its code.
    pub fn is_updatable(&self) -> bool {
        matches!(self.0.account_type(), NativeAccountType::RegularAccountUpdatableCode)
    }

    /// Returns true if the account exposes public storage.
    pub fn is_public(&self) -> bool {
        self.0.is_public()
    }

    /// Returns true if the account storage is private.
    pub fn is_private(&self) -> bool {
        self.0.is_private()
    }

    /// Returns true if this is a network-owned account.
    pub fn is_network(&self) -> bool {
        self.0.is_network()
    }

    /// Returns true if the account has not yet been committed to the chain.
    pub fn is_new(&self) -> bool {
        self.0.is_new()
    }

    /// Returns the public key commitments derived from the account's authentication scheme.
    pub fn get_public_key_commitments(&self) -> Vec<Word> {
        let inner_account = &self.0;
        let mut pks = vec![];
        let interface: AccountInterface = AccountInterface::from_account(inner_account);

        for auth in interface.auth() {
            pks.extend(auth.get_public_key_commitments());
        }

        pks.into_iter().map(NativeWord::from).map(Into::into).collect()
    }

    /// Serializes the account into bytes.
    pub fn serialize(&self) -> JsBytes {
        platform::serialize_to_bytes(&self.0)
    }
}

// wasm-only: vault, storage, code, deserialize
#[cfg(feature = "wasm")]
impl Account {
    /// Returns the vault commitment for this account.
    pub fn vault(&self) -> AssetVault {
        self.0.vault().into()
    }

    /// Returns the account storage commitment.
    pub fn storage(&self) -> AccountStorage {
        self.0.storage().into()
    }

    /// Returns the code commitment for this account.
    pub fn code(&self) -> AccountCode {
        self.0.code().into()
    }

    /// Restores an account from its serialized bytes.
    pub fn deserialize(bytes: &JsBytes) -> JsResult<Account> {
        platform::deserialize_from_bytes::<NativeAccount>(bytes).map(Account)
    }
}

// napi-only: deserialize
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl Account {
    /// Restores an account from its serialized bytes.
    #[napi(factory)]
    pub fn deserialize(bytes: JsBytes) -> JsResult<Account> {
        platform::deserialize_from_bytes::<NativeAccount>(&bytes).map(Account)
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
