use miden_client::account::AccountHeader as NativeAccountHeader;

use super::account_id::AccountId;
use super::felt::Felt;
use super::word::Word;
use super::{napi_delegate, napi_wrap};

napi_wrap!(clone AccountHeader wraps NativeAccountHeader, one_way);

napi_delegate!(impl AccountHeader {
    /// Returns the full account commitment.
    delegate commitment -> Word;
    /// Returns the account ID.
    delegate id -> AccountId;
    /// Returns the current nonce.
    delegate nonce -> Felt;
    /// Returns the storage commitment.
    delegate storage_commitment -> Word;
    /// Returns the code commitment.
    delegate code_commitment -> Word;
});

#[napi]
impl AccountHeader {
    /// Returns the vault commitment.
    #[napi]
    pub fn vault_commitment(&self) -> Word {
        self.0.vault_root().into()
    }
}
