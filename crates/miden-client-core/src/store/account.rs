// ACCOUNT RECORD
// ================================================================================================
use alloc::vec::Vec;
use core::fmt::Display;

use miden_objects::account::{Account, AccountCode, AccountId, PartialAccount};
use miden_objects::address::Address;
use miden_objects::{Felt, Word};

use crate::ClientError;

#[derive(Debug)]
pub enum AccountRecordData {
    Full(Account),
    Partial(PartialAccount),
}

/// Represents a stored account state along with its status.
///
/// The account should be stored in the database with its parts normalized. Meaning that the
/// account header, vault, storage and code are stored separately. This is done to avoid data
/// duplication as the header can reference the same elements if they have equal roots.
#[derive(Debug)]
pub struct AccountRecord {
    /// Full account object.
    account_data: AccountRecordData,
    /// Status of the tracked account.
    status: AccountStatus,
    /// Addresses by which this account can be referenced.
    addresses: Vec<Address>,
}

impl AccountRecord {
    pub fn new(
        account_data: AccountRecordData,
        status: AccountStatus,
        addresses: Vec<Address>,
    ) -> Self {
        // TODO: remove this?
        #[cfg(debug_assertions)]
        {
            let account_seed = match &account_data {
                AccountRecordData::Full(acc) => acc.seed(),
                AccountRecordData::Partial(acc) => acc.seed(),
            };
            debug_assert_eq!(account_seed, status.seed().copied(), "account seed mismatch");
        }

        Self { account_data, status, addresses }
    }

    pub fn id(&self) -> AccountId {
        match &self.account_data {
            AccountRecordData::Full(acc) => acc.id(),
            AccountRecordData::Partial(acc) => acc.id(),
        }
    }

    pub fn account_data(&self) -> &AccountRecordData {
        &self.account_data
    }

    pub fn status(&self) -> &AccountStatus {
        &self.status
    }

    pub fn is_locked(&self) -> bool {
        self.status.is_locked()
    }

    pub fn seed(&self) -> Option<Word> {
        match &self.account_data {
            AccountRecordData::Full(acc) => acc.seed(),
            AccountRecordData::Partial(acc) => acc.seed(),
        }
    }

    pub fn nonce(&self) -> Felt {
        match &self.account_data {
            AccountRecordData::Full(acc) => acc.nonce(),
            AccountRecordData::Partial(acc) => acc.nonce(),
        }
    }

    pub fn commitment(&self) -> Felt {
        match &self.account_data {
            AccountRecordData::Full(acc) => acc.nonce(),
            AccountRecordData::Partial(acc) => acc.nonce(),
        }
    }

    pub fn addresses(&self) -> &Vec<Address> {
        &self.addresses
    }

    pub fn code(&self) -> AccountCode {
        match &self.account_data {
            AccountRecordData::Full(acc) => acc.code().clone(),
            AccountRecordData::Partial(acc) => acc.code().clone(),
        }
    }
}

impl TryFrom<AccountRecord> for Account {
    type Error = ClientError;

    fn try_from(value: AccountRecord) -> Result<Self, Self::Error> {
        match value.account_data {
            AccountRecordData::Full(acc) => Ok(acc),
            AccountRecordData::Partial(acc) => Err(ClientError::AccountRecordNotFull(acc.id())),
        }
    }
}

impl TryFrom<AccountRecord> for PartialAccount {
    type Error = ClientError;

    fn try_from(value: AccountRecord) -> Result<Self, Self::Error> {
        match value.account_data {
            AccountRecordData::Partial(acc) => Ok(acc),
            AccountRecordData::Full(acc) => Err(ClientError::AccountRecordNotPartial(acc.id())),
        }
    }
}

// ACCOUNT STATUS
// ================================================================================================

/// Represents the status of an account tracked by the client.
///
/// The status of an account may change by local or external factors.
#[derive(Debug)]
pub enum AccountStatus {
    /// The account is new and hasn't been used yet. The seed used to create the account is
    /// stored in this state.
    New { seed: Word },
    /// The account is tracked by the node and was used at least once.
    Tracked,
    /// The local account state doesn't match the node's state, rendering it unusable. Only used
    /// for private accounts.
    Locked,
}

impl AccountStatus {
    pub fn is_locked(&self) -> bool {
        matches!(self, AccountStatus::Locked)
    }

    pub fn seed(&self) -> Option<&Word> {
        match self {
            AccountStatus::New { seed } => Some(seed),
            _ => None,
        }
    }
}

impl Display for AccountStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            AccountStatus::New { .. } => write!(f, "New"),
            AccountStatus::Tracked => write!(f, "Tracked"),
            AccountStatus::Locked => write!(f, "Locked"),
        }
    }
}

// ACCOUNT UPDATES
// ================================================================================================

/// Contains account changes to apply to the store.
pub struct AccountUpdates {
    /// Updated public accounts.
    updated_public_accounts: Vec<Account>,
    /// Network account commitments that don't match the current tracked state for private
    /// accounts.
    mismatched_private_accounts: Vec<(AccountId, Word)>,
}

impl AccountUpdates {
    /// Creates a new instance of `AccountUpdates`.
    pub fn new(
        updated_public_accounts: Vec<Account>,
        mismatched_private_accounts: Vec<(AccountId, Word)>,
    ) -> Self {
        Self {
            updated_public_accounts,
            mismatched_private_accounts,
        }
    }

    /// Returns the updated public accounts.
    pub fn updated_public_accounts(&self) -> &[Account] {
        &self.updated_public_accounts
    }

    /// Returns the mismatched private accounts.
    pub fn mismatched_private_accounts(&self) -> &[(AccountId, Word)] {
        &self.mismatched_private_accounts
    }
}
