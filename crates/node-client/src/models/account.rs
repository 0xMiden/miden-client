use miden_client::Word as NativeWord;
use miden_client::account::{
    Account as NativeAccount,
    AccountInterfaceExt,
    AccountType as NativeAccountType,
};
use miden_client::transaction::AccountInterface;
use miden_client::utils::{Deserializable, Serializable};
use napi::bindgen_prelude::*;

use super::account_id::AccountId;
use super::felt::Felt;
use super::word::Word;

#[napi]
#[derive(Clone)]
pub struct Account(pub(crate) NativeAccount);

#[napi]
impl Account {
    /// Returns the account identifier.
    #[napi]
    pub fn id(&self) -> AccountId {
        self.0.id().into()
    }

    /// Returns the commitment to the account header, storage, and code.
    #[napi]
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns the account nonce.
    #[napi]
    pub fn nonce(&self) -> Felt {
        self.0.nonce().into()
    }

    /// Returns true if the account is a faucet.
    #[napi(js_name = "isFaucet")]
    pub fn is_faucet(&self) -> bool {
        self.0.is_faucet()
    }

    /// Returns true if the account is a regular account.
    #[napi(js_name = "isRegularAccount")]
    pub fn is_regular_account(&self) -> bool {
        self.0.is_regular_account()
    }

    /// Returns true if the account can update its code.
    #[napi(js_name = "isUpdatable")]
    pub fn is_updatable(&self) -> bool {
        matches!(self.0.account_type(), NativeAccountType::RegularAccountUpdatableCode)
    }

    /// Returns true if the account uses public storage.
    #[napi(js_name = "isPublic")]
    pub fn is_public(&self) -> bool {
        self.0.is_public()
    }

    /// Returns true if the account storage is private.
    #[napi(js_name = "isPrivate")]
    pub fn is_private(&self) -> bool {
        self.0.is_private()
    }

    /// Returns true if this is a network-owned account.
    #[napi(js_name = "isNetwork")]
    pub fn is_network(&self) -> bool {
        self.0.is_network()
    }

    /// Returns true if the account has not yet been committed to the chain.
    #[napi(js_name = "isNew")]
    pub fn is_new(&self) -> bool {
        self.0.is_new()
    }

    /// Serializes the account into bytes.
    #[napi]
    pub fn serialize(&self) -> Buffer {
        self.0.to_bytes().into()
    }

    /// Restores an account from its serialized bytes.
    #[napi]
    pub fn deserialize(bytes: Buffer) -> Result<Account> {
        let native = NativeAccount::read_from_bytes(&bytes).map_err(|err| {
            napi::Error::from_reason(format!("Failed to deserialize Account: {err}"))
        })?;
        Ok(Account(native))
    }

    /// Returns the public key commitments derived from the account's auth scheme.
    #[napi(js_name = "getPublicKeyCommitments")]
    pub fn get_public_key_commitments(&self) -> Vec<Word> {
        let interface: AccountInterface = AccountInterface::from_account(&self.0);
        let mut pks = vec![];
        for auth in interface.auth() {
            pks.extend(auth.get_public_key_commitments());
        }
        pks.into_iter().map(NativeWord::from).map(Into::into).collect()
    }
}

impl From<NativeAccount> for Account {
    fn from(native: NativeAccount) -> Self {
        Account(native)
    }
}

impl From<&Account> for NativeAccount {
    fn from(account: &Account) -> Self {
        account.0.clone()
    }
}

impl From<Account> for NativeAccount {
    fn from(account: Account) -> Self {
        account.0
    }
}
