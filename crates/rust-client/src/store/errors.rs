use alloc::string::String;
use core::num::TryFromIntError;

use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;
use miden_protocol::crypto::merkle::MerkleError;
use miden_protocol::crypto::merkle::mmr::MmrError;
use miden_protocol::crypto::merkle::smt::SmtProofError;
use miden_protocol::errors::{
    AccountError,
    AccountIdError,
    AddressError,
    AssetError,
    AssetVaultError,
    NoteError,
    StorageMapError,
    TransactionScriptError,
};
use miden_protocol::utils::{DeserializationError, HexParseError};
use miden_protocol::{Word, WordError};
use miden_tx::DataStoreError;
use thiserror::Error;

use super::note_record::NoteRecordError;

// STORE ERROR
// ================================================================================================

/// Errors generated from the store.
#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum StoreError {
    #[error("asset error")]
    AssetError(#[from] AssetError),
    #[error("asset vault error")]
    AssetVaultError(#[from] AssetVaultError),
    #[error("account code data with root {0} not found")]
    AccountCodeDataNotFound(Word),
    #[error("account data wasn't found for account id {0}")]
    AccountDataNotFound(AccountId),
    #[error("account error")]
    AccountError(#[from] AccountError),
    #[error("address error")]
    AddressError(#[from] AddressError),
    #[error("account id error")]
    AccountIdError(#[from] AccountIdError),
    #[error("account commitment mismatch for account {0}")]
    AccountCommitmentMismatch(AccountId),
    #[error("public key {0} not found")]
    AccountKeyNotFound(String),
    #[error("account storage data with root {0} not found")]
    AccountStorageRootNotFound(Word),
    #[error("account storage data with index {0} not found")]
    AccountStorageIndexNotFound(usize),
    #[error("block header for block {0} not found")]
    BlockHeaderNotFound(BlockNumber),
    #[error("partial blockchain node at index {0} not found")]
    PartialBlockchainNodeNotFound(u64),
    #[error("error deserializing data from the store")]
    DataDeserializationError(#[from] DeserializationError),
    #[error("database-related non-query error: {0}")]
    DatabaseError(String),
    #[error("error parsing hex")]
    HexParseError(#[from] HexParseError),
    #[error("failed to convert int")]
    InvalidInt(#[from] TryFromIntError),
    #[error("note record error")]
    NoteRecordError(#[from] NoteRecordError),
    #[error("error in merkle store")]
    MerkleStoreError(#[from] MerkleError),
    #[error("error constructing mmr")]
    MmrError(#[from] MmrError),
    #[error("inclusion proof creation error")]
    NoteInclusionProofError(#[from] NoteError),
    #[error("note tag {0} is already being tracked")]
    NoteTagAlreadyTracked(u64),
    #[error("note transport cursor not found")]
    NoteTransportCursorNotFound,
    #[error("note script with root {0} not found")]
    NoteScriptNotFound(String),
    #[error("failed to parse data retrieved from the database: {0}")]
    ParsingError(String),
    #[error("failed to retrieve data from the database: {0}")]
    QueryError(String),
    #[error("error with the SMT proof")]
    SmtProofError(#[from] SmtProofError),
    #[error("error with a storage map")]
    StorageMapError(#[from] StorageMapError),
    #[error("error instantiating transaction script")]
    TransactionScriptError(#[from] TransactionScriptError),
    #[error("account vault data for root {0} not found")]
    VaultDataNotFound(Word),
    #[error("failed to parse word: {0}")]
    WordError(#[from] WordError),
}

impl From<StoreError> for DataStoreError {
    fn from(value: StoreError) -> Self {
        match value {
            StoreError::AccountDataNotFound(account_id) => {
                DataStoreError::AccountNotFound(account_id)
            },
            err => DataStoreError::other_with_source("store error", err),
        }
    }
}
