use miden_client::rpc::domain::account::FetchedAccount as NativeFetchedAccount;
use wasm_bindgen::prelude::*;

use super::account::Account;
use super::account_id::AccountId;
use super::word::Word;

/// Account details returned by the node.
#[derive(Clone)]
#[wasm_bindgen]
pub struct FetchedAccount {
    account_id: AccountId,
    commitment: Word,
    last_block_num: u32,
    account: Option<Account>,
}

#[wasm_bindgen]
impl FetchedAccount {
    /// Returns the account ID.
    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    /// Returns the account commitment reported by the node.
    pub fn commitment(&self) -> Word {
        self.commitment.clone()
    }

    /// Returns the last block height where the account was updated.
    #[wasm_bindgen(js_name = "lastBlockNum")]
    pub fn last_block_num(&self) -> u32 {
        self.last_block_num
    }

    /// Returns the full account data when the account is public.
    pub fn account(&self) -> Option<Account> {
        self.account.clone()
    }

    /// Returns true when the account is public.
    #[wasm_bindgen(js_name = "isPublic")]
    pub fn is_public(&self) -> bool {
        self.account_id.is_public()
    }

    /// Returns true when the account is private.
    #[wasm_bindgen(js_name = "isPrivate")]
    pub fn is_private(&self) -> bool {
        self.account_id.is_private()
    }

    /// Returns true when the account is a network account.
    #[wasm_bindgen(js_name = "isNetwork")]
    pub fn is_network(&self) -> bool {
        self.account_id.is_network()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeFetchedAccount> for FetchedAccount {
    fn from(native_account: NativeFetchedAccount) -> Self {
        match native_account {
            NativeFetchedAccount::Private(account_id, summary) => FetchedAccount {
                account_id: account_id.into(),
                commitment: summary.commitment.into(),
                last_block_num: summary.last_block_num,
                account: None,
            },
            NativeFetchedAccount::Public(account, summary) => {
                let account_id = account.id().into();
                let account = (*account).into();
                FetchedAccount {
                    account_id,
                    commitment: summary.commitment.into(),
                    last_block_num: summary.last_block_num,
                    account: Some(account),
                }
            },
        }
    }
}
