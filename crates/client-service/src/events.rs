//! Service events emitted during operations.

use miden_client::sync::SyncSummary;
use miden_client::transaction::DiscardCause;
use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::{NoteId, NoteMetadata, NoteTag, Nullifier};
use miden_protocol::transaction::TransactionId;

/// Events emitted by the service during sync and transaction operations.
///
/// These events allow consumers to react to state changes without polling.
#[derive(Debug, Clone)]
pub enum ServiceEvent {
    /// A new note was received and is now tracked by the client.
    NoteReceived {
        /// The ID of the received note.
        note_id: NoteId,
        /// The tag associated with the note.
        tag: NoteTag,
        /// Metadata associated with the note.
        metadata: NoteMetadata,
    },

    /// A tracked note was committed to the chain.
    NoteCommitted {
        /// The ID of the committed note.
        note_id: NoteId,
        /// The block number where the note was committed.
        block_num: BlockNumber,
        /// Optional metadata if available.
        metadata: Option<NoteMetadata>,
    },

    /// A note was consumed (spent).
    NoteConsumed {
        /// The ID of the consumed note.
        note_id: NoteId,
        /// Nullifier for the consumed note.
        nullifier: Nullifier,
        /// The block number where the note was consumed.
        block_num: BlockNumber,
        /// Optional metadata if available.
        metadata: Option<NoteMetadata>,
    },

    /// A transaction was committed to the chain.
    TransactionCommitted {
        /// The ID of the committed transaction.
        transaction_id: TransactionId,
        /// The block number where the transaction was committed.
        block_num: BlockNumber,
    },

    /// A transaction was discarded.
    TransactionDiscarded {
        /// The ID of the discarded transaction.
        transaction_id: TransactionId,
        /// The reason the transaction was discarded.
        cause: DiscardCause,
    },

    /// An account's state was updated.
    AccountUpdated {
        /// The ID of the updated account.
        account_id: AccountId,
        /// New account nonce after the update.
        new_nonce: u64,
    },

    /// An account was locked.
    AccountLocked {
        /// The ID of the locked account.
        account_id: AccountId,
    },

    /// A sync operation completed successfully.
    SyncCompleted {
        /// Full summary of the sync operation.
        summary: SyncSummary,
    },
}

impl ServiceEvent {
    /// Returns a human-readable name for this event type.
    pub fn event_type(&self) -> &'static str {
        match self {
            ServiceEvent::NoteReceived { .. } => "NoteReceived",
            ServiceEvent::NoteCommitted { .. } => "NoteCommitted",
            ServiceEvent::NoteConsumed { .. } => "NoteConsumed",
            ServiceEvent::TransactionCommitted { .. } => "TransactionCommitted",
            ServiceEvent::TransactionDiscarded { .. } => "TransactionDiscarded",
            ServiceEvent::AccountUpdated { .. } => "AccountUpdated",
            ServiceEvent::AccountLocked { .. } => "AccountLocked",
            ServiceEvent::SyncCompleted { .. } => "SyncCompleted",
        }
    }
}
