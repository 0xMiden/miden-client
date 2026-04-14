//! Event types emitted by the client service.
//!
//! After each sync, the [`ClientService`](crate::ClientService) decomposes the
//! [`SyncSummary`] into individual [`ClientEvent`] variants and publishes them
//! on a broadcast channel. Consumers can react to these events in three ways:
//!
//! ## 1. Raw Event Stream
//!
//! ```rust,ignore
//! let mut events = service.subscribe();
//! tokio::spawn(async move {
//!     while let Ok(event) = events.recv().await {
//!         match event {
//!             ClientEvent::NoteCommitted { note_id } => println!("committed: {note_id}"),
//!             ClientEvent::TransactionCommitted { transaction_id } => println!("confirmed: {transaction_id}"),
//!             _ => {}
//!         }
//!     }
//! });
//! ```
//!
//! ## 2. Persistent Handlers
//!
//! ```rust,ignore
//! service.on(EventFilter::AnyNoteReceived, |event, service| async move {
//!     let client = service.client().await;
//!     // query client state...
//! });
//! ```
//!
//! ## 3. One-Shot Awaiters
//!
//! ```rust,ignore
//! let tx_id = service.submit_transaction(account_id, request).await?;
//! service.once(EventFilter::TransactionCommitted(tx_id), Some(Duration::from_secs(60))).await?;
//! ```

use miden_client::sync::SyncSummary;
use miden_client::transaction::TransactionId;
use miden_protocol::account::AccountId;
use miden_protocol::note::NoteId;

/// Events emitted by the [`ClientService`](crate::ClientService).
///
/// Granular events (notes, transactions, accounts) are emitted before
/// [`SyncCompleted`](Self::SyncCompleted) so subscribers see individual
/// changes first.
#[derive(Debug, Clone)]
pub enum ClientEvent {
    /// A new public note was received during sync.
    NoteReceived { note_id: NoteId },
    /// A tracked note was committed on-chain.
    NoteCommitted { note_id: NoteId },
    /// A note was consumed.
    NoteConsumed { note_id: NoteId },
    /// A transaction was committed on-chain.
    TransactionCommitted { transaction_id: TransactionId },
    /// An on-chain account was updated.
    AccountUpdated { account_id: AccountId },
    /// A private account was locked.
    AccountLocked { account_id: AccountId },
    /// Sync completed. Always emitted last in a sync cycle.
    SyncCompleted { summary: SyncSummary },
}

impl ClientEvent {
    /// Returns the [`NoteId`] if this is a note-related event.
    pub fn note_id(&self) -> Option<NoteId> {
        match self {
            Self::NoteReceived { note_id }
            | Self::NoteCommitted { note_id }
            | Self::NoteConsumed { note_id } => Some(*note_id),
            _ => None,
        }
    }

    /// Returns the [`TransactionId`] if this is a transaction-related event.
    pub fn transaction_id(&self) -> Option<TransactionId> {
        match self {
            Self::TransactionCommitted { transaction_id } => Some(*transaction_id),
            _ => None,
        }
    }

    /// Returns the [`AccountId`] if this is an account-related event.
    pub fn account_id(&self) -> Option<AccountId> {
        match self {
            Self::AccountUpdated { account_id } | Self::AccountLocked { account_id } => {
                Some(*account_id)
            },
            _ => None,
        }
    }

    /// Returns the [`SyncSummary`] if this is a [`SyncCompleted`](Self::SyncCompleted) event.
    pub fn summary(&self) -> Option<&SyncSummary> {
        match self {
            Self::SyncCompleted { summary } => Some(summary),
            _ => None,
        }
    }
}

/// Decomposes a [`SyncSummary`] into individual [`ClientEvent`] variants.
///
/// Granular events are emitted first, with [`ClientEvent::SyncCompleted`] last.
pub(crate) fn events_from_sync(summary: &SyncSummary) -> Vec<ClientEvent> {
    let capacity = summary.new_public_notes.len()
        + summary.committed_notes.len()
        + summary.consumed_notes.len()
        + summary.committed_transactions.len()
        + summary.updated_accounts.len()
        + summary.locked_accounts.len()
        + 1;
    let mut events = Vec::with_capacity(capacity);

    for &note_id in &summary.new_public_notes {
        events.push(ClientEvent::NoteReceived { note_id });
    }
    for &note_id in &summary.committed_notes {
        events.push(ClientEvent::NoteCommitted { note_id });
    }
    for &note_id in &summary.consumed_notes {
        events.push(ClientEvent::NoteConsumed { note_id });
    }
    for &transaction_id in &summary.committed_transactions {
        events.push(ClientEvent::TransactionCommitted { transaction_id });
    }
    for &account_id in &summary.updated_accounts {
        events.push(ClientEvent::AccountUpdated { account_id });
    }
    for &account_id in &summary.locked_accounts {
        events.push(ClientEvent::AccountLocked { account_id });
    }

    events.push(ClientEvent::SyncCompleted { summary: summary.clone() });

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{empty_summary, test_note_id};

    #[test]
    fn empty_summary_produces_only_sync_completed() {
        let events = events_from_sync(&empty_summary());
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ClientEvent::SyncCompleted { .. }));
    }

    #[test]
    fn sync_completed_is_last() {
        let mut summary = empty_summary();
        let note_id = test_note_id();
        summary.new_public_notes = vec![note_id];
        summary.committed_notes = vec![note_id];

        let events = events_from_sync(&summary);
        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], ClientEvent::NoteReceived { .. }));
        assert!(matches!(events[1], ClientEvent::NoteCommitted { .. }));
        assert!(matches!(events[2], ClientEvent::SyncCompleted { .. }));
    }
}
