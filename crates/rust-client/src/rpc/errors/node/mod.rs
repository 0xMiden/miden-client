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
use thiserror::Error;
pub use transaction::AddTransactionError;

use crate::rpc::NodeRpcClientEndpoint;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
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

/// Parses the application-level error code into a typed error for the given endpoint.
///
/// Returns `None` if details are empty or if the endpoint doesn't have typed errors.
pub fn parse_node_error(
    endpoint: &NodeRpcClientEndpoint,
    details: &[u8],
    message: &str,
) -> Option<NodeRpcError> {
    let code = *details.first()?;

    match endpoint {
        NodeRpcClientEndpoint::SubmitProvenTx => {
            Some(NodeRpcError::AddTransaction(AddTransactionError::from_code(code, message)))
        },
        NodeRpcClientEndpoint::GetBlockHeaderByNumber => {
            Some(NodeRpcError::GetBlockHeader(GetBlockHeaderError::from_code(code, message)))
        },
        NodeRpcClientEndpoint::GetBlockByNumber => {
            Some(NodeRpcError::GetBlockByNumber(GetBlockByNumberError::from_code(code, message)))
        },
        NodeRpcClientEndpoint::SyncNotes => {
            Some(NodeRpcError::NoteSync(NoteSyncError::from_code(code, message)))
        },
        NodeRpcClientEndpoint::SyncNullifiers => {
            Some(NodeRpcError::SyncNullifiers(SyncNullifiersError::from_code(code, message)))
        },
        NodeRpcClientEndpoint::SyncAccountVault => {
            Some(NodeRpcError::SyncAccountVault(SyncAccountVaultError::from_code(code, message)))
        },
        NodeRpcClientEndpoint::SyncStorageMaps => Some(NodeRpcError::SyncStorageMaps(
            SyncAccountStorageMapsError::from_code(code, message),
        )),
        NodeRpcClientEndpoint::SyncTransactions => {
            Some(NodeRpcError::SyncTransactions(SyncTransactionsError::from_code(code, message)))
        },
        NodeRpcClientEndpoint::GetNotesById => {
            Some(NodeRpcError::GetNotesById(GetNotesByIdError::from_code(code, message)))
        },
        NodeRpcClientEndpoint::GetNoteScriptByRoot => Some(NodeRpcError::GetNoteScriptByRoot(
            GetNoteScriptByRootError::from_code(code, message),
        )),
        NodeRpcClientEndpoint::CheckNullifiers => {
            Some(NodeRpcError::CheckNullifiers(CheckNullifiersError::from_code(code, message)))
        },
        // These endpoints don't have typed errors from the node
        NodeRpcClientEndpoint::GetAccount
        | NodeRpcClientEndpoint::SyncState
        | NodeRpcClientEndpoint::Status
        | NodeRpcClientEndpoint::GetLimits => None,
    }
}
