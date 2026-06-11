//! PSWAP chain tracking — follows partial-swap orders across fills so the
//! creator can always see the current tip and reclaim the unfilled balance.
//!
//! Flow:
//! 1. Create → persist a [`PswapLineageRecord`] + asset-pair tag subscription.
//! 2. Sync → [`PswapChainObserver`] collects PSWAP-attachment notes;
//!    `discovery::discover_pswap_rounds` correlates them with tracked-note consumption events and
//!    emits one `PswapLineageRoundUpdate` per round.
//! 3. Reclaim → [`Client::build_pswap_cancel_by_order`].
//!
//! Protocol invariants (≤1 payback + ≤1 remainder per round, attachment
//! word layout, deterministic reconstruction) live on
//! `miden_standards::note::PswapNote`.

pub mod discovery;
pub mod errors;
pub mod lineage;
pub mod observer;
pub(crate) mod store;

// `PswapTransactionObserver` is defined inline below in this file.
use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use alloc::vec::Vec;

use async_trait::async_trait;
pub use errors::PswapLineageError;
pub use lineage::{PswapLineageFilter, PswapLineageRecord, PswapLineageState};
use miden_protocol::Felt;
use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::Note;
use miden_standards::note::PswapNote;
use miden_tx::auth::TransactionAuthenticator;
pub use observer::PswapChainObserver;

use crate::store::{NoteFilter, Store};
use crate::sync::{NoteTagRecord, NoteTagSource};
use crate::transaction::{
    TransactionObserver,
    TransactionRequest,
    TransactionRequestBuilder,
    TransactionResult,
    notes_from_output,
};
use crate::{Client, ClientError};

// PSWAP TRANSACTION OBSERVER
// ================================================================================================

/// Registers a [`PswapLineageRecord`] + asset-pair tag subscription for
/// every depth-0 PSWAP this wallet emits. Creator-agnostic (service
/// wallets are tracked too; reclaim surfaces `CreatorNotLocal` later).
pub struct PswapTransactionObserver {
    store: Arc<dyn Store>,
}

impl PswapTransactionObserver {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }
}

#[async_trait(?Send)]
impl TransactionObserver for PswapTransactionObserver {
    fn name(&self) -> &'static str {
        "PswapTransactionObserver"
    }

    async fn apply(
        &self,
        tx_result: &TransactionResult,
        submission_height: BlockNumber,
    ) -> Result<(), ClientError> {
        let output_notes = tx_result.executed_transaction().output_notes();

        for note in notes_from_output(output_notes) {
            let Ok(pswap) = PswapNote::try_from(note) else {
                continue;
            };

            // Remainders we emitted filling someone else's order — skip.
            if pswap.parent_depth() != 0 {
                continue;
            }

            // The full note lives in `output_notes`; the record keeps only its id
            // plus the immutable order facts (see `PswapLineageRecord`).
            let record = PswapLineageRecord::new_depth_zero(note.id(), &pswap, submission_height);

            store::put_lineage(&self.store, &record).await?;
            self.store
                .add_note_tag(NoteTagRecord {
                    // `note_type` is read straight off the note we just parsed — no
                    // need to mirror it on the record.
                    tag: record.asset_pair_tag(pswap.note_type()),
                    source: NoteTagSource::Subscription(record.original_note_id),
                })
                .await?;
        }

        Ok(())
    }
}

// =============================================================================
// PUBLIC API
// =============================================================================

impl<AUTH: TransactionAuthenticator + Sync + 'static> Client<AUTH> {
    /// Returns every PSWAP lineage tracked by this client.
    pub async fn pswap_lineages(&self) -> Result<Vec<PswapLineageRecord>, ClientError> {
        store::list_lineages(&self.store, PswapLineageFilter::All)
            .await
            .map_err(Into::into)
    }

    /// Returns lineages created by a specific local account.
    pub async fn pswap_lineages_for(
        &self,
        creator: AccountId,
    ) -> Result<Vec<PswapLineageRecord>, ClientError> {
        store::list_lineages(&self.store, PswapLineageFilter::ByCreator(creator))
            .await
            .map_err(Into::into)
    }

    /// Returns the lineage for one order, or `None` if not tracked.
    pub async fn pswap_lineage(
        &self,
        order_id: Felt,
    ) -> Result<Option<PswapLineageRecord>, ClientError> {
        store::get_lineage(&self.store, order_id).await.map_err(Into::into)
    }

    /// Builds a tx reclaiming the unfilled offered asset on the current
    /// tip of an Active lineage. See [`PswapLineageError`] for failure modes.
    pub async fn build_pswap_cancel_by_order(
        &self,
        order_id: Felt,
    ) -> Result<TransactionRequest, ClientError> {
        let lineage = store::get_lineage(&self.store, order_id)
            .await?
            .ok_or(PswapLineageError::NotFound(order_id))?;

        if lineage.state != PswapLineageState::Active {
            return Err(PswapLineageError::NotActive(lineage.state).into());
        }

        // Fail loud now — opaque signing failure later is worse.
        let creator = lineage.creator_account_id();
        let local_accounts: BTreeSet<_> = self.store.get_account_ids().await?.into_iter().collect();
        if !local_accounts.contains(&creator) {
            return Err(PswapLineageError::CreatorNotLocal(creator).into());
        }

        // At depth 0 the tip is the original PSWAP, fetched from `output_notes`
        // by its id. At depth > 0 the tip is a remainder discovered during sync
        // and persisted to `input_notes`.
        let tip_note: Note = if lineage.current_depth == 0 {
            Note::from(store::get_original_pswap(&self.store, lineage.original_note_id).await?)
        } else {
            let record = self
                .store
                .get_input_notes(NoteFilter::Unique(lineage.current_tip_note_id))
                .await?
                .into_iter()
                .next()
                .ok_or(PswapLineageError::TipMissing)?;
            record.try_into().map_err(ClientError::NoteRecordConversionError)?
        };

        TransactionRequestBuilder::new()
            .build_pswap_cancel(tip_note, lineage.creator_account_id())
            .map_err(ClientError::TransactionRequestError)
    }
}
