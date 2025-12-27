use miden_client::rpc::domain::account::FetchedAccount as NativeFetchedAccount;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::models::account::Account;
use crate::models::account_id::AccountId;
use crate::models::word::Word;

/// Describes the response from the `GetAccountDetails` endpoint.
///
/// The content varies based on account visibility:
/// - **Public or Network accounts**: Contains the complete [`Account`] details, as these are stored
///   on-chain
/// - **Private accounts**: Contains only the state commitment, since full account data is stored
///   off-chain
#[wasm_bindgen]
pub struct FetchedAccount(NativeFetchedAccount);

#[wasm_bindgen]
impl FetchedAccount {
    /// Returns true if the fetched account is private
    #[wasm_bindgen(js_name = "isPrivate")]
    pub fn is_private(&self) -> bool {
        matches!(&self.0, NativeFetchedAccount::Private(_, _))
    }

    /// Returns true if the fetched account is public
    #[wasm_bindgen(js_name = "isPublic")]
    pub fn is_public(&self) -> bool {
        matches!(&self.0, NativeFetchedAccount::Public(_, _))
    }

    /// Returns the associated [`Account`] if the account is public, otherwise none
    pub fn account(&self) -> Option<Account> {
        self.0.account().map(Into::into)
    }

    /// Returns the account identifier
    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.0.account_id().into()
    }

    /// Returns the account update summary commitment
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeFetchedAccount> for FetchedAccount {
    fn from(native_account: NativeFetchedAccount) -> Self {
        FetchedAccount(native_account)
    }
}

impl From<FetchedAccount> for NativeFetchedAccount {
    fn from(fetched_account: FetchedAccount) -> Self {
        fetched_account.0
    }
}
