use miden_client::account::AccountHeader as NativeAccountHeader;

use super::account_id::AccountId;
use super::felt::Felt;
use super::napi_wrap;
use super::word::Word;

napi_wrap!(clone AccountHeader wraps NativeAccountHeader, one_way);

#[napi]
impl AccountHeader {
    /// Returns the full account commitment.
    #[napi]
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns the account ID.
    #[napi]
    pub fn id(&self) -> AccountId {
        self.0.id().into()
    }

    /// Returns the current nonce.
    #[napi]
    pub fn nonce(&self) -> Felt {
        self.0.nonce().into()
    }

    /// Returns the vault commitment.
    #[napi(js_name = "vaultCommitment")]
    pub fn vault_commitment(&self) -> Word {
        self.0.vault_root().into()
    }

    /// Returns the storage commitment.
    #[napi(js_name = "storageCommitment")]
    pub fn storage_commitment(&self) -> Word {
        self.0.storage_commitment().into()
    }

    /// Returns the code commitment.
    #[napi(js_name = "codeCommitment")]
    pub fn code_commitment(&self) -> Word {
        self.0.code_commitment().into()
    }
}
