use miden_client::note::BlockNumber;
use miden_client::rpc::domain::account::FetchedAccount as NativeFetchedAccount;

use crate::prelude::*;
use super::account::Account;
use super::account_id::AccountId;
use super::word::Word;

/// Account details returned by the node.
#[bindings]
#[derive(Clone)]
pub struct FetchedAccount {
    account_id: AccountId,
    commitment: Word,
    last_block_num: BlockNumber,
    account: Option<Account>,
}

#[bindings]
impl FetchedAccount {
    /// Returns the account ID.
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    /// Returns the account commitment reported by the node.
    pub fn commitment(&self) -> Word {
        self.commitment.clone()
    }

    /// Returns the last block height where the account was updated.
    #[bindings]
    pub fn last_block_num(&self) -> u32 {
        self.last_block_num.as_u32()
    }

    /// Returns the full account data when the account is public.
    pub fn account(&self) -> Option<Account> {
        self.account.clone()
    }

    /// Returns true when the account is public.
    pub fn is_public(&self) -> bool {
        self.account_id.is_public()
    }

    /// Returns true when the account is private.
    pub fn is_private(&self) -> bool {
        self.account_id.is_private()
    }

    /// Returns true when the account is a network account.
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
