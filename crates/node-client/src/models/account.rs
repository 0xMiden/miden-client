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
use super::asset_vault::AssetVault;
use super::felt::Felt;
use super::word::Word;
use super::{napi_delegate, napi_wrap};

napi_wrap!(clone Account wraps NativeAccount);

napi_delegate!(impl Account {
    /// Returns the account identifier.
    delegate id -> AccountId;
    /// Returns the commitment to the account header, storage, and code.
    delegate commitment -> Word;
    /// Returns the account nonce.
    delegate nonce -> Felt;
    /// Returns the asset vault for this account.
    delegate vault -> AssetVault;
    /// Returns true if the account is a faucet.
    delegate is_faucet -> bool;
    /// Returns true if the account is a regular account.
    delegate is_regular_account -> bool;
    /// Returns true if the account uses public storage.
    delegate is_public -> bool;
    /// Returns true if the account storage is private.
    delegate is_private -> bool;
    /// Returns true if this is a network-owned account.
    delegate is_network -> bool;
    /// Returns true if the account has not yet been committed to the chain.
    delegate is_new -> bool;
});

#[napi]
impl Account {
    /// Returns true if the account can update its code.
    #[napi]
    pub fn is_updatable(&self) -> bool {
        matches!(self.0.account_type(), NativeAccountType::RegularAccountUpdatableCode)
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
    #[napi]
    pub fn get_public_key_commitments(&self) -> Vec<Word> {
        let interface: AccountInterface = AccountInterface::from_account(&self.0);
        let mut pks = vec![];
        for auth in interface.auth() {
            pks.extend(auth.get_public_key_commitments());
        }
        pks.into_iter().map(NativeWord::from).map(Into::into).collect()
    }
}
