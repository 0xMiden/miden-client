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
//!             ClientEvent::NoteReceived { note } => println!("received: {}", note.id()),
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
//! service.on(EventFilter::AnyNoteReceived, |event, _service| async move {
//!     // Handler receives the full InputNoteRecord — no store lookup needed.
//!     if let ClientEvent::NoteReceived { note } = event {
//!         println!("new note {} carrying {:?}", note.id(), note.assets());
//!     }
//! });
//! ```
//!
//! ## 3. One-Shot Awaiters
//!
//! ```rust,ignore
//! let tx_id = service.submit_transaction(account_id, request).await?;
//! service.once(EventFilter::TransactionCommitted(tx_id), Some(Duration::from_secs(60))).await?;
//! ```

use std::collections::BTreeMap;
use std::sync::Arc;

use miden_client::store::InputNoteRecord;
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
    /// A new public note was received during sync. Carries the full record so
    /// handlers can build consume requests, inspect the script, etc. without a
    /// store roundtrip.
    ///
    /// Wrapped in `Arc` so broadcasting to N subscribers is an O(N) refcount
    /// bump rather than N deep clones of the (potentially multi-KB) record.
    NoteReceived { note: Arc<InputNoteRecord> },
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
            Self::NoteReceived { note } => Some(note.id()),
            Self::NoteCommitted { note_id } | Self::NoteConsumed { note_id } => Some(*note_id),
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
/// `new_note_records` must contain the full [`InputNoteRecord`] for every entry in
/// `summary.new_public_notes`. Callers build this map by bulk-reading the store after
/// `apply_state_sync`. Entries missing from the map are silently skipped (the note will
/// still be in the store, just not in the event stream).
///
/// Granular events are emitted first, with [`ClientEvent::SyncCompleted`] last.
pub(crate) fn events_from_sync(
    summary: &SyncSummary,
    new_note_records: &BTreeMap<NoteId, Arc<InputNoteRecord>>,
) -> Vec<ClientEvent> {
    let capacity = summary.new_public_notes.len()
        + summary.committed_notes.len()
        + summary.consumed_notes.len()
        + summary.committed_transactions.len()
        + summary.updated_accounts.len()
        + summary.locked_accounts.len()
        + 1;
    let mut events = Vec::with_capacity(capacity);

    for note_id in &summary.new_public_notes {
        if let Some(note) = new_note_records.get(note_id) {
            events.push(ClientEvent::NoteReceived { note: Arc::clone(note) });
        }
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
    use crate::test_utils::{empty_summary, test_note_arc, test_note_id};

    #[test]
    fn empty_summary_produces_only_sync_completed() {
        let events = events_from_sync(&empty_summary(), &BTreeMap::new());
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ClientEvent::SyncCompleted { .. }));
    }

    #[test]
    fn sync_completed_is_last() {
        let note = test_note_arc();
        let note_id = test_note_id();
        let mut summary = empty_summary();
        summary.new_public_notes = vec![note_id];
        summary.committed_notes = vec![note_id];

        let mut records = BTreeMap::new();
        records.insert(note_id, Arc::clone(&note));

        let events = events_from_sync(&summary, &records);
        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], ClientEvent::NoteReceived { .. }));
        assert!(matches!(events[1], ClientEvent::NoteCommitted { .. }));
        assert!(matches!(events[2], ClientEvent::SyncCompleted { .. }));
    }

    #[test]
    fn new_public_notes_without_records_are_skipped() {
        let mut summary = empty_summary();
        summary.new_public_notes = vec![test_note_id()];
        let events = events_from_sync(&summary, &BTreeMap::new());
        // No record in the map → no NoteReceived emitted; SyncCompleted still fires.
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ClientEvent::SyncCompleted { .. }));
    }
}
