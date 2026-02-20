use miden_client::account::AccountHeader as NativeAccountHeader;
use crate::prelude::*;

use super::account_id::AccountId;
use super::felt::Felt;
use super::word::Word;

/// A header of an account which contains information that succinctly describes the state of the
/// components of the account.
///
/// The account header is composed of:
/// - `id`: the account ID (`AccountId`).
/// - `nonce`: the nonce of the account.
/// - `vault_root`: a commitment to the account's vault (`AssetVault`).
/// - `storage_commitment`: a commitment to the account's storage (`AccountStorage`).
/// - `code_commitment`: a commitment to the account's code (`AccountCode`).
#[derive(Clone)]
#[bindings]
pub struct AccountHeader(NativeAccountHeader);

#[bindings]
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
    pub fn vault_commitment(&self) -> Word {
        self.0.vault_root().into()
    }

    /// Returns the storage commitment.
    pub fn storage_commitment(&self) -> Word {
        self.0.storage_commitment().into()
    }

    /// Returns the code commitment.
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
