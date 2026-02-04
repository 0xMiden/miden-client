//! Typed error codes parsed from gRPC status details sent by the node.

mod block;
mod note;
mod sync;
mod transaction;

pub use block::{GetBlockByNumberError, GetBlockHeaderError};
pub use note::{CheckNullifiersError, GetNoteScriptByRootError, GetNotesByIdError};
pub use sync::{
    NoteSyncError,
    SyncAccountStorageMapsError,
    SyncAccountVaultError,
    SyncNullifiersError,
    SyncTransactionsError,
};
pub use transaction::AddTransactionError;

use crate::rpc::NodeRpcClientEndpoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NodeRpcError {
    /// Error from the `SubmitProvenTransaction` endpoint
    #[error(transparent)]
    AddTransaction(#[from] AddTransactionError),
    /// Error from the `GetBlockHeaderByNumber` endpoint
    #[error(transparent)]
    GetBlockHeader(#[from] GetBlockHeaderError),
    /// Error from the `GetBlockByNumber` endpoint
    #[error(transparent)]
    GetBlockByNumber(#[from] GetBlockByNumberError),
    /// Error from the `SyncNotes` endpoint
    #[error(transparent)]
    NoteSync(#[from] NoteSyncError),
    /// Error from the `SyncNullifiers` endpoint
    #[error(transparent)]
    SyncNullifiers(#[from] SyncNullifiersError),
    /// Error from the `SyncAccountVault` endpoint
    #[error(transparent)]
    SyncAccountVault(#[from] SyncAccountVaultError),
    /// Error from the `SyncStorageMaps` endpoint
    #[error(transparent)]
    SyncStorageMaps(#[from] SyncAccountStorageMapsError),
    /// Error from the `SyncTransactions` endpoint
    #[error(transparent)]
    SyncTransactions(#[from] SyncTransactionsError),
    /// Error from the `GetNotesById` endpoint
    #[error(transparent)]
    GetNotesById(#[from] GetNotesByIdError),
    /// Error from the `GetNoteScriptByRoot` endpoint
    #[error(transparent)]
    GetNoteScriptByRoot(#[from] GetNoteScriptByRootError),
    /// Error from the `CheckNullifiers` endpoint
    #[error(transparent)]
    CheckNullifiers(#[from] CheckNullifiersError),
}

/// Parses error code from `status.details()` (a single u8 byte).
pub fn parse_node_error(endpoint: &NodeRpcClientEndpoint, details: &[u8]) -> Option<NodeRpcError> {
    let code = *details.first()?;
    match endpoint {
        NodeRpcClientEndpoint::SubmitProvenTx => {
            Some(NodeRpcError::AddTransaction(AddTransactionError::from(code)))
        },
        NodeRpcClientEndpoint::GetBlockHeaderByNumber => {
            Some(NodeRpcError::GetBlockHeader(GetBlockHeaderError::from(code)))
        },
        NodeRpcClientEndpoint::GetBlockByNumber => {
            Some(NodeRpcError::GetBlockByNumber(GetBlockByNumberError::from(code)))
        },
        NodeRpcClientEndpoint::SyncNotes => Some(NodeRpcError::NoteSync(NoteSyncError::from(code))),
        NodeRpcClientEndpoint::SyncNullifiers => {
            Some(NodeRpcError::SyncNullifiers(SyncNullifiersError::from(code)))
        },
        NodeRpcClientEndpoint::SyncAccountVault => {
            Some(NodeRpcError::SyncAccountVault(SyncAccountVaultError::from(code)))
        },
        NodeRpcClientEndpoint::SyncStorageMaps => {
            Some(NodeRpcError::SyncStorageMaps(SyncAccountStorageMapsError::from(code)))
        },
        NodeRpcClientEndpoint::SyncTransactions => {
            Some(NodeRpcError::SyncTransactions(SyncTransactionsError::from(code)))
        },
        NodeRpcClientEndpoint::GetNotesById => {
            Some(NodeRpcError::GetNotesById(GetNotesByIdError::from(code)))
        },
        NodeRpcClientEndpoint::GetNoteScriptByRoot => {
            Some(NodeRpcError::GetNoteScriptByRoot(GetNoteScriptByRootError::from(code)))
        },
        NodeRpcClientEndpoint::CheckNullifiers => {
            Some(NodeRpcError::CheckNullifiers(CheckNullifiersError::from(code)))
        },
        // These endpoints don't have typed errors from the node
        NodeRpcClientEndpoint::GetAccount
        | NodeRpcClientEndpoint::SyncState
        | NodeRpcClientEndpoint::Status => None,
    }
}
