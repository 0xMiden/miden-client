//! Post-sync correlator: joins tracked-note consumption events from
//! `NoteUpdateTracker::consumed_note_ids()` with the PSWAP-attachment
//! notes collected by [`super::observer::PswapChainObserver`], emitting
//! one [`super::lineage::PswapLineageRoundUpdate`] per round transition.
//!
//! See [`crate::pswap`] for the overall design.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_protocol::asset::FungibleAsset;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::note::NoteId;
use tracing::error;

use super::errors::PswapLineageError;
use super::lineage::{
    PswapLineageFilter,
    PswapLineageRecord,
    PswapLineageRoundUpdate,
    PswapLineageState,
};
use super::observer::PswapChainNoteUpdate;
use super::types::OrderIdKey;
use crate::store::Store;
use crate::sync::StateSyncUpdate;

// ================================================================================================
// PUBLIC ENTRY POINT
// ================================================================================================

/// Returns one [`PswapLineageRoundUpdate`] per round advanced this sync.
///
/// Each active lineage is walked in memory across as many rounds as this sync window reveals.
/// A round fires when either the current tip's consumption was observed (`consumed_note_ids`)
/// or depth+1 chain notes exist for the order; the latter alone is a sound consumption proof,
/// which is what carries a same-block multi-fill on a private chain (where the intermediate
/// remainder is never tracked). Only the final tip's remainder is persisted into `input_notes`;
/// intermediate remainders are dropped since they are already spent on-chain.
pub async fn discover_pswap_rounds(
    store: Arc<dyn Store>,
    state_sync_update: &StateSyncUpdate,
    chain_note_updates: &[PswapChainNoteUpdate],
) -> Result<Vec<PswapLineageRoundUpdate>, PswapLineageError> {
    let consumed_note_ids: BTreeSet<NoteId> =
        state_sync_update.note_updates.consumed_note_ids().collect();

    if consumed_note_ids.is_empty() && chain_note_updates.is_empty() {
        return Ok(Vec::new());
    }

    // Load only lineages whose tip is in this sync window.
    let active_lineages = store
        .list_pswap_lineages(PswapLineageFilter::ActiveByTipNoteIds(
            consumed_note_ids.iter().copied().collect(),
        ))
        .await?;
    if active_lineages.is_empty() {
        return Ok(Vec::new());
    }

    // Group notes by (order_id, depth) for O(1) per-round lookup.
    let mut notes_by_order_depth: BTreeMap<(OrderIdKey, u32), Vec<&PswapChainNoteUpdate>> =
        BTreeMap::new();
    for note in chain_note_updates {
        notes_by_order_depth
            .entry((OrderIdKey::from(note.attachment.order_id()), note.attachment.depth()))
            .or_default()
            .push(note);
    }

    // All rounds discovered this sync share the sync's terminal block.
    let sync_block = state_sync_update.block_num;
    let block_headers: BTreeMap<BlockNumber, BlockHeader> = state_sync_update
        .partial_blockchain_updates
        .block_headers()
        .map(|(header, _)| (header.block_num(), header.clone()))
        .collect();
    let mut round_updates: Vec<PswapLineageRoundUpdate> = Vec::new();

    for lineage_record in active_lineages {
        let mut lineage = lineage_record;
        let mut lineage_rounds: Vec<PswapLineageRoundUpdate> = Vec::new();

        // Advance round-by-round while the lineage is still live. A round fires when EITHER
        // the current tip's consumption was observed this sync (`tip_consumed`) OR depth+1
        // chain notes exist for this order. The latter is a sound consumption proof on its
        // own: by protocol invariant a payback/remainder at depth N+1 can only be emitted by
        // consuming the depth-N tip. This is what lets us follow a same-block multi-fill on a
        // PRIVATE chain, where the intermediate remainder is never tracked and so never enters
        // `consumed_note_ids`. The state guard terminates the loop on FullyFilled / Reclaimed.
        while lineage.state == PswapLineageState::Active {
            let round_depth = lineage.current_depth + 1;
            let notes = notes_by_order_depth
                .get(&(lineage.order_id_key(), round_depth))
                .map_or(&[][..], Vec::as_slice);

            let tip_consumed = consumed_note_ids.contains(&lineage.current_tip_note_id);
            if !tip_consumed && notes.is_empty() {
                break;
            }

            let update = match build_round_update(
                &lineage,
                round_depth,
                sync_block,
                notes,
                &block_headers,
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

        // Every intermediate remainder was already consumed on-chain by the following round, so
        // inserting it into `input_notes` would leave a stale Unverified note whose consumption
        // falls outside the next sync's nullifier window. Only the final tip is live, so keep
        // its remainder insert and drop the intermediate ones. Paybacks are all kept — each is a
        // distinct consumable note for the creator.
        if let Some((_, intermediate_rounds)) = lineage_rounds.split_last_mut() {
            for round in intermediate_rounds {
                round.remainder = None;
            }
        }
        round_updates.extend(lineage_rounds);
    }

    Ok(round_updates)
}

// ================================================================================================
// PER-ROUND CLASSIFICATION
// ================================================================================================

/// Builds one round's [`PswapLineageRoundUpdate`].
fn build_round_update(
    lineage: &PswapLineageRecord,
    round_depth: u32,
    block_number: BlockNumber,
    notes: &[&PswapChainNoteUpdate],
    block_headers: &BTreeMap<BlockNumber, BlockHeader>,
) -> Result<PswapLineageRoundUpdate, PswapLineageError> {
    match notes.len() {
        0 => Ok(build_reclaim_round(lineage, round_depth, block_number)),
        1 => build_full_fill_round(lineage, round_depth, block_number, notes[0], block_headers),
        2 => build_partial_fill_round(lineage, round_depth, block_number, notes, block_headers),
        _ => unreachable!("PSWAP emits ≤ 2 notes per (order_id, depth)"),
    }
}

/// Zero-amount asset for the order's offered faucet.
fn zero_offered(lineage: &PswapLineageRecord) -> FungibleAsset {
    FungibleAsset::new(lineage.original_pswap.offered_asset().faucet_id(), 0)
        .expect("FA(_, 0) is always valid")
}

/// Zero-amount asset for the order's requested faucet.
fn zero_requested(lineage: &PswapLineageRecord) -> FungibleAsset {
    FungibleAsset::new(lineage.original_pswap.storage().requested_asset().faucet_id(), 0)
        .expect("FA(_, 0) is always valid")
}

/// Reclaim — cancel branch emits no outputs; only the creator can hit it.
fn build_reclaim_round(
    lineage: &PswapLineageRecord,
    round_depth: u32,
    block_number: BlockNumber,
) -> PswapLineageRoundUpdate {
    PswapLineageRoundUpdate {
        order_id: lineage.order_id(),
        round_depth,
        consumer_account_id: lineage.creator_account_id(),
        fill_amount: zero_requested(lineage),
        payout_amount: lineage.remaining_offered,
        remaining_offered: zero_offered(lineage),
        remaining_requested: zero_requested(lineage),
        state: PswapLineageState::Reclaimed,
        tip_note_id: None,
        at_block: block_number,
        at_block_header: None,
        payback: None,
        remainder: None,
    }
}

/// Full fill — only payback emitted; `remaining_requested` → 0.
fn build_full_fill_round(
    lineage: &PswapLineageRecord,
    round_depth: u32,
    block_number: BlockNumber,
    payback_note_update: &PswapChainNoteUpdate,
    block_headers: &BTreeMap<BlockNumber, BlockHeader>,
) -> Result<PswapLineageRoundUpdate, PswapLineageError> {
    let pswap = &lineage.original_pswap;
    let requested_faucet = pswap.storage().requested_asset().faucet_id();
    let payback = pswap
        .payback_note(payback_note_update.sender, &payback_note_update.attachment)
        .map_err(PswapLineageError::Reconstruction)?;
    let fill_amount =
        FungibleAsset::new(requested_faucet, u64::from(payback_note_update.attachment.amount()))
            .map_err(PswapLineageError::AssetError)?;

    Ok(PswapLineageRoundUpdate {
        order_id: lineage.order_id(),
        round_depth,
        consumer_account_id: payback_note_update.sender,
        fill_amount,
        payout_amount: lineage.remaining_offered,
        remaining_offered: zero_offered(lineage),
        remaining_requested: zero_requested(lineage),
        state: PswapLineageState::FullyFilled,
        tip_note_id: None,
        at_block: block_number,
        at_block_header: block_headers.get(&payback_note_update.block_num).cloned(),
        payback: Some((payback, payback_note_update.inclusion_proof.clone())),
        remainder: None,
    })
}

/// Partial fill — payback + remainder. Distinguishes the two by tag.
fn build_partial_fill_round(
    lineage: &PswapLineageRecord,
    round_depth: u32,
    block_number: BlockNumber,
    notes: &[&PswapChainNoteUpdate],
    block_headers: &BTreeMap<BlockNumber, BlockHeader>,
) -> Result<PswapLineageRoundUpdate, PswapLineageError> {
    let pswap = &lineage.original_pswap;
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

    let fill_amount =
        FungibleAsset::new(requested_faucet, u64::from(payback_note_update.attachment.amount()))
            .map_err(PswapLineageError::AssetError)?;
    let payout_amount =
        FungibleAsset::new(offered_faucet, u64::from(remainder_note_update.attachment.amount()))
            .map_err(PswapLineageError::AssetError)?;

    // Saturating sub — clamp to zero on over-fill.
    let remaining_requested = lineage
        .remaining_requested
        .sub(fill_amount)
        .unwrap_or_else(|_| zero_requested(lineage));
    let remaining_offered = lineage
        .remaining_offered
        .sub(payout_amount)
        .unwrap_or_else(|_| zero_offered(lineage));

    let remainder_note = pswap
        .remainder_note(
            remainder_note_update.sender,
            &remainder_note_update.attachment,
            remaining_offered.amount(),
            remaining_requested.amount(),
        )
        .map_err(PswapLineageError::Reconstruction)?;
    Ok(PswapLineageRoundUpdate {
        order_id: lineage.order_id(),
        round_depth,
        consumer_account_id: payback_note_update.sender,
        fill_amount,
        payout_amount,
        remaining_offered,
        remaining_requested,
        state: PswapLineageState::Active,
        tip_note_id: Some(remainder_note_update.note_id),
        at_block: block_number,
        at_block_header: block_headers.get(&payback_note_update.block_num).cloned(),
        payback: Some((payback_note, payback_note_update.inclusion_proof.clone())),
        remainder: Some((remainder_note, remainder_note_update.inclusion_proof.clone())),
    })
}

// ================================================================================================
// IN-MEMORY LINEAGE ADVANCE
// ================================================================================================

impl PswapLineageRecord {
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

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    //! Correlator tests — exercise `build_round_update` + multi-fill advance.
    use alloc::vec::Vec;

    use miden_protocol::account::AccountId;
    use miden_protocol::asset::AssetAmount;
    use miden_protocol::crypto::merkle::SparseMerklePath;
    use miden_protocol::note::{Note, NoteInclusionProof};
    use miden_standards::note::{PswapNote, PswapNoteAttachment};

    use super::super::lineage::test_helpers::{build_test_pswap, fixed_account_ids};
    use super::*;

    /// Minimum-valid inclusion proof — correlator never inspects the path.
    fn dummy_inclusion_proof(block: u32) -> NoteInclusionProof {
        let path =
            SparseMerklePath::from_parts(0, Vec::new()).expect("empty SparseMerklePath is valid");
        NoteInclusionProof::new(BlockNumber::from(block), 0, path)
            .expect("zero index is well below the per-block notes ceiling")
    }

    /// Empty header map — these tests don't assert on inserted-note state.
    fn no_block_headers() -> BTreeMap<BlockNumber, BlockHeader> {
        BTreeMap::new()
    }

    /// Active lineage at depth 0 built from a fresh test PSWAP.
    fn initial_record(pswap: PswapNote, offered: u64, requested: u64) -> PswapLineageRecord {
        let note = Note::from(pswap.clone());
        let offered_faucet = pswap.offered_asset().faucet_id();
        let requested_faucet = pswap.storage().requested_asset().faucet_id();
        PswapLineageRecord {
            original_pswap: pswap,
            current_tip_note_id: note.id(),
            current_depth: 0,
            remaining_offered: FungibleAsset::new(offered_faucet, offered)
                .expect("test value fits in FungibleAsset"),
            remaining_requested: FungibleAsset::new(requested_faucet, requested)
                .expect("test value fits in FungibleAsset"),
            state: PswapLineageState::Active,
            created_at_block: BlockNumber::from(0),
            updated_at_block: BlockNumber::from(0),
        }
    }

    /// `PswapChainNoteUpdate` mirroring `note` (id + tag) so the
    /// correlator's tag-based payback/remainder split works.
    fn chain_update_from(
        note: &Note,
        attachment: PswapNoteAttachment,
        sender: AccountId,
        block: u32,
    ) -> PswapChainNoteUpdate {
        PswapChainNoteUpdate {
            note_id: note.id(),
            attachment,
            sender,
            tag: note.metadata().tag(),
            block_num: BlockNumber::from(block),
            inclusion_proof: dummy_inclusion_proof(block),
        }
    }

    /// 2-candidate partial fill → `Active`, both `remaining_*` reduced.
    #[test]
    fn build_round_update_partial_fill_advances_active() {
        let (_sender, _creator, offered_faucet, requested_faucet) = fixed_account_ids();
        // Distinct consumer asserts round-trip through `consumer_account_id`.
        let consumer = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
        )
        .unwrap();
        let creator = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2,
        )
        .unwrap();

        let pswap = build_test_pswap(consumer, creator, offered_faucet, 100, requested_faucet, 50);
        let record = initial_record(pswap.clone(), 100, 50);

        let fill_amount = 20;
        let payout_amount = 40;
        let new_off = 100 - payout_amount;
        let new_req = 50 - fill_amount;

        let payback_att =
            PswapNoteAttachment::new(AssetAmount::new(fill_amount).unwrap(), pswap.order_id(), 1);
        let remainder_att =
            PswapNoteAttachment::new(AssetAmount::new(payout_amount).unwrap(), pswap.order_id(), 1);
        let payback = pswap.payback_note(consumer, &payback_att).unwrap();
        let remainder = pswap
            .remainder_note(
                consumer,
                &remainder_att,
                AssetAmount::new(new_off).unwrap(),
                AssetAmount::new(new_req).unwrap(),
            )
            .unwrap();

        let cand_payback = chain_update_from(&payback, payback_att, consumer, 7);
        let cand_remainder = chain_update_from(&remainder, remainder_att, consumer, 7);

        let update = build_round_update(
            &record,
            1,
            BlockNumber::from(7),
            &[&cand_payback, &cand_remainder],
            &no_block_headers(),
        )
        .expect("partial fill must produce a round update");

        assert_eq!(update.round_depth, 1);
        assert_eq!(update.consumer_account_id, consumer);
        assert_eq!(update.fill_amount.amount(), AssetAmount::new(fill_amount).unwrap());
        assert_eq!(update.payout_amount.amount(), AssetAmount::new(payout_amount).unwrap());
        assert_eq!(update.remaining_offered.amount(), AssetAmount::new(new_off).unwrap());
        assert_eq!(update.remaining_requested.amount(), AssetAmount::new(new_req).unwrap());
        assert_eq!(update.state, PswapLineageState::Active);
        assert_eq!(update.tip_note_id, Some(remainder.id()));
        // Each side carries its note paired with its inclusion proof.
        assert!(update.payback.is_some());
        assert!(update.remainder.is_some());
    }

    /// Note order within a round must not change classification: passing
    /// `[remainder, payback]` (the reverse of the natural ordering) yields the
    /// same result as `[payback, remainder]`. Covers the tag-split else-branch.
    #[test]
    fn build_round_update_partial_fill_classifies_regardless_of_note_order() {
        let (_sender, _creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let consumer = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
        )
        .unwrap();
        let creator = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2,
        )
        .unwrap();

        let pswap = build_test_pswap(consumer, creator, offered_faucet, 100, requested_faucet, 50);
        let record = initial_record(pswap.clone(), 100, 50);

        let fill_amount = 20;
        let payout_amount = 40;
        let new_off = 100 - payout_amount;
        let new_req = 50 - fill_amount;

        let payback_att =
            PswapNoteAttachment::new(AssetAmount::new(fill_amount).unwrap(), pswap.order_id(), 1);
        let remainder_att =
            PswapNoteAttachment::new(AssetAmount::new(payout_amount).unwrap(), pswap.order_id(), 1);
        let payback = pswap.payback_note(consumer, &payback_att).unwrap();
        let remainder = pswap
            .remainder_note(
                consumer,
                &remainder_att,
                AssetAmount::new(new_off).unwrap(),
                AssetAmount::new(new_req).unwrap(),
            )
            .unwrap();

        let cand_payback = chain_update_from(&payback, payback_att, consumer, 7);
        let cand_remainder = chain_update_from(&remainder, remainder_att, consumer, 7);

        // Reverse the input order — remainder first.
        let update = build_round_update(
            &record,
            1,
            BlockNumber::from(7),
            &[&cand_remainder, &cand_payback],
            &no_block_headers(),
        )
        .expect("partial fill must classify regardless of input order");

        assert_eq!(update.fill_amount.amount(), AssetAmount::new(fill_amount).unwrap());
        assert_eq!(update.payout_amount.amount(), AssetAmount::new(payout_amount).unwrap());
        assert_eq!(update.tip_note_id, Some(remainder.id()));
        assert_eq!(update.state, PswapLineageState::Active);
    }

    /// A malformed attachment (`depth == 0`) makes `PswapNote::payback_note`
    /// reject reconstruction; `build_round_update` surfaces the error rather
    /// than panicking. In `discover_pswap_rounds` this is caught, logged via
    /// `error!`, and the lineage is left at its previous tip.
    #[test]
    fn build_round_update_propagates_reconstruction_error() {
        let (_sender, _creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let consumer = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
        )
        .unwrap();
        let creator = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2,
        )
        .unwrap();

        let pswap = build_test_pswap(consumer, creator, offered_faucet, 100, requested_faucet, 50);
        let record = initial_record(pswap.clone(), 100, 50);

        // `depth == 0` trips `payback_note`'s "depth must be >= 1" guard.
        let bad_attachment =
            PswapNoteAttachment::new(AssetAmount::new(20).unwrap(), pswap.order_id(), 0);
        let dummy_note = Note::from(pswap);
        let cand = chain_update_from(&dummy_note, bad_attachment, consumer, 5);

        let result =
            build_round_update(&record, 1, BlockNumber::from(5), &[&cand], &no_block_headers());
        assert!(result.is_err(), "depth-0 attachment must fail reconstruction");
    }

    /// 1-candidate full fill → `FullyFilled`, no remainder, both zeros.
    #[test]
    fn build_round_update_full_fill_marks_fully_filled() {
        let (_sender, _creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let consumer = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
        )
        .unwrap();
        let creator = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2,
        )
        .unwrap();

        // Smaller initial sizes so the single fill exhausts both sides.
        let pswap = build_test_pswap(consumer, creator, offered_faucet, 30, requested_faucet, 50);
        let record = initial_record(pswap.clone(), 30, 50);

        let fill_amount = 50; // exhausts requested side
        let payback_att =
            PswapNoteAttachment::new(AssetAmount::new(fill_amount).unwrap(), pswap.order_id(), 1);
        let payback = pswap.payback_note(consumer, &payback_att).unwrap();
        let cand = chain_update_from(&payback, payback_att, consumer, 9);

        let update =
            build_round_update(&record, 1, BlockNumber::from(9), &[&cand], &no_block_headers())
                .expect("full fill must produce a round update");

        assert_eq!(update.state, PswapLineageState::FullyFilled);
        assert_eq!(update.fill_amount.amount(), AssetAmount::new(fill_amount).unwrap());
        assert_eq!(update.payout_amount.amount(), AssetAmount::new(30).unwrap()); // entire remaining_offered
        assert_eq!(update.remaining_offered.amount(), AssetAmount::ZERO);
        assert_eq!(update.remaining_requested.amount(), AssetAmount::ZERO);
        assert_eq!(update.tip_note_id, None);
        assert!(update.remainder.is_none());
    }

    /// 0-candidate consumption → `Reclaimed`, consumer == creator, both
    /// `remaining_*` zeroed. Regression guard.
    #[test]
    fn build_round_update_zero_outputs_marks_reclaimed_with_remaining_zero() {
        let (_sender, _creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let consumer = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
        )
        .unwrap();
        let creator = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2,
        )
        .unwrap();

        let pswap = build_test_pswap(consumer, creator, offered_faucet, 80, requested_faucet, 40);
        let record = initial_record(pswap, 80, 40);

        let update = build_round_update(&record, 1, BlockNumber::from(5), &[], &no_block_headers())
            .expect("zero-output consumption must produce a round update");

        assert_eq!(update.state, PswapLineageState::Reclaimed);
        assert_eq!(update.consumer_account_id, creator);
        assert_eq!(update.fill_amount.amount(), AssetAmount::ZERO);
        assert_eq!(update.payout_amount.amount(), AssetAmount::new(80).unwrap());
        assert_eq!(update.remaining_offered.amount(), AssetAmount::ZERO);
        // Regression: reclaim used to leak the pre-reclaim
        // `remaining_requested` into the terminal row.
        assert_eq!(update.remaining_requested.amount(), AssetAmount::ZERO);
        assert!(update.payback.is_none());
    }

    /// Same-block multi-fill: round 2 must build against round 1's
    /// in-memory-advanced lineage, not the original.
    #[test]
    fn apply_round_in_memory_chains_correctly_for_multi_fill() {
        let (_sender, _creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let consumer = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
        )
        .unwrap();
        let creator = AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2,
        )
        .unwrap();

        let pswap = build_test_pswap(consumer, creator, offered_faucet, 100, requested_faucet, 50);
        let record0 = initial_record(pswap.clone(), 100, 50);

        // ── Round 1: partial fill, 20 requested for 40 offered.
        let fill1 = 20;
        let payout1 = 40;
        let new_off1 = 100 - payout1;
        let new_req1 = 50 - fill1;
        let payback_att1 =
            PswapNoteAttachment::new(AssetAmount::new(fill1).unwrap(), pswap.order_id(), 1);
        let remainder_att1 =
            PswapNoteAttachment::new(AssetAmount::new(payout1).unwrap(), pswap.order_id(), 1);
        let payback1 = pswap.payback_note(consumer, &payback_att1).unwrap();
        let remainder1 = pswap
            .remainder_note(
                consumer,
                &remainder_att1,
                AssetAmount::new(new_off1).unwrap(),
                AssetAmount::new(new_req1).unwrap(),
            )
            .unwrap();
        let payback_cand = chain_update_from(&payback1, payback_att1, consumer, 11);
        let remainder_cand = chain_update_from(&remainder1, remainder_att1, consumer, 11);

        let update1 = build_round_update(
            &record0,
            1,
            BlockNumber::from(11),
            &[&payback_cand, &remainder_cand],
            &no_block_headers(),
        )
        .unwrap();

        // Mirrors `discover_pswap_rounds`'s inner loop.
        let record1 = record0.apply_round_in_memory(&update1);
        assert_eq!(record1.current_depth, 1);
        assert_eq!(record1.remaining_offered.amount(), AssetAmount::new(new_off1).unwrap());
        assert_eq!(record1.remaining_requested.amount(), AssetAmount::new(new_req1).unwrap());
        assert_eq!(record1.current_tip_note_id, remainder1.id());
        assert_eq!(record1.state, PswapLineageState::Active);

        // ── Round 2: full fill of the remainder, exhausts requested side.
        let fill2 = new_req1; // = 30
        let payback_att2 =
            PswapNoteAttachment::new(AssetAmount::new(fill2).unwrap(), pswap.order_id(), 2);
        let payback2 = pswap.payback_note(consumer, &payback_att2).unwrap();
        let cand_p2 = chain_update_from(&payback2, payback_att2, consumer, 11);

        let update2 = build_round_update(
            &record1,
            2,
            BlockNumber::from(11),
            &[&cand_p2],
            &no_block_headers(),
        )
        .unwrap();

        assert_eq!(update2.round_depth, 2);
        assert_eq!(update2.state, PswapLineageState::FullyFilled);
        assert_eq!(update2.fill_amount.amount(), AssetAmount::new(fill2).unwrap());
        assert_eq!(update2.payout_amount.amount(), AssetAmount::new(new_off1).unwrap()); // remaining_offered exhausted
        assert_eq!(update2.remaining_offered.amount(), AssetAmount::ZERO);
        assert_eq!(update2.remaining_requested.amount(), AssetAmount::ZERO);

        let record2 = record1.apply_round_in_memory(&update2);
        assert_eq!(record2.state, PswapLineageState::FullyFilled);
        let emitted = [update1, update2];
        assert_eq!(emitted.len(), 2);
    }
}
