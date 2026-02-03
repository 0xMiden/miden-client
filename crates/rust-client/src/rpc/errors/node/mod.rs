//! Node RPC error types.
//!
//! This module contains typed error types for all node RPC endpoints that support
//! application-level error codes. These allow programmatic handling of specific
//! error conditions instead of parsing error messages.
//!
//! Error codes are parsed from the gRPC status details sent by the node.

use core::fmt;

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

/// Application-level errors returned by the node in gRPC status details.
///
/// These typed errors allow programmatic handling of specific error conditions
/// instead of parsing error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRpcError {
    /// Error from the `SubmitProvenTransaction` endpoint
    AddTransaction(AddTransactionError),
    /// Error from the `GetBlockHeaderByNumber` endpoint
    GetBlockHeader(GetBlockHeaderError),
    /// Error from the `GetBlockByNumber` endpoint
    GetBlockByNumber(GetBlockByNumberError),
    /// Error from the `SyncNotes` endpoint
    NoteSync(NoteSyncError),
    /// Error from the `SyncNullifiers` endpoint
    SyncNullifiers(SyncNullifiersError),
    /// Error from the `SyncAccountVault` endpoint
    SyncAccountVault(SyncAccountVaultError),
    /// Error from the `SyncStorageMaps` endpoint
    SyncStorageMaps(SyncAccountStorageMapsError),
    /// Error from the `SyncTransactions` endpoint
    SyncTransactions(SyncTransactionsError),
    /// Error from the `GetNotesById` endpoint
    GetNotesById(GetNotesByIdError),
    /// Error from the `GetNoteScriptByRoot` endpoint
    GetNoteScriptByRoot(GetNoteScriptByRootError),
    /// Error from the `CheckNullifiers` endpoint
    CheckNullifiers(CheckNullifiersError),
}

impl fmt::Display for NodeRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddTransaction(err) => write!(f, "{err}"),
            Self::GetBlockHeader(err) => write!(f, "{err}"),
            Self::GetBlockByNumber(err) => write!(f, "{err}"),
            Self::NoteSync(err) => write!(f, "{err}"),
            Self::SyncNullifiers(err) => write!(f, "{err}"),
            Self::SyncAccountVault(err) => write!(f, "{err}"),
            Self::SyncStorageMaps(err) => write!(f, "{err}"),
            Self::SyncTransactions(err) => write!(f, "{err}"),
            Self::GetNotesById(err) => write!(f, "{err}"),
            Self::GetNoteScriptByRoot(err) => write!(f, "{err}"),
            Self::CheckNullifiers(err) => write!(f, "{err}"),
        }
    }
}

/// Parses application-level error codes from gRPC status details.
///
/// The node sends a single u8 byte in `status.details()` for certain endpoints.
/// This function extracts that byte and converts it to the appropriate typed error.
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
        | NodeRpcClientEndpoint::GetAccountStateDelta
        | NodeRpcClientEndpoint::SyncState
        | NodeRpcClientEndpoint::Status => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_node_error_submit_proven_tx() {
        let error = parse_node_error(&NodeRpcClientEndpoint::SubmitProvenTx, &[5]);
        assert_eq!(
            error,
            Some(NodeRpcError::AddTransaction(AddTransactionError::InvalidTransactionProof))
        );
    }

    #[test]
    fn test_parse_node_error_get_block_header() {
        let error = parse_node_error(&NodeRpcClientEndpoint::GetBlockHeaderByNumber, &[0]);
        assert_eq!(error, Some(NodeRpcError::GetBlockHeader(GetBlockHeaderError::Internal)));
    }

    #[test]
    fn test_parse_node_error_get_block_by_number() {
        let error = parse_node_error(&NodeRpcClientEndpoint::GetBlockByNumber, &[1]);
        assert_eq!(
            error,
            Some(NodeRpcError::GetBlockByNumber(GetBlockByNumberError::DeserializationFailed))
        );
    }

    #[test]
    fn test_parse_node_error_sync_notes() {
        let error = parse_node_error(&NodeRpcClientEndpoint::SyncNotes, &[1]);
        assert_eq!(error, Some(NodeRpcError::NoteSync(NoteSyncError::InvalidBlockRange)));
    }

    #[test]
    fn test_parse_node_error_sync_nullifiers() {
        let error = parse_node_error(&NodeRpcClientEndpoint::SyncNullifiers, &[2]);
        assert_eq!(
            error,
            Some(NodeRpcError::SyncNullifiers(SyncNullifiersError::InvalidPrefixLength))
        );
    }

    #[test]
    fn test_parse_node_error_sync_account_vault() {
        let error = parse_node_error(&NodeRpcClientEndpoint::SyncAccountVault, &[3]);
        assert_eq!(
            error,
            Some(NodeRpcError::SyncAccountVault(SyncAccountVaultError::AccountNotPublic))
        );
    }

    #[test]
    fn test_parse_node_error_sync_storage_maps() {
        let error = parse_node_error(&NodeRpcClientEndpoint::SyncStorageMaps, &[4]);
        assert_eq!(
            error,
            Some(NodeRpcError::SyncStorageMaps(SyncAccountStorageMapsError::AccountNotPublic))
        );
    }

    #[test]
    fn test_parse_node_error_sync_transactions() {
        let error = parse_node_error(&NodeRpcClientEndpoint::SyncTransactions, &[4]);
        assert_eq!(
            error,
            Some(NodeRpcError::SyncTransactions(SyncTransactionsError::WitnessError))
        );
    }

    #[test]
    fn test_parse_node_error_get_notes_by_id() {
        let error = parse_node_error(&NodeRpcClientEndpoint::GetNotesById, &[2]);
        assert_eq!(error, Some(NodeRpcError::GetNotesById(GetNotesByIdError::NoteNotFound)));
    }

    #[test]
    fn test_parse_node_error_get_note_script_by_root() {
        let error = parse_node_error(&NodeRpcClientEndpoint::GetNoteScriptByRoot, &[2]);
        assert_eq!(
            error,
            Some(NodeRpcError::GetNoteScriptByRoot(GetNoteScriptByRootError::ScriptNotFound))
        );
    }

    #[test]
    fn test_parse_node_error_check_nullifiers() {
        let error = parse_node_error(&NodeRpcClientEndpoint::CheckNullifiers, &[1]);
        assert_eq!(
            error,
            Some(NodeRpcError::CheckNullifiers(CheckNullifiersError::DeserializationFailed))
        );
    }

    #[test]
    fn test_parse_node_error_empty_details() {
        let error = parse_node_error(&NodeRpcClientEndpoint::SubmitProvenTx, &[]);
        assert_eq!(error, None);
    }

    #[test]
    fn test_parse_node_error_unsupported_endpoints() {
        assert_eq!(parse_node_error(&NodeRpcClientEndpoint::GetAccount, &[1]), None);
        assert_eq!(parse_node_error(&NodeRpcClientEndpoint::GetAccountStateDelta, &[1]), None);
        assert_eq!(parse_node_error(&NodeRpcClientEndpoint::SyncState, &[1]), None);
        assert_eq!(parse_node_error(&NodeRpcClientEndpoint::Status, &[1]), None);
    }
}
