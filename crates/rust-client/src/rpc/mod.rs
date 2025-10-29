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
//! The client implementation adapts to the target environment automatically:
//! - Native targets use `tonic` transport with TLS.
//! - `wasm32` targets use `tonic-web-wasm-client` transport.
//!
//! ## Example
//!
//! ```no_run
//! # use miden_client::rpc::{Endpoint, NodeRpcClient, GrpcClient};
//! # use miden_objects::block::BlockNumber;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a gRPC client instance (assumes default endpoint configuration).
//! let endpoint = Endpoint::new("https".into(), "localhost".into(), Some(57291));
//! let mut rpc_client = GrpcClient::new(&endpoint, 1000);
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

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use domain::account::{AccountProofs, FetchedAccount};
use domain::note::{FetchedNote, NoteSyncInfo};
use domain::nullifier::NullifierUpdate;
use domain::sync::StateSyncInfo;
use miden_objects::Word;
use miden_objects::account::{Account, AccountCode, AccountHeader, AccountId};
use miden_objects::address::NetworkId;
use miden_objects::block::{BlockHeader, BlockNumber, ProvenBlock};
use miden_objects::crypto::merkle::{MmrProof, SmtProof};
use miden_objects::note::{NoteId, NoteScript, NoteTag, Nullifier};
use miden_objects::transaction::{ProvenTransaction, TransactionInputs};

/// Contains domain types related to RPC requests and responses, as well as utility functions
/// for dealing with them.
pub mod domain;

mod errors;
pub use errors::*;

mod endpoint;
pub use endpoint::Endpoint;

#[cfg(not(feature = "testing"))]
mod generated;
#[cfg(feature = "testing")]
pub mod generated;

#[cfg(feature = "tonic")]
mod tonic_client;
#[cfg(feature = "tonic")]
pub use tonic_client::GrpcClient;

use crate::rpc::domain::account_vault::AccountVaultInfo;
use crate::rpc::domain::storage_map::StorageMapInfo;
use crate::rpc::domain::transaction::TransactionsInfo;
use crate::store::InputNoteRecord;
use crate::store::input_note_states::UnverifiedNoteState;
use crate::transaction::ForeignAccount;

// RPC ENDPOINT LIMITS
// ================================================================================================

pub const ACCOUNT_ID_LIMIT: usize = 500;
pub const NOTE_TAG_LIMIT: usize = 500;

// NODE RPC CLIENT TRAIT
// ================================================================================================

/// Defines the interface for communicating with the Miden node.
///
/// The implementers are responsible for connecting to the Miden node, handling endpoint
/// requests/responses, and translating responses into domain objects relevant for each of the
/// endpoints.
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait NodeRpcClient: Send + Sync {
    /// Sets the genesis commitment for the client and reconnects to the node providing the
    /// genesis commitment in the request headers. If the genesis commitment is already set,
    /// this method does nothing.
    async fn set_genesis_commitment(&self, commitment: Word) -> Result<(), RpcError>;

    /// Given a Proven Transaction, send it to the node for it to be included in a future block
    /// using the `/SubmitProvenTransaction` RPC endpoint.
    async fn submit_proven_transaction(
        &self,
        proven_transaction: ProvenTransaction,
        transaction_inputs: TransactionInputs,
    ) -> Result<BlockNumber, RpcError>;

    /// Given a block number, fetches the block header corresponding to that height from the node
    /// using the `/GetBlockHeaderByNumber` endpoint.
    /// If `include_mmr_proof` is set to true and the function returns an `Ok`, the second value
    /// of the return tuple should always be Some(MmrProof).
    ///
    /// When `None` is provided, returns info regarding the latest block.
    async fn get_block_header_by_number(
        &self,
        block_num: Option<BlockNumber>,
        include_mmr_proof: bool,
    ) -> Result<(BlockHeader, Option<MmrProof>), RpcError>;

    /// Given a block number, fetches the block corresponding to that height from the node using
    /// the `/GetBlockByNumber` RPC endpoint.
    async fn get_block_by_number(&self, block_num: BlockNumber) -> Result<ProvenBlock, RpcError>;

    /// Fetches note-related data for a list of [`NoteId`] using the `/GetNotesById` rpc endpoint.
    ///
    /// For any [`miden_objects::note::NoteType::Private`] note, the return data is only the
    /// [`miden_objects::note::NoteMetadata`], whereas for [`miden_objects::note::NoteType::Public`]
    /// notes, the return data includes all details.
    async fn get_notes_by_id(&self, note_ids: &[NoteId]) -> Result<Vec<FetchedNote>, RpcError>;

    /// Fetches info from the node necessary to perform a state sync using the
    /// `/SyncState` RPC endpoint.
    ///
    /// - `block_num` is the last block number known by the client. The returned [`StateSyncInfo`]
    ///   should contain data starting from the next block, until the first block which contains a
    ///   note of matching the requested tag, or the chain tip if there are no notes.
    /// - `account_ids` is a list of account IDs and determines the accounts the client is
    ///   interested in and should receive account updates of.
    /// - `note_tags` is a list of tags used to filter the notes the client is interested in, which
    ///   serves as a "note group" filter. Notice that you can't filter by a specific note ID.
    /// - `nullifiers_tags` similar to `note_tags`, is a list of tags used to filter the nullifiers
    ///   corresponding to some notes the client is interested in.
    async fn sync_state(
        &self,
        block_num: BlockNumber,
        account_ids: &[AccountId],
        note_tags: &[NoteTag],
    ) -> Result<StateSyncInfo, RpcError>;

    /// Fetches the current state of an account from the node using the `/GetAccountDetails` RPC
    /// endpoint.
    ///
    /// - `account_id` is the ID of the wanted account.
    async fn get_account_details(&self, account_id: AccountId) -> Result<FetchedAccount, RpcError>;

    /// Fetches the notes related to the specified tags using the `/SyncNotes` RPC endpoint.
    ///
    /// - `block_num` is the last block number known by the client.
    /// - `note_tags` is a list of tags used to filter the notes the client is interested in.
    async fn sync_notes(
        &self,
        block_num: BlockNumber,
        block_to: Option<BlockNumber>,
        note_tags: &BTreeSet<NoteTag>,
    ) -> Result<NoteSyncInfo, RpcError>;

    /// Fetches the nullifiers corresponding to a list of prefixes using the
    /// `/SyncNullifiers` RPC endpoint.
    ///
    /// - `prefix` is a list of nullifiers prefixes to search for.
    /// - `block_num` is the block number to start the search from. Nullifiers created in this block
    ///   or the following blocks will be included.
    /// - `block_to` is the optional block number to stop the search at. If not provided, syncs up
    ///   to the network chain tip.
    async fn sync_nullifiers(
        &self,
        prefix: &[u16],
        block_num: BlockNumber,
        block_to: Option<BlockNumber>,
    ) -> Result<Vec<NullifierUpdate>, RpcError>;

    /// Fetches the nullifier proofs corresponding to a list of nullifiers using the
    /// `/CheckNullifiers` RPC endpoint.
    async fn check_nullifiers(&self, nullifiers: &[Nullifier]) -> Result<Vec<SmtProof>, RpcError>;

    /// Fetches the account data needed to perform a Foreign Procedure Invocation (FPI) on the
    /// specified foreign accounts, using the `GetAccountProofs` endpoint.
    ///
    /// The `code_commitments` parameter is a list of known code commitments
    /// to prevent unnecessary data fetching. Returns the block number and the FPI account data. If
    /// one of the tracked accounts is not found in the node, the method will return an error.
    async fn get_account_proofs(
        &self,
        account_storage_requests: &BTreeSet<ForeignAccount>,
        known_account_codes: BTreeMap<AccountId, AccountCode>,
    ) -> Result<AccountProofs, RpcError>;

    /// Fetches the commit height where the nullifier was consumed. If the nullifier isn't found,
    /// then `None` is returned.
    /// The `block_num` parameter is the block number to start the search from.
    ///
    /// The default implementation of this method uses
    /// [`NodeRpcClient::sync_nullifiers`].
    async fn get_nullifier_commit_height(
        &self,
        nullifier: &Nullifier,
        block_num: BlockNumber,
    ) -> Result<Option<BlockNumber>, RpcError> {
        let nullifiers = self.sync_nullifiers(&[nullifier.prefix()], block_num, None).await?;

        Ok(nullifiers
            .iter()
            .find(|update| update.nullifier == *nullifier)
            .map(|update| update.block_num))
    }

    /// Fetches public note-related data for a list of [`NoteId`] and builds [`InputNoteRecord`]s
    /// with it. If a note is not found or it's private, it is ignored and will not be included
    /// in the returned list.
    ///
    /// The default implementation of this method uses [`NodeRpcClient::get_notes_by_id`].
    async fn get_public_note_records(
        &self,
        note_ids: &[NoteId],
        current_timestamp: Option<u64>,
    ) -> Result<Vec<InputNoteRecord>, RpcError> {
        if note_ids.is_empty() {
            return Ok(vec![]);
        }

        let mut public_notes = Vec::with_capacity(note_ids.len());
        // TODO: We need a better structured way of getting limits as defined by the node (#1139)
        for chunk in note_ids.chunks(1_000) {
            let note_details = self.get_notes_by_id(chunk).await?;

            for detail in note_details {
                if let FetchedNote::Public(note, inclusion_proof) = detail {
                    let state = UnverifiedNoteState {
                        metadata: *note.metadata(),
                        inclusion_proof,
                    }
                    .into();
                    let note = InputNoteRecord::new(note.into(), current_timestamp, state);

                    public_notes.push(note);
                }
            }
        }

        Ok(public_notes)
    }

    /// Fetches the public accounts that have been updated since the last known state of the
    /// accounts.
    ///
    /// The `local_accounts` parameter is a list of account headers that the client has
    /// stored locally and that it wants to check for updates. If an account is private or didn't
    /// change, it is ignored and will not be included in the returned list.
    /// The default implementation of this method uses [`NodeRpcClient::get_account_details`].
    async fn get_updated_public_accounts(
        &self,
        local_accounts: &[&AccountHeader],
    ) -> Result<Vec<Account>, RpcError> {
        let mut public_accounts = vec![];

        for local_account in local_accounts {
            let response = self.get_account_details(local_account.id()).await?;

            if let FetchedAccount::Public(account, _) = response {
                let account = *account;
                // We should only return an account if it's newer, otherwise we ignore it
                if account.nonce().as_int() > local_account.nonce().as_int() {
                    public_accounts.push(account);
                }
            }
        }

        Ok(public_accounts)
    }

    /// Given a block number, fetches the block header corresponding to that height from the node
    /// along with the MMR proof.
    ///
    /// The default implementation of this method uses
    /// [`NodeRpcClient::get_block_header_by_number`].
    async fn get_block_header_with_proof(
        &self,
        block_num: BlockNumber,
    ) -> Result<(BlockHeader, MmrProof), RpcError> {
        let (header, proof) = self.get_block_header_by_number(Some(block_num), true).await?;
        Ok((header, proof.ok_or(RpcError::ExpectedDataMissing(String::from("MmrProof")))?))
    }

    /// Fetches the note with the specified ID.
    ///
    /// The default implementation of this method uses [`NodeRpcClient::get_notes_by_id`].
    ///
    /// Errors:
    /// - [`RpcError::NoteNotFound`] if the note with the specified ID is not found.
    async fn get_note_by_id(&self, note_id: NoteId) -> Result<FetchedNote, RpcError> {
        let notes = self.get_notes_by_id(&[note_id]).await?;
        notes.into_iter().next().ok_or(RpcError::NoteNotFound(note_id))
    }

    /// Fetches the note script with the specified root.
    ///
    /// Errors:
    /// - [`RpcError::ExpectedDataMissing`] if the note with the specified root is not found.
    async fn get_note_script_by_root(&self, root: Word) -> Result<NoteScript, RpcError>;

    /// Fetches storage map updates for specified account and storage slots within a block range,
    /// using the `/SyncStorageMaps` RPC endpoint.
    ///
    /// - `block_from`: The starting block number for the range.
    /// - `block_to`: The ending block number for the range.
    /// - `account_id`: The account ID for which to fetch storage map updates.
    async fn sync_storage_maps(
        &self,
        block_from: BlockNumber,
        block_to: Option<BlockNumber>,
        account_id: AccountId,
    ) -> Result<StorageMapInfo, RpcError>;

    /// Fetches account vault updates for specified account within a block range,
    /// using the `/SyncAccountVault` RPC endpoint.
    ///
    /// - `block_from`: The starting block number for the range.
    /// - `block_to`: The ending block number for the range.
    /// - `account_id`: The account ID for which to fetch storage map updates.
    async fn sync_account_vault(
        &self,
        block_from: BlockNumber,
        block_to: Option<BlockNumber>,
        account_id: AccountId,
    ) -> Result<AccountVaultInfo, RpcError>;

    /// Fetches transactions records for specific accounts within a block range.
    /// Using the `/SyncTransactions` RPC endpoint.
    ///
    /// - `block_from`: The starting block number for the range.
    /// - `block_to`: The ending block number for the range.
    /// - `account_ids`: The account IDs for which to fetch storage map updates.
    async fn sync_transactions(
        &self,
        block_from: BlockNumber,
        block_to: Option<BlockNumber>,
        account_ids: Vec<AccountId>,
    ) -> Result<TransactionsInfo, RpcError>;

    /// Fetches the network ID of the node.
    /// Errors:
    /// - [`RpcError::ExpectedDataMissing`] if the note with the specified root is not found.
    async fn get_network_id(&self) -> Result<NetworkId, RpcError>;
}

// RPC API ENDPOINT
// ================================================================================================
//
/// RPC methods for the Miden protocol.
#[derive(Debug)]
pub enum NodeRpcClientEndpoint {
    CheckNullifiers,
    SyncNullifiers,
    GetAccountDetails,
    GetAccountStateDelta,
    GetAccountProofs,
    GetBlockByNumber,
    GetBlockHeaderByNumber,
    GetNotesById,
    SyncState,
    SubmitProvenTx,
    SyncNotes,
    GetNoteScriptByRoot,
    SyncStorageMaps,
    SyncAccountVault,
    SyncTransactions,
}

impl fmt::Display for NodeRpcClientEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeRpcClientEndpoint::CheckNullifiers => write!(f, "check_nullifiers"),
            NodeRpcClientEndpoint::SyncNullifiers => {
                write!(f, "sync_nullifiers")
            },
            NodeRpcClientEndpoint::GetAccountDetails => write!(f, "get_account_details"),
            NodeRpcClientEndpoint::GetAccountStateDelta => write!(f, "get_account_state_delta"),
            NodeRpcClientEndpoint::GetAccountProofs => write!(f, "get_account_proofs"),
            NodeRpcClientEndpoint::GetBlockByNumber => write!(f, "get_block_by_number"),
            NodeRpcClientEndpoint::GetBlockHeaderByNumber => {
                write!(f, "get_block_header_by_number")
            },
            NodeRpcClientEndpoint::GetNotesById => write!(f, "get_notes_by_id"),
            NodeRpcClientEndpoint::SyncState => write!(f, "sync_state"),
            NodeRpcClientEndpoint::SubmitProvenTx => write!(f, "submit_proven_transaction"),
            NodeRpcClientEndpoint::SyncNotes => write!(f, "sync_notes"),
            NodeRpcClientEndpoint::GetNoteScriptByRoot => write!(f, "get_note_script_by_root"),
            NodeRpcClientEndpoint::SyncStorageMaps => write!(f, "sync_storage_maps"),
            NodeRpcClientEndpoint::SyncAccountVault => write!(f, "sync_account_vault"),
            NodeRpcClientEndpoint::SyncTransactions => write!(f, "sync_transactions"),
        }
    }
}
