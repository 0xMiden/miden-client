//! Provides an interface for the client to communicate with a Miden node using
//! Remote Procedure Calls (RPC).
//!
//! This module defines the [`NodeRpcClient`] trait which abstracts calls to the RPC protocol used
//! to:
//!
//! - Submit proven transactions.
//! - Retrieve block headers (optionally with MMR proofs).
//! - Sync state updates (including notes, nullifiers, and account updates).
//! - Fetch details for specific notes and accounts.
//!
//! In addition, the module provides implementations for different environments (e.g. tonic-based or
//! web-based) via feature flags ( `tonic` and `web-tonic`).
//!
//! ## Example
//!
//! ```no_run
//! # use miden_client::rpc::{Endpoint, NodeRpcClient, TonicRpcClient};
//! # use miden_objects::block::BlockNumber;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a Tonic RPC client instance (assumes default endpoint configuration).
//! let endpoint = Endpoint::new("https".into(), "localhost".into(), Some(57291));
//! let mut rpc_client = TonicRpcClient::new(&endpoint, 1000);
//!
//! // Fetch the latest block header (by passing None).
//! let (block_header, mmr_proof) = rpc_client.get_block_header_by_number(None, true).await?;
//!
//! println!("Latest block number: {}", block_header.block_num());
//! if let Some(proof) = mmr_proof {
//!     println!("MMR proof received accordingly");
//! }
//!
//! #    Ok(())
//! # }
//! ```
//! The client also makes use of this component in order to communicate with the node.
//!
//! For further details and examples, see the documentation for the individual methods in the
//! [`NodeRpcClient`] trait.

use alloc::{boxed::Box, collections::BTreeSet, string::String, vec::Vec};
use core::{fmt, pin::Pin};

use domain::{
    account::{AccountDetails, AccountProofs},
    note::{NetworkNote, NoteSyncInfo},
    nullifier::NullifierUpdate,
    sync::StateSyncInfo,
};
use miden_objects::{
    account::{Account, AccountCode, AccountDelta, AccountHeader, AccountId},
    block::{BlockHeader, BlockNumber, ProvenBlock},
    crypto::merkle::{MmrProof, SmtProof},
    note::{NoteId, NoteTag, Nullifier},
    transaction::ProvenTransaction,
};

/// Contains domain types related to RPC requests and responses, as well as utility functions
/// for dealing with them.
pub mod domain;

mod errors;
pub use errors::RpcError;

mod endpoint;
pub use endpoint::Endpoint;

#[cfg(not(test))]
mod generated;
#[cfg(test)]
pub mod generated;

#[cfg(all(feature = "tonic", feature = "web-tonic"))]
compile_error!("features `tonic` and `web-tonic` are mutually exclusive");

#[cfg(any(feature = "tonic", feature = "web-tonic"))]
mod tonic_client;
#[cfg(any(feature = "tonic", feature = "web-tonic"))]
pub use tonic_client::TonicRpcClient;

use crate::{
    store::{InputNoteRecord, input_note_states::UnverifiedNoteState},
    transaction::ForeignAccount,
};

// NODE RPC CLIENT TRAIT
// ================================================================================================

/// Defines the interface for communicating with the Miden node.
///
/// The implementers are responsible for connecting to the Miden node, handling endpoint
/// requests/responses, and translating responses into domain objects relevant for each of the
/// endpoints.
pub trait NodeRpcClient {
    /// Given a Proven Transaction, send it to the node for it to be included in a future block
    /// using the `/SubmitProvenTransaction` RPC endpoint.
    fn submit_proven_transaction<'a>(
        &'a self,
        proven_transaction: ProvenTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<(), RpcError>> + 'a>>;

    /// Given a block number, fetches the block header corresponding to that height from the node
    /// using the `/GetBlockHeaderByNumber` endpoint.
    /// If `include_mmr_proof` is set to true and the function returns an `Ok`, the second value
    /// of the return tuple should always be Some(MmrProof).
    ///
    /// When `None` is provided, returns info regarding the latest block.
    fn get_block_header_by_number<'a>(
        &'a self,
        block_num: Option<BlockNumber>,
        include_mmr_proof: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(BlockHeader, Option<MmrProof>), RpcError>> + Send + 'a>>;

    /// Given a block number, fetches the block corresponding to that height from the node using
    /// the `/GetBlockByNumber` RPC endpoint.
    fn get_block_by_number<'a>(
        &'a self,
        block_num: BlockNumber,
    ) -> Pin<Box<dyn Future<Output = Result<ProvenBlock, RpcError>> + 'a>>;

    /// Fetches note-related data for a list of [NoteId] using the `/GetNotesById` rpc endpoint.
    ///
    /// For any NoteType::Private note, the return data is only the
    /// [miden_objects::note::NoteMetadata], whereas for NoteType::Onchain notes, the return
    /// data includes all details.
    fn get_notes_by_id<'a, 'b>(
        &'a self,
        note_ids: &'b [NoteId],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<NetworkNote>, RpcError>> + 'a>>
    where
        'b: 'a;

    /// Fetches info from the node necessary to perform a state sync using the
    /// `/SyncState` RPC endpoint.
    ///
    /// - `block_num` is the last block number known by the client. The returned [StateSyncInfo]
    ///   should contain data starting from the next block, until the first block which contains a
    ///   note of matching the requested tag, or the chain tip if there are no notes.
    /// - `account_ids` is a list of account IDs and determines the accounts the client is
    ///   interested in and should receive account updates of.
    /// - `note_tags` is a list of tags used to filter the notes the client is interested in, which
    ///   serves as a "note group" filter. Notice that you can't filter by a specific note ID.
    /// - `nullifiers_tags` similar to `note_tags`, is a list of tags used to filter the nullifiers
    ///   corresponding to some notes the client is interested in.
    fn sync_state<'a, 'b, 'c>(
        &'a self,
        block_num: BlockNumber,
        account_ids: &'b [AccountId],
        note_tags: &'c [NoteTag],
    ) -> Pin<Box<dyn Future<Output = Result<StateSyncInfo, RpcError>> + 'a>>
    where
        'b: 'a,
        'c: 'a;

    /// Fetches the current state of an account from the node using the `/GetAccountDetails` RPC
    /// endpoint.
    ///
    /// - `account_id` is the ID of the wanted account.
    fn get_account_details<'a>(
        &'a self,
        account_id: AccountId,
    ) -> Pin<Box<dyn Future<Output = Result<AccountDetails, RpcError>> + 'a>>;

    /// Fetches the notes related to the specified tags using the `/SyncNotes` RPC endpoint.
    ///
    /// - `block_num` is the last block number known by the client.
    /// - `note_tags` is a list of tags used to filter the notes the client is interested in.
    fn sync_notes<'a, 'b>(
        &'a self,
        block_num: BlockNumber,
        note_tags: &'b [NoteTag],
    ) -> Pin<Box<dyn Future<Output = Result<NoteSyncInfo, RpcError>> + 'a>>
    where
        'b: 'a;

    /// Fetches the nullifiers corresponding to a list of prefixes using the
    /// `/CheckNullifiersByPrefix` RPC endpoint.
    ///
    /// - `prefix` is a list of nullifiers prefixes to search for.
    /// - `block_num` is the block number to start the search from. Nullifiers created in this block
    ///   or the following blocks will be included.
    fn check_nullifiers_by_prefix<'a, 'b>(
        &'a self,
        prefix: &'b [u16],
        block_num: BlockNumber,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<NullifierUpdate>, RpcError>> + 'a>>
    where
        'b: 'a;

    /// Fetches the nullifier proofs corresponding to a list of nullifiers using the
    /// `/CheckNullifiers` RPC endpoint.
    fn check_nullifiers<'a, 'b>(
        &'a self,
        nullifiers: &'b [Nullifier],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SmtProof>, RpcError>> + 'a>>
    where
        'b: 'a;

    /// Fetches the account data needed to perform a Foreign Procedure Invocation (FPI) on the
    /// specified foreign accounts, using the `GetAccountProofs` endpoint.
    ///
    /// The `code_commitments` parameter is a list of known code commitments
    /// to prevent unnecessary data fetching. Returns the block number and the FPI account data. If
    /// one of the tracked accounts is not found in the node, the method will return an error.
    fn get_account_proofs<'a, 'b>(
        &'a self,
        account_storage_requests: &'b BTreeSet<ForeignAccount>,
        known_account_codes: Vec<AccountCode>,
    ) -> Pin<Box<dyn Future<Output = Result<AccountProofs, RpcError>> + 'a>>
    where
        'b: 'a;

    /// Fetches the account state delta for the specified account between the specified blocks
    /// using the `/GetAccountStateDelta` RPC endpoint.
    fn get_account_state_delta<'a>(
        &'a self,
        account_id: AccountId,
        from_block: BlockNumber,
        to_block: BlockNumber,
    ) -> Pin<Box<dyn Future<Output = Result<AccountDelta, RpcError>> + 'a>>;

    /// Fetches the commit height where the nullifier was consumed. If the nullifier isn't found,
    /// then `None` is returned.
    /// The `block_num` parameter is the block number to start the search from.
    ///
    /// The default implementation of this method uses [NodeRpcClient::check_nullifiers_by_prefix].
    fn get_nullifier_commit_height<'a, 'b>(
        &'a self,
        nullifier: &'b Nullifier,
        block_num: BlockNumber,
    ) -> Pin<Box<dyn Future<Output = Result<Option<u32>, RpcError>> + 'a>>
    where
        'b: 'a,
    {
        Box::pin(async move {
            let nullifiers =
                self.check_nullifiers_by_prefix(&[nullifier.prefix()], block_num).await?;

            Ok(nullifiers
                .iter()
                .find(|update| update.nullifier == *nullifier)
                .map(|update| update.block_num))
        })
    }

    /// Fetches public note-related data for a list of [NoteId] and builds [InputNoteRecord]s with
    /// it. If a note is not found or it's private, it is ignored and will not be included in the
    /// returned list.
    ///
    /// The default implementation of this method uses [NodeRpcClient::get_notes_by_id].
    fn get_public_note_records<'a, 'b>(
        &'a self,
        note_ids: &'b [NoteId],
        current_timestamp: Option<u64>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<InputNoteRecord>, RpcError>> + 'a>>
    where
        'b: 'a,
    {
        Box::pin(async move {
            let note_details = self.get_notes_by_id(note_ids).await?;

            let mut public_notes = vec![];
            for detail in note_details {
                if let NetworkNote::Public(note, inclusion_proof) = detail {
                    let state = UnverifiedNoteState {
                        metadata: *note.metadata(),
                        inclusion_proof,
                    }
                    .into();
                    let note = InputNoteRecord::new(note.into(), current_timestamp, state);

                    public_notes.push(note);
                }
            }

            Ok(public_notes)
        })
    }

    /// Fetches the public accounts that have been updated since the last known state of the
    /// accounts.
    ///
    /// The `local_accounts` parameter is a list of account headers that the client has
    /// stored locally and that it wants to check for updates. If an account is private or didn't
    /// change, it is ignored and will not be included in the returned list.
    /// The default implementation of this method uses [NodeRpcClient::get_account_details].
    fn get_updated_public_accounts<'a, 'b>(
        &'a self,
        local_accounts: &'b [&AccountHeader],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Account>, RpcError>> + 'a>>
    where
        'b: 'a,
    {
        Box::pin(async move {
            let mut public_accounts = vec![];

            for local_account in local_accounts {
                let response = self.get_account_details(local_account.id()).await?;

                if let AccountDetails::Public(account, _) = response {
                    // We should only return an account if it's newer, otherwise we ignore it
                    if account.nonce().as_int() > local_account.nonce().as_int() {
                        public_accounts.push(account);
                    }
                }
            }

            Ok(public_accounts)
        })
    }

    /// Given a block number, fetches the block header corresponding to that height from the node
    /// along with the MMR proof.
    ///
    /// The default implementation of this method uses [NodeRpcClient::get_block_header_by_number].
    fn get_block_header_with_proof<'a>(
        &'a self,
        block_num: BlockNumber,
    ) -> Pin<Box<dyn Future<Output = Result<(BlockHeader, MmrProof), RpcError>> + 'a>> {
        Box::pin(async move {
            let (header, proof) = self.get_block_header_by_number(Some(block_num), true).await?;
            Ok((header, proof.ok_or(RpcError::ExpectedDataMissing(String::from("MmrProof")))?))
        })
    }

    /// Fetches the note with the specified ID.
    ///
    /// The default implementation of this method uses [NodeRpcClient::get_notes_by_id].
    ///
    /// Errors:
    /// - [RpcError::NoteNotFound] if the note with the specified ID is not found.
    fn get_note_by_id<'a>(
        &'a self,
        note_id: NoteId,
    ) -> Pin<Box<dyn Future<Output = Result<NetworkNote, RpcError>> + 'a>> {
        Box::pin(async move {
            let notes = self.get_notes_by_id(&[note_id]).await?;
            notes.into_iter().next().ok_or(RpcError::NoteNotFound(note_id))
        })
    }
}

// RPC API ENDPOINT
// ================================================================================================
//
/// RPC methods for the Miden protocol.
#[derive(Debug)]
pub enum NodeRpcClientEndpoint {
    CheckNullifiers,
    CheckNullifiersByPrefix,
    GetAccountDetails,
    GetAccountStateDelta,
    GetAccountProofs,
    GetBlockByNumber,
    GetBlockHeaderByNumber,
    SyncState,
    SubmitProvenTx,
    SyncNotes,
}

impl fmt::Display for NodeRpcClientEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeRpcClientEndpoint::CheckNullifiers => write!(f, "check_nullifiers"),
            NodeRpcClientEndpoint::CheckNullifiersByPrefix => {
                write!(f, "check_nullifiers_by_prefix")
            },
            NodeRpcClientEndpoint::GetAccountDetails => write!(f, "get_account_details"),
            NodeRpcClientEndpoint::GetAccountStateDelta => write!(f, "get_account_state_delta"),
            NodeRpcClientEndpoint::GetAccountProofs => write!(f, "get_account_proofs"),
            NodeRpcClientEndpoint::GetBlockByNumber => write!(f, "get_block_by_number"),
            NodeRpcClientEndpoint::GetBlockHeaderByNumber => {
                write!(f, "get_block_header_by_number")
            },
            NodeRpcClientEndpoint::SyncState => write!(f, "sync_state"),
            NodeRpcClientEndpoint::SubmitProvenTx => write!(f, "submit_proven_transaction"),
            NodeRpcClientEndpoint::SyncNotes => write!(f, "sync_notes"),
        }
    }
}
