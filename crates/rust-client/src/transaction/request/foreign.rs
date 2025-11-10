//! Contains structures and functions related to FPI (Foreign Procedure Invocation) transactions.
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::Ordering;

use miden_objects::account::{
    AccountId, PartialAccount, PartialStorage, PartialStorageMap, StorageMap,
};
use miden_objects::asset::{AssetVault, PartialVault};
use miden_objects::transaction::AccountInputs;
use miden_tx::utils::{Deserializable, DeserializationError, Serializable};

use super::TransactionRequestError;
use crate::rpc::domain::account::{AccountDetails, AccountProof, AccountStorageRequirements};

// FOREIGN ACCOUNT
// ================================================================================================

/// Account types for foreign procedure invocation.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum ForeignAccount {
    /// Public account data will be retrieved from the network at execution time, based on the
    /// account ID. The second element of the tuple indicates which storage slot indices
    /// and map keys are desired to be retrieved.
    Public(AccountId, AccountStorageRequirements),
    /// Private account data requires [`PartialAccount`] to be passed. An account witness
    /// will be retrieved from the network at execution time so that it can be used as inputs to
    /// the transaction kernel.
    Private(PartialAccount),
}

impl ForeignAccount {
    /// Creates a new [`ForeignAccount::Public`]. The account's components (code, storage header and
    /// inclusion proof) will be retrieved at execution time, alongside particular storage slot
    /// maps correspondent to keys passed in `indices`.
    pub fn public(
        account_id: AccountId,
        storage_requirements: AccountStorageRequirements,
    ) -> Result<Self, TransactionRequestError> {
        if !account_id.is_public() {
            return Err(TransactionRequestError::InvalidForeignAccountId(account_id));
        }

        Ok(Self::Public(account_id, storage_requirements))
    }

    /// Creates a new [`ForeignAccount::Private`]. A proof of the account's inclusion will be
    /// retrieved at execution time.
    pub fn private(account: impl Into<PartialAccount>) -> Result<Self, TransactionRequestError> {
        let partial_account: PartialAccount = account.into();
        if partial_account.id().is_public() {
            return Err(TransactionRequestError::InvalidForeignAccountId(partial_account.id()));
        }

        Ok(Self::Private(partial_account))
    }

    pub fn storage_slot_requirements(&self) -> AccountStorageRequirements {
        match self {
            ForeignAccount::Public(_, account_storage_requirements) => {
                account_storage_requirements.clone()
            },
            ForeignAccount::Private(_) => AccountStorageRequirements::default(),
        }
    }

    /// Returns the foreign account's [`AccountId`].
    pub fn account_id(&self) -> AccountId {
        match self {
            ForeignAccount::Public(account_id, _) => *account_id,
            ForeignAccount::Private(partial_account) => partial_account.id(),
        }
    }
}

impl Ord for ForeignAccount {
    fn cmp(&self, other: &Self) -> Ordering {
        self.account_id().cmp(&other.account_id())
    }
}

impl PartialOrd for ForeignAccount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Serializable for ForeignAccount {
    fn write_into<W: miden_tx::utils::ByteWriter>(&self, target: &mut W) {
        match self {
            ForeignAccount::Public(account_id, storage_requirements) => {
                target.write(0u8);
                account_id.write_into(target);
                storage_requirements.write_into(target);
            },
            ForeignAccount::Private(partial_account) => {
                target.write(1u8);
                partial_account.write_into(target);
            },
        }
    }
}

impl Deserializable for ForeignAccount {
    fn read_from<R: miden_tx::utils::ByteReader>(
        source: &mut R,
    ) -> Result<Self, miden_tx::utils::DeserializationError> {
        let account_type: u8 = source.read_u8()?;
        match account_type {
            0 => {
                let account_id = AccountId::read_from(source)?;
                let storage_requirements = AccountStorageRequirements::read_from(source)?;
                Ok(ForeignAccount::Public(account_id, storage_requirements))
            },
            1 => {
                let foreign_inputs = PartialAccount::read_from(source)?;
                Ok(ForeignAccount::Private(foreign_inputs))
            },
            _ => Err(DeserializationError::InvalidValue("Invalid account type".to_string())),
        }
    }
}

impl TryFrom<AccountProof> for AccountInputs {
    type Error = TransactionRequestError;

    fn try_from(value: AccountProof) -> Result<Self, Self::Error> {
        let (witness, account_details) = value.into_parts();

        if let Some(AccountDetails {
            header: account_header,
            code,
            storage_details,
            vault_details,
        }) = account_details
        {
            // discard slot indices - not needed for execution
            let account_storage_map_details = storage_details.map_details;
            let mut storage_map_proofs = Vec::with_capacity(account_storage_map_details.len());
            for account_storage_detail in account_storage_map_details {
                let storage_entries_iter =
                    account_storage_detail.entries.iter().map(|e| (e.key, e.value));
                let partial_storage = PartialStorageMap::new_full(
                    StorageMap::with_entries(storage_entries_iter)
                        .map_err(TransactionRequestError::StorageMapError)?,
                );
                storage_map_proofs.push(partial_storage);
            }

            let vault = AssetVault::new(&vault_details.assets)?;
            return Ok(AccountInputs::new(
                PartialAccount::new(
                    account_header.id(),
                    account_header.nonce(),
                    code,
                    PartialStorage::new(storage_details.header, storage_map_proofs.into_iter())?,
                    PartialVault::new_full(vault),
                    None,
                )?,
                witness,
            ));
        }
        Err(TransactionRequestError::ForeignAccountDataMissing)
    }
}
