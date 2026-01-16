// ACCOUNT RECORD
// ================================================================================================
use alloc::vec::Vec;
use core::fmt::Display;

use miden_objects::Word;
use miden_objects::account::{Account, AccountId};
use miden_objects::address::Address;

/// Represents a stored account state along with its status.
///
/// The account should be stored in the database with its parts normalized. Meaning that the
/// account header, vault, storage and code are stored separately. This is done to avoid data
/// duplication as the header can reference the same elements if they have equal roots.
#[derive(Debug)]
pub struct AccountRecord {
    /// Full account object.
    account: Account,
    /// Status of the tracked account.
    status: AccountStatus,
    /// Addresses by which this account can be referenced.
    addresses: Vec<Address>,
}

impl AccountRecord {
    pub fn new(account: Account, status: AccountStatus, addresses: Vec<Address>) -> Self {
        // TODO: remove this?
        #[cfg(debug_assertions)]
        {
            let account_seed = account.seed();
            debug_assert_eq!(account_seed, status.seed().copied(), "account seed mismatch");
        }

        Self { account, status, addresses }
    }

    pub fn account(&self) -> &Account {
        &self.account
    }

    pub fn status(&self) -> &AccountStatus {
        &self.status
    }

    pub fn is_locked(&self) -> bool {
        self.status.is_locked()
    }

    pub fn seed(&self) -> Option<Word> {
        self.account.seed()
    }

    pub fn addresses(&self) -> &Vec<Address> {
        &self.addresses
    }
}

impl From<AccountRecord> for Account {
    fn from(record: AccountRecord) -> Self {
        record.account
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
    /// The seed is preserved for private accounts with nonce=0 that need
    /// reconstruction via `Account::new()`.
    Locked { seed: Option<Word> },
}

impl AccountStatus {
    pub fn is_locked(&self) -> bool {
        matches!(self, AccountStatus::Locked { .. })
    }

    pub fn seed(&self) -> Option<&Word> {
        match self {
            AccountStatus::New { seed } => Some(seed),
            AccountStatus::Locked { seed } => seed.as_ref(),
            AccountStatus::Tracked => None,
        }
    }
}

impl Display for AccountStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            AccountStatus::New { .. } => write!(f, "New"),
            AccountStatus::Tracked => write!(f, "Tracked"),
            AccountStatus::Locked { .. } => write!(f, "Locked"),
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
