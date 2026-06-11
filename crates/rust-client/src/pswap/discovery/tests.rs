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
fn initial_record(pswap: &PswapNote, offered: u64, requested: u64) -> PswapLineageRecord {
    let offered_faucet = pswap.offered_asset().faucet_id();
    let requested_faucet = pswap.storage().requested_asset().faucet_id();
    let original_note_id = Note::from(pswap.clone()).id();
    let mut record =
        PswapLineageRecord::new_depth_zero(original_note_id, pswap, BlockNumber::from(0));
    // Override the seeded remaining_* so callers can exercise reduced balances.
    record.remaining_offered =
        FungibleAsset::new(offered_faucet, offered).expect("test value fits in FungibleAsset");
    record.remaining_requested =
        FungibleAsset::new(requested_faucet, requested).expect("test value fits in FungibleAsset");
    record
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
    let consumer = AccountId::try_from(
        miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
    )
    .unwrap();
    let creator = AccountId::try_from(
        miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2,
    )
    .unwrap();

    let pswap = build_test_pswap(consumer, creator, offered_faucet, 100, requested_faucet, 50);
    let record = initial_record(&pswap, 100, 50);

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

    let update = record
        .build_round_update(
            1,
            BlockNumber::from(7),
            &[&cand_payback, &cand_remainder],
            &no_block_headers(),
            Some(&pswap),
        )
        .expect("partial fill must produce a round update");

    assert_eq!(update.round_depth, 1);
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
    let record = initial_record(&pswap, 100, 50);

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
    let update = record
        .build_round_update(
            1,
            BlockNumber::from(7),
            &[&cand_remainder, &cand_payback],
            &no_block_headers(),
            Some(&pswap),
        )
        .expect("partial fill must classify regardless of input order");

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
    let record = initial_record(&pswap, 100, 50);

    // `depth == 0` trips `payback_note`'s "depth must be >= 1" guard.
    let bad_attachment =
        PswapNoteAttachment::new(AssetAmount::new(20).unwrap(), pswap.order_id(), 0);
    let dummy_note = Note::from(pswap.clone());
    let cand = chain_update_from(&dummy_note, bad_attachment, consumer, 5);

    let result = record.build_round_update(
        1,
        BlockNumber::from(5),
        &[&cand],
        &no_block_headers(),
        Some(&pswap),
    );
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
    let record = initial_record(&pswap, 30, 50);

    let fill_amount = 50; // exhausts requested side
    let payback_att =
        PswapNoteAttachment::new(AssetAmount::new(fill_amount).unwrap(), pswap.order_id(), 1);
    let payback = pswap.payback_note(consumer, &payback_att).unwrap();
    let cand = chain_update_from(&payback, payback_att, consumer, 9);

    let update = record
        .build_round_update(1, BlockNumber::from(9), &[&cand], &no_block_headers(), Some(&pswap))
        .expect("full fill must produce a round update");

    assert_eq!(update.state, PswapLineageState::FullyFilled);
    assert_eq!(update.remaining_offered.amount(), AssetAmount::ZERO);
    assert_eq!(update.remaining_requested.amount(), AssetAmount::ZERO);
    assert_eq!(update.tip_note_id, None);
    assert!(update.remainder.is_none());
}

/// 0-candidate consumption → `Reclaimed`, both `remaining_*` zeroed.
/// Regression guard.
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
    let record = initial_record(&pswap, 80, 40);

    let update = record
        .build_round_update(1, BlockNumber::from(5), &[], &no_block_headers(), None)
        .expect("zero-output consumption must produce a round update");

    assert_eq!(update.state, PswapLineageState::Reclaimed);
    assert_eq!(update.remaining_offered.amount(), AssetAmount::ZERO);
    // Regression: reclaim used to leak the pre-reclaim
    // `remaining_requested` into the terminal record.
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
    let record0 = initial_record(&pswap, 100, 50);

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

    let update1 = record0
        .build_round_update(
            1,
            BlockNumber::from(11),
            &[&payback_cand, &remainder_cand],
            &no_block_headers(),
            Some(&pswap),
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

    let update2 = record1
        .build_round_update(
            2,
            BlockNumber::from(11),
            &[&cand_p2],
            &no_block_headers(),
            Some(&pswap),
        )
        .unwrap();

    assert_eq!(update2.round_depth, 2);
    assert_eq!(update2.state, PswapLineageState::FullyFilled);
    assert_eq!(update2.remaining_offered.amount(), AssetAmount::ZERO);
    assert_eq!(update2.remaining_requested.amount(), AssetAmount::ZERO);

    let record2 = record1.apply_round_in_memory(&update2);
    assert_eq!(record2.state, PswapLineageState::FullyFilled);
    let emitted = [update1, update2];
    assert_eq!(emitted.len(), 2);
}
