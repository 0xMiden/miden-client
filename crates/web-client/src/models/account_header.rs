use miden_client::account::AccountHeader as NativeAccountHeader;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::felt::Felt;
use super::word::Word;

/// Commitment-bearing header for an account.
#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountHeader(NativeAccountHeader);

#[wasm_bindgen]
impl AccountHeader {
    /// Returns the full account commitment.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns the account ID.
    pub fn id(&self) -> AccountId {
        self.0.id().into()
    }

    /// Returns the current nonce.
    pub fn nonce(&self) -> Felt {
        self.0.nonce().into()
    }

    /// Returns the vault commitment.
    #[wasm_bindgen(js_name = "vaultCommitment")]
    pub fn vault_commitment(&self) -> Word {
        self.0.vault_root().into()
    }

    /// Returns the storage commitment.
    #[wasm_bindgen(js_name = "storageCommitment")]
    pub fn storage_commitment(&self) -> Word {
        self.0.storage_commitment().into()
    }

    /// Returns the code commitment.
    #[wasm_bindgen(js_name = "codeCommitment")]
    pub fn code_commitment(&self) -> Word {
        self.0.code_commitment().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountHeader> for AccountHeader {
    fn from(native_account_header: NativeAccountHeader) -> Self {
        AccountHeader(native_account_header)
    }
}

impl From<&NativeAccountHeader> for AccountHeader {
    fn from(native_account_header: &NativeAccountHeader) -> Self {
        AccountHeader(native_account_header.clone())
    }
}
