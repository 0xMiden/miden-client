//! Post-sync correlator: joins tracked-note consumption events from
//! `NoteUpdateTracker::consumed_note_ids()` with the PSWAP-attachment
//! notes collected by [`super::observer::PswapChainObserver`], emitting
//! one `PswapLineageRoundUpdate` per round transition.
//!
//! See [`crate::pswap`] for the overall design.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_protocol::Felt;
use miden_protocol::asset::FungibleAsset;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::note::NoteId;
use miden_standards::note::PswapNote;
use tracing::error;

use super::errors::PswapLineageError;
use super::lineage::{PswapLineageRecord, PswapLineageRoundUpdate, PswapLineageState};
use super::observer::PswapChainNoteUpdate;
use super::store;
use crate::store::Store;
use crate::sync::StateSyncUpdate;

/// Returns one [`PswapLineageRoundUpdate`] per round advanced this sync.
///
/// Each active lineage is walked in memory across as many rounds as this sync
/// window reveals. Only the final tip's remainder is persisted to `input_notes`;
/// intermediate remainders are already spent on-chain.
pub(crate) async fn discover_pswap_rounds(
    store: Arc<dyn Store>,
    state_sync_update: &StateSyncUpdate,
    chain_note_updates: &[PswapChainNoteUpdate],
) -> Result<Vec<PswapLineageRoundUpdate>, PswapLineageError> {
    let consumed_note_ids: BTreeSet<NoteId> =
        state_sync_update.note_updates.consumed_note_ids().collect();

    if consumed_note_ids.is_empty() && chain_note_updates.is_empty() {
        return Ok(Vec::new());
    }

    let candidate_orders =
        collect_candidate_orders(&store, &consumed_note_ids, chain_note_updates).await?;
    let active_lineages = load_active_lineages(&store, candidate_orders).await?;
    if active_lineages.is_empty() {
        return Ok(Vec::new());
    }

    let notes_by_order_depth = group_notes_by_order_depth(chain_note_updates);

    // All rounds discovered this sync share the sync's terminal block.
    let sync_block = state_sync_update.block_num;
    let block_headers: BTreeMap<BlockNumber, BlockHeader> = state_sync_update
        .partial_blockchain_updates
        .block_headers()
        .map(|(header, _)| (header.block_num(), header.clone()))
        .collect();

    let mut round_updates: Vec<PswapLineageRoundUpdate> = Vec::new();
    for lineage in active_lineages {
        let lineage_rounds = advance_lineage(
            &store,
            lineage,
            &consumed_note_ids,
            &notes_by_order_depth,
            sync_block,
            &block_headers,
        )
        .await;
        round_updates.extend(lineage_rounds);
    }

    Ok(round_updates)
}

/// Candidate orders from a union of two signals, each resolving to an `order_id`
/// without scanning:
///   1. a consumed note id that is a tracked tip → via the tip index;
///   2. a chain note → carries its `order_id` on its attachment.
///
/// Both are needed: signal 2 catches a fill whose notes arrive before its tip
/// nullifier; signal 1 carries reclaim, which emits no chain notes.
async fn collect_candidate_orders(
    store: &Arc<dyn Store>,
    consumed_note_ids: &BTreeSet<NoteId>,
    chain_note_updates: &[PswapChainNoteUpdate],
) -> Result<BTreeSet<Felt>, PswapLineageError> {
    let mut candidate_orders: BTreeSet<Felt> = BTreeSet::new();
    for note_id in consumed_note_ids {
        if let Some(order_id) = store::resolve_order_by_tip(store, *note_id).await? {
            candidate_orders.insert(order_id);
        }
    }
    for note in chain_note_updates {
        candidate_orders.insert(note.attachment.order_id());
    }
    Ok(candidate_orders)
}

/// Loads the `Active` lineage record for each candidate order, skipping orders
/// with no tracked record or already in a terminal state.
async fn load_active_lineages(
    store: &Arc<dyn Store>,
    candidate_orders: BTreeSet<Felt>,
) -> Result<Vec<PswapLineageRecord>, PswapLineageError> {
    let mut active_lineages = Vec::new();
    for order_id in candidate_orders {
        if let Some(record) = store::get_lineage(store, order_id).await?
            && record.state == PswapLineageState::Active
        {
            active_lineages.push(record);
        }
    }
    Ok(active_lineages)
}

/// Groups observed chain notes by `(order_id, depth)` for O(1) per-round lookup.
fn group_notes_by_order_depth(
    chain_note_updates: &[PswapChainNoteUpdate],
) -> BTreeMap<(Felt, u32), Vec<&PswapChainNoteUpdate>> {
    let mut notes_by_order_depth: BTreeMap<(Felt, u32), Vec<&PswapChainNoteUpdate>> =
        BTreeMap::new();
    for note in chain_note_updates {
        notes_by_order_depth
            .entry((note.attachment.order_id(), note.attachment.depth()))
            .or_default()
            .push(note);
    }
    notes_by_order_depth
}

/// Walks one active lineage across every round this sync window reveals,
/// returning its round updates (final-tip remainder kept, intermediates dropped).
///
/// Advances round-by-round while live. A round fires when the tip's consumption
/// was observed (`tip_consumed`) OR depth+1 chain notes exist: by protocol
/// invariant a payback/remainder at depth N+1 can only come from consuming the
/// depth-N tip, so notes alone prove consumption. That's what follows a
/// same-block multi-fill on a private chain, whose intermediate remainder is
/// never tracked. The state guard ends the loop on terminal.
async fn advance_lineage(
    store: &Arc<dyn Store>,
    mut lineage: PswapLineageRecord,
    consumed_note_ids: &BTreeSet<NoteId>,
    notes_by_order_depth: &BTreeMap<(Felt, u32), Vec<&PswapChainNoteUpdate>>,
    sync_block: BlockNumber,
    block_headers: &BTreeMap<BlockNumber, BlockHeader>,
) -> Vec<PswapLineageRoundUpdate> {
    let mut lineage_rounds: Vec<PswapLineageRoundUpdate> = Vec::new();
    // The depth-0 note is immutable across rounds and only fills (not reclaim)
    // need it to reconstruct outputs. Fetched lazily from `output_notes` on the
    // first fill and cached for the rest of this lineage's rounds.
    let mut original_pswap: Option<PswapNote> = None;

    while lineage.state == PswapLineageState::Active {
        let round_depth = lineage.current_depth + 1;
        let notes = notes_by_order_depth
            .get(&(lineage.order_id(), round_depth))
            .map_or(&[][..], Vec::as_slice);

        let tip_consumed = consumed_note_ids.contains(&lineage.current_tip_note_id);
        if !tip_consumed && notes.is_empty() {
            break;
        }

        // Fills (notes present) reconstruct payback/remainder from the original note;
        // fetch it once. A reclaim round (no notes) needs nothing from the note.
        if !notes.is_empty() && original_pswap.is_none() {
            match store::get_original_pswap(store, lineage.original_note_id).await {
                Ok(pswap) => original_pswap = Some(pswap),
                Err(err) => {
                    error!(
                        order_id = ?lineage.order_id(),
                        original_note_id = ?lineage.original_note_id,
                        error = ?err,
                        "discover_pswap_rounds: original note unavailable; skipping lineage",
                    );
                    break;
                },
            }
        }

        let update = match lineage.build_round_update(
            round_depth,
            sync_block,
            notes,
            block_headers,
            original_pswap.as_ref(),
        ) {
            Ok(u) => u,
            Err(err) => {
                error!(
                    order_id = ?lineage.order_id(),
                    round_depth,
                    error = ?err,
                    "discover_pswap_rounds: round build failed; skipping lineage",
                );
                break;
            },
        };

        lineage = lineage.apply_round_in_memory(&update);
        lineage_rounds.push(update);
    }

    // Intermediate remainders are already spent on-chain; inserting them would leave stale
    // Unverified notes whose consumption falls outside the next sync's window. Keep only the
    // final (live) tip's remainder; drop the rest. Paybacks are all kept — each is a distinct
    // consumable note for the creator.
    if let Some((_, intermediate_rounds)) = lineage_rounds.split_last_mut() {
        for round in intermediate_rounds {
            round.remainder = None;
        }
    }
    lineage_rounds
}

// Per-round classification and in-memory advance. These hang off `PswapLineageRecord` because each
// transition is a function of the lineage's current state plus the round's observed chain notes.
impl PswapLineageRecord {
    /// Builds one round's [`PswapLineageRoundUpdate`] from the chain notes observed at this depth.
    fn build_round_update(
        &self,
        round_depth: u32,
        block_number: BlockNumber,
        notes: &[&PswapChainNoteUpdate],
        block_headers: &BTreeMap<BlockNumber, BlockHeader>,
        original_pswap: Option<&PswapNote>,
    ) -> Result<PswapLineageRoundUpdate, PswapLineageError> {
        match notes.len() {
            0 => Ok(self.build_reclaim_round(round_depth, block_number)),
            1 => {
                let pswap = self.require_original(original_pswap)?;
                self.build_full_fill_round(
                    round_depth,
                    block_number,
                    notes[0],
                    block_headers,
                    pswap,
                )
            },
            2 => {
                let pswap = self.require_original(original_pswap)?;
                self.build_partial_fill_round(
                    round_depth,
                    block_number,
                    notes,
                    block_headers,
                    pswap,
                )
            },
            count => Err(PswapLineageError::AmbiguousRound { depth: round_depth, count }),
        }
    }

    /// The caller fetches the original note before any fill round, so a missing
    /// note here is a broken invariant rather than an expected branch.
    fn require_original<'a>(
        &self,
        original_pswap: Option<&'a PswapNote>,
    ) -> Result<&'a PswapNote, PswapLineageError> {
        original_pswap.ok_or(PswapLineageError::OriginalNoteUnavailable(self.original_note_id))
    }

    /// Zero-amount asset for the order's offered faucet (taken from the
    /// chain-invariant faucet on `remaining_offered`).
    fn zero_offered(&self) -> FungibleAsset {
        FungibleAsset::new(self.remaining_offered.faucet_id(), 0).expect("FA(_, 0) is always valid")
    }

    /// Zero-amount asset for the order's requested faucet (taken from the
    /// chain-invariant faucet on `remaining_requested`).
    fn zero_requested(&self) -> FungibleAsset {
        FungibleAsset::new(self.remaining_requested.faucet_id(), 0)
            .expect("FA(_, 0) is always valid")
    }

    /// Reclaim — cancel branch emits no outputs; only the creator can hit it.
    fn build_reclaim_round(
        &self,
        round_depth: u32,
        block_number: BlockNumber,
    ) -> PswapLineageRoundUpdate {
        PswapLineageRoundUpdate {
            order_id: self.order_id(),
            round_depth,
            remaining_offered: self.zero_offered(),
            remaining_requested: self.zero_requested(),
            state: PswapLineageState::Reclaimed,
            tip_note_id: None,
            at_block: block_number,
            at_block_note_root: None,
            payback: None,
            remainder: None,
        }
    }

    /// Full fill — only payback emitted; `remaining_requested` → 0.
    fn build_full_fill_round(
        &self,
        round_depth: u32,
        block_number: BlockNumber,
        payback_note_update: &PswapChainNoteUpdate,
        block_headers: &BTreeMap<BlockNumber, BlockHeader>,
        pswap: &PswapNote,
    ) -> Result<PswapLineageRoundUpdate, PswapLineageError> {
        let payback = pswap
            .payback_note(payback_note_update.sender, &payback_note_update.attachment)
            .map_err(PswapLineageError::Reconstruction)?;

        Ok(PswapLineageRoundUpdate {
            order_id: self.order_id(),
            round_depth,
            remaining_offered: self.zero_offered(),
            remaining_requested: self.zero_requested(),
            state: PswapLineageState::FullyFilled,
            tip_note_id: None,
            at_block: block_number,
            at_block_note_root: block_headers
                .get(&payback_note_update.block_num)
                .map(BlockHeader::note_root),
            payback: Some((payback, payback_note_update.inclusion_proof.clone())),
            remainder: None,
        })
    }

    /// Partial fill — payback + remainder. Distinguishes the two by tag.
    fn build_partial_fill_round(
        &self,
        round_depth: u32,
        block_number: BlockNumber,
        notes: &[&PswapChainNoteUpdate],
        block_headers: &BTreeMap<BlockNumber, BlockHeader>,
        pswap: &PswapNote,
    ) -> Result<PswapLineageRoundUpdate, PswapLineageError> {
        let offered_faucet = pswap.offered_asset().faucet_id();
        let requested_faucet = pswap.storage().requested_asset().faucet_id();

        let payback_tag = pswap.storage().payback_note_tag();
        let (payback_note_update, remainder_note_update) = if notes[0].tag == payback_tag {
            (notes[0], notes[1])
        } else {
            (notes[1], notes[0])
        };

        let payback_note = pswap
            .payback_note(payback_note_update.sender, &payback_note_update.attachment)
            .map_err(PswapLineageError::Reconstruction)?;

        let fill_amount = FungibleAsset::new(
            requested_faucet,
            u64::from(payback_note_update.attachment.amount()),
        )
        .map_err(PswapLineageError::AssetError)?;
        let payout_amount = FungibleAsset::new(
            offered_faucet,
            u64::from(remainder_note_update.attachment.amount()),
        )
        .map_err(PswapLineageError::AssetError)?;

        // Saturating sub — clamp to zero on over-fill.
        let remaining_requested = self
            .remaining_requested
            .sub(fill_amount)
            .unwrap_or_else(|_| self.zero_requested());
        let remaining_offered = self
            .remaining_offered
            .sub(payout_amount)
            .unwrap_or_else(|_| self.zero_offered());

        let remainder_note = pswap
            .remainder_note(
                remainder_note_update.sender,
                &remainder_note_update.attachment,
                remaining_offered.amount(),
                remaining_requested.amount(),
            )
            .map_err(PswapLineageError::Reconstruction)?;
        Ok(PswapLineageRoundUpdate {
            order_id: self.order_id(),
            round_depth,
            remaining_offered,
            remaining_requested,
            state: PswapLineageState::Active,
            tip_note_id: Some(remainder_note_update.note_id),
            at_block: block_number,
            at_block_note_root: block_headers
                .get(&payback_note_update.block_num)
                .map(BlockHeader::note_root),
            payback: Some((payback_note, payback_note_update.inclusion_proof.clone())),
            remainder: Some((remainder_note, remainder_note_update.inclusion_proof.clone())),
        })
    }

    /// Returns the post-round version. Drives the same-block multi-fill loop.
    fn apply_round_in_memory(mut self, update: &PswapLineageRoundUpdate) -> PswapLineageRecord {
        self.current_depth = update.round_depth;
        self.remaining_offered = update.remaining_offered;
        self.remaining_requested = update.remaining_requested;
        self.state = update.state;
        self.updated_at_block = update.at_block;
        if let Some(note_id) = update.tip_note_id {
            self.current_tip_note_id = note_id;
        }
        self
    }
}

#[cfg(test)]
mod tests;
