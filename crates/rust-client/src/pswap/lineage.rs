//! Persistent record and per-round transition types for one PSWAP order.
//!
//! See module-level docs on [`crate::pswap`].

use alloc::format;
use alloc::string::ToString;

use miden_protocol::account::AccountId;
use miden_protocol::asset::FungibleAsset;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::{Note, NoteId, NoteInclusionProof, NoteTag, NoteType};
use miden_protocol::{Felt, Word};
use miden_standards::note::PswapNote;

use super::errors::PswapLineageError;
use crate::utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable};

// PSWAP LINEAGE STATE
// ================================================================================================

/// Lifecycle state of a PSWAP order. Discriminants are part of the
/// serialized encoding — do not renumber.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PswapLineageState {
    /// Still fillable / reclaimable.
    Active = 0,
    /// Fully filled. Terminal.
    FullyFilled = 1,
    /// Reclaimed by the creator. Terminal.
    Reclaimed = 2,
}

impl PswapLineageState {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Errors on unknown discriminants — guards against forward-
    /// incompatible serialized encodings.
    pub fn try_from_u8(value: u8) -> Result<Self, PswapLineageError> {
        match value {
            0 => Ok(Self::Active),
            1 => Ok(Self::FullyFilled),
            2 => Ok(Self::Reclaimed),
            other => Err(PswapLineageError::UnknownState(other)),
        }
    }
}

// PSWAP LINEAGE RECORD
// ================================================================================================

/// Persistent record of one PSWAP order's chain state. The immutable order
/// facts (order id, creator, asset pair, note type) are mirrored here so the
/// common read paths — keying, filtering, tag derivation — stay lookup-free.
/// The full depth-0 note (script + recipient), needed only to reconstruct
/// paybacks/remainders and to reclaim at depth 0, is fetched on demand from
/// `output_notes` by `original_note_id` (see `store::get_original_pswap`).
#[derive(Debug, Clone)]
pub struct PswapLineageRecord {
    /// Fetch handle for the depth-0 PSWAP note in `output_notes`. Stable for the
    /// order's lifetime; distinct from `current_tip_note_id`, which advances
    /// each round.
    pub original_note_id: NoteId,

    // Immutable order facts, mirrored from the depth-0 note so accessors and
    // the asset-pair tag need no store lookup.
    order_id: Felt,
    creator_account_id: AccountId,
    offered_asset: FungibleAsset,
    requested_asset: FungibleAsset,
    note_type: NoteType,

    /// Current tip's note id. Equals `original_note_id` at depth 0; otherwise a
    /// remainder we didn't originate.
    pub current_tip_note_id: NoteId,
    /// 0 for the original tip; +1 per round. Matches `PswapNoteAttachment::depth()`.
    pub current_depth: u32,
    pub remaining_offered: FungibleAsset,
    pub remaining_requested: FungibleAsset,
    pub state: PswapLineageState,
    pub created_at_block: BlockNumber,
    pub updated_at_block: BlockNumber,
}

impl PswapLineageRecord {
    /// Builds the depth-0 record for a PSWAP the wallet has just emitted. Mirrors
    /// the immutable order facts off the note and seeds the mutable tip state:
    /// the tip is the original note, depth is 0, and `remaining_*` equal the
    /// initial offered/requested amounts.
    pub fn new_depth_zero(
        original_note_id: NoteId,
        pswap: &PswapNote,
        created_at_block: BlockNumber,
    ) -> Self {
        Self {
            original_note_id,
            order_id: pswap.order_id(),
            creator_account_id: pswap.storage().creator_account_id(),
            offered_asset: *pswap.offered_asset(),
            requested_asset: *pswap.storage().requested_asset(),
            note_type: pswap.note_type(),
            current_tip_note_id: original_note_id,
            current_depth: 0,
            remaining_offered: *pswap.offered_asset(),
            remaining_requested: *pswap.storage().requested_asset(),
            state: PswapLineageState::Active,
            created_at_block,
            updated_at_block: created_at_block,
        }
    }

    /// Stable identifier (== the depth-0 note's `serial[1]`) shared by every
    /// note in the chain.
    pub fn order_id(&self) -> Felt {
        self.order_id
    }

    /// Account that created the order (recipient of every payback).
    pub fn creator_account_id(&self) -> AccountId {
        self.creator_account_id
    }

    pub fn offered_asset(&self) -> &FungibleAsset {
        &self.offered_asset
    }

    pub fn requested_asset(&self) -> &FungibleAsset {
        &self.requested_asset
    }

    pub fn note_type(&self) -> NoteType {
        self.note_type
    }

    /// Asset-pair tag — sync returns every remainder in this chain via it.
    pub fn asset_pair_tag(&self) -> NoteTag {
        PswapNote::create_tag(self.note_type(), self.offered_asset(), self.requested_asset())
    }
}

// PSWAP LINEAGE ROUND UPDATE
// ================================================================================================

/// One round's transition. Fill = payback + remainder (≤1 each); reclaim
/// = no outputs. Applied atomically by `apply_round`.
#[derive(Debug, Clone)]
pub(crate) struct PswapLineageRoundUpdate {
    pub order_id: Felt,
    pub round_depth: u32,
    // Post-round state — all fields below describe the lineage AFTER this round.
    pub remaining_offered: FungibleAsset,
    pub remaining_requested: FungibleAsset,
    pub state: PswapLineageState,
    /// New tip; `None` for terminal rounds.
    pub tip_note_id: Option<NoteId>,
    pub at_block: BlockNumber,
    /// Commit block's note root, used by `apply_round` to insert payback/remainder as
    /// `Committed`. `None` on reclaim rounds (no note to insert) and in store-tier fixtures.
    pub at_block_note_root: Option<Word>,
    /// Reconstructed payback and its inclusion proof. `None` only on
    /// reclaim. The note and proof are always observed together in the
    /// same sync window, so they live or die as a pair.
    pub payback: Option<(Note, NoteInclusionProof)>,
    /// Reconstructed remainder and its inclusion proof. `None` on terminal
    /// rounds (full fill / reclaim). Paired for the same reason as `payback`.
    pub remainder: Option<(Note, NoteInclusionProof)>,
}

// PSWAP LINEAGE FILTER
// ================================================================================================

/// Client-side filter for `crate::pswap::store::list_lineages`. Applied in
/// Rust after a prefix-scan of the `settings` KV — not a store-trait concept.
#[derive(Debug, Clone)]
pub enum PswapLineageFilter {
    All,
    Active,
    ByCreator(AccountId),
}

// SERDE HELPERS
// ================================================================================================

/// Builds a [`PswapLineageRecord`] from its decoded fields. Lives here (not
/// in a store backend) so alternative backends can reuse it. The two
/// `remaining_*` amounts are paired with their faucets (taken from
/// `offered_asset`/`requested_asset`) to rebuild the `FungibleAsset`s the
/// record stores only as amounts.
#[allow(clippy::too_many_arguments)]
pub fn build_record_from_fields(
    original_note_id: NoteId,
    order_id: Felt,
    creator_account_id: AccountId,
    offered_asset: FungibleAsset,
    requested_asset: FungibleAsset,
    note_type: NoteType,
    current_tip_note_id: NoteId,
    current_depth: u32,
    remaining_offered: u64,
    remaining_requested: u64,
    state_byte: u8,
    created_at_block: BlockNumber,
    updated_at_block: BlockNumber,
) -> Result<PswapLineageRecord, PswapLineageError> {
    // Faucets are chain-invariant (carried on the asset pair); the record stores only amounts.
    let offered_faucet = offered_asset.faucet_id();
    let requested_faucet = requested_asset.faucet_id();
    let remaining_offered = FungibleAsset::new(offered_faucet, remaining_offered).map_err(|err| {
        PswapLineageError::InconsistentRecord(format!(
            "remaining_offered = {remaining_offered} (faucet {offered_faucet}) failed FungibleAsset construction: {err}"
        ))
    })?;
    let remaining_requested = FungibleAsset::new(requested_faucet, remaining_requested).map_err(|err| {
        PswapLineageError::InconsistentRecord(format!(
            "remaining_requested = {remaining_requested} (faucet {requested_faucet}) failed FungibleAsset construction: {err}"
        ))
    })?;

    Ok(PswapLineageRecord {
        original_note_id,
        order_id,
        creator_account_id,
        offered_asset,
        requested_asset,
        note_type,
        current_tip_note_id,
        current_depth,
        remaining_offered,
        remaining_requested,
        state: PswapLineageState::try_from_u8(state_byte)?,
        created_at_block,
        updated_at_block,
    })
}

// VALUE CODEC
// ================================================================================================

/// Encodes the record's fields in declaration order: the `original_note_id`
/// fetch handle and the mirrored immutable order facts, then the mutable tip
/// state. The full depth-0 note is no longer inlined — it is recovered from
/// `output_notes` via `original_note_id` when reconstruction needs it.
impl Serializable for PswapLineageRecord {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.original_note_id.write_into(target);
        self.order_id.write_into(target);
        self.creator_account_id.write_into(target);
        self.offered_asset.write_into(target);
        self.requested_asset.write_into(target);
        self.note_type.write_into(target);
        self.current_tip_note_id.write_into(target);
        self.current_depth.write_into(target);
        u64::from(self.remaining_offered.amount()).write_into(target);
        u64::from(self.remaining_requested.amount()).write_into(target);
        self.state.as_u8().write_into(target);
        self.created_at_block.as_u32().write_into(target);
        self.updated_at_block.as_u32().write_into(target);
    }
}

impl Deserializable for PswapLineageRecord {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let original_note_id = NoteId::read_from(source)?;
        let order_id = Felt::read_from(source)?;
        let creator_account_id = AccountId::read_from(source)?;
        let offered_asset = FungibleAsset::read_from(source)?;
        let requested_asset = FungibleAsset::read_from(source)?;
        let note_type = NoteType::read_from(source)?;
        let current_tip_note_id = NoteId::read_from(source)?;
        let current_depth = u32::read_from(source)?;
        let remaining_offered = u64::read_from(source)?;
        let remaining_requested = u64::read_from(source)?;
        let state_byte = u8::read_from(source)?;
        let created_at_block = u32::read_from(source)?;
        let updated_at_block = u32::read_from(source)?;
        build_record_from_fields(
            original_note_id,
            order_id,
            creator_account_id,
            offered_asset,
            requested_asset,
            note_type,
            current_tip_note_id,
            current_depth,
            remaining_offered,
            remaining_requested,
            state_byte,
            BlockNumber::from(created_at_block),
            BlockNumber::from(updated_at_block),
        )
        .map_err(|err| DeserializationError::InvalidValue(err.to_string()))
    }
}

#[cfg(test)]
pub(crate) mod test_helpers {
    //! Synthetic-PSWAP factory shared across lineage / discovery / store tests.

    use miden_protocol::Word;
    use miden_protocol::account::AccountId;
    use miden_protocol::asset::FungibleAsset;
    use miden_protocol::note::NoteType;
    use miden_protocol::testing::account_id::{
        ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
        ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_1,
        ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
        ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2,
    };
    use miden_standards::note::{PswapNote, PswapNoteStorage};

    /// Returns `(sender, creator, offered_faucet, requested_faucet)` —
    /// four distinct testing `AccountId`s chosen to satisfy PSWAP's
    /// faucet-distinctness invariant.
    pub fn fixed_account_ids() -> (AccountId, AccountId, AccountId, AccountId) {
        (
            AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE).unwrap(),
            AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2).unwrap(),
            AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET).unwrap(),
            AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_1).unwrap(),
        )
    }

    /// Builds a fully-formed [`PswapNote`] for use in tests. Defaults:
    /// public note type, 100-unit offered, 50-unit requested, serial
    /// number `[1, 2, 3, 4]`. Override via the params.
    pub fn build_test_pswap(
        sender: AccountId,
        creator: AccountId,
        offered_faucet: AccountId,
        offered_amount: u64,
        requested_faucet: AccountId,
        requested_amount: u64,
    ) -> PswapNote {
        let offered = FungibleAsset::new(offered_faucet, offered_amount).unwrap();
        let requested = FungibleAsset::new(requested_faucet, requested_amount).unwrap();
        let storage = PswapNoteStorage::builder()
            .requested_asset(requested)
            .creator_account_id(creator)
            .build();
        PswapNote::builder()
            .sender(sender)
            .storage(storage)
            .serial_number(Word::from([
                miden_protocol::Felt::new(1).unwrap(),
                miden_protocol::Felt::new(2).unwrap(),
                miden_protocol::Felt::new(3).unwrap(),
                miden_protocol::Felt::new(4).unwrap(),
            ]))
            .note_type(NoteType::Public)
            .offered_asset(offered)
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use miden_protocol::asset::AssetAmount;
    use miden_standards::note::PswapNote;

    use super::test_helpers::{build_test_pswap, fixed_account_ids};
    use super::*;

    /// Builds a record from a test `PswapNote`, mirroring the immutable scalars
    /// the observer would extract. Keeps the codec/accessor tests focused on the
    /// fields they exercise instead of the new wide constructor signature.
    #[allow(clippy::too_many_arguments)]
    fn record_from_test_pswap(
        pswap: &PswapNote,
        current_tip_note_id: NoteId,
        current_depth: u32,
        remaining_offered: u64,
        remaining_requested: u64,
        state_byte: u8,
        created_at_block: BlockNumber,
        updated_at_block: BlockNumber,
    ) -> Result<PswapLineageRecord, PswapLineageError> {
        let original_note_id = miden_protocol::note::Note::from(pswap.clone()).id();
        build_record_from_fields(
            original_note_id,
            pswap.order_id(),
            pswap.storage().creator_account_id(),
            *pswap.offered_asset(),
            *pswap.storage().requested_asset(),
            pswap.note_type(),
            current_tip_note_id,
            current_depth,
            remaining_offered,
            remaining_requested,
            state_byte,
            created_at_block,
            updated_at_block,
        )
    }

    /// Stable byte encoding of `PswapLineageState`. The values are
    /// persisted in the serialized lineage record; reordering
    /// would silently corrupt existing stores.
    #[test]
    fn state_byte_encoding_is_stable() {
        assert_eq!(PswapLineageState::Active.as_u8(), 0);
        assert_eq!(PswapLineageState::FullyFilled.as_u8(), 1);
        assert_eq!(PswapLineageState::Reclaimed.as_u8(), 2);
    }

    /// Round-trip every state via `try_from_u8`. Belt-and-suspenders
    /// against a future renumbering breaking the serialized format.
    #[test]
    fn state_try_from_u8_round_trips_known_variants() {
        for state in [
            PswapLineageState::Active,
            PswapLineageState::FullyFilled,
            PswapLineageState::Reclaimed,
        ] {
            assert_eq!(PswapLineageState::try_from_u8(state.as_u8()).unwrap(), state);
        }
    }

    /// Unknown discriminants must error — defends against a future
    /// store reading a forward-incompatible byte.
    #[test]
    fn state_try_from_u8_rejects_unknown() {
        match PswapLineageState::try_from_u8(99) {
            Err(PswapLineageError::UnknownState(99)) => {},
            other => panic!("expected UnknownState(99), got {other:?}"),
        }
    }

    /// Happy path for `build_record_from_fields` at depth 0.
    #[test]
    fn build_record_from_fields_accepts_valid_depth_zero_record() {
        let (sender, creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let pswap = build_test_pswap(sender, creator, offered_faucet, 100, requested_faucet, 50);
        let initial_note_id = miden_protocol::note::Note::from(pswap.clone()).id();

        let record = record_from_test_pswap(
            &pswap,
            initial_note_id,
            0,
            100,
            50,
            PswapLineageState::Active.as_u8(),
            BlockNumber::from(7),
            BlockNumber::from(7),
        )
        .unwrap();

        assert_eq!(record.current_depth, 0);
        assert_eq!(record.remaining_offered.amount(), AssetAmount::new(100).unwrap());
        assert_eq!(record.remaining_requested.amount(), AssetAmount::new(50).unwrap());
        assert_eq!(record.state, PswapLineageState::Active);
    }

    /// Happy path at `current_depth > 0`.
    #[test]
    fn build_record_from_fields_accepts_valid_advanced_record() {
        let (sender, creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let pswap = build_test_pswap(sender, creator, offered_faucet, 100, requested_faucet, 50);
        let note = miden_protocol::note::Note::from(pswap.clone());
        let record = record_from_test_pswap(
            &pswap,
            note.id(),
            3,
            70,
            35,
            PswapLineageState::Active.as_u8(),
            BlockNumber::from(7),
            BlockNumber::from(12),
        )
        .unwrap();

        assert_eq!(record.current_depth, 3);
        assert_eq!(record.remaining_offered.amount(), AssetAmount::new(70).unwrap());
    }

    /// Unknown state discriminant in a stored record bubbles up as `UnknownState`.
    #[test]
    fn build_record_from_fields_rejects_unknown_state() {
        let (sender, creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let pswap = build_test_pswap(sender, creator, offered_faucet, 100, requested_faucet, 50);
        let note = miden_protocol::note::Note::from(pswap.clone());
        match record_from_test_pswap(
            &pswap,
            note.id(),
            0,
            100,
            50,
            42,
            BlockNumber::from(0),
            BlockNumber::from(0),
        ) {
            Err(PswapLineageError::UnknownState(42)) => {},
            other => panic!("expected UnknownState(42), got {other:?}"),
        }
    }

    /// The mirrored scalars back the accessors with the same values the
    /// depth-0 note would yield, so `order_id()`, `asset_pair_tag()` and
    /// `creator_account_id()` stay correct without re-fetching the note.
    #[test]
    fn accessors_mirror_depth_zero_note() {
        let (sender, creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let pswap = build_test_pswap(sender, creator, offered_faucet, 100, requested_faucet, 50);

        let expected_order_id = pswap.order_id();
        let expected_tag = PswapNote::create_tag(
            pswap.note_type(),
            pswap.offered_asset(),
            pswap.storage().requested_asset(),
        );

        let note = miden_protocol::note::Note::from(pswap.clone());
        let record = record_from_test_pswap(
            &pswap,
            note.id(),
            0,
            100,
            50,
            PswapLineageState::Active.as_u8(),
            BlockNumber::from(0),
            BlockNumber::from(0),
        )
        .unwrap();

        assert_eq!(record.original_note_id, note.id());
        assert_eq!(record.order_id(), expected_order_id);
        assert_eq!(record.asset_pair_tag(), expected_tag);
        assert_eq!(record.creator_account_id(), creator);
    }

    /// `Serializable`/`Deserializable` round-trip preserves every field,
    /// including the faucets recovered (not stored) for the remaining
    /// amounts. Exercised at an advanced depth with reduced amounts to
    /// catch a faucet mix-up between offered/requested.
    #[test]
    fn value_codec_round_trips() {
        let (sender, creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let pswap = build_test_pswap(sender, creator, offered_faucet, 100, requested_faucet, 50);
        let note = miden_protocol::note::Note::from(pswap.clone());
        let record = record_from_test_pswap(
            &pswap,
            note.id(),
            3,
            70,
            35,
            PswapLineageState::Active.as_u8(),
            BlockNumber::from(7),
            BlockNumber::from(12),
        )
        .unwrap();

        let bytes = record.to_bytes();
        let decoded = PswapLineageRecord::read_from_bytes(&bytes).unwrap();

        assert_eq!(decoded.original_note_id, record.original_note_id);
        assert_eq!(decoded.creator_account_id(), record.creator_account_id());
        assert_eq!(decoded.offered_asset(), record.offered_asset());
        assert_eq!(decoded.requested_asset(), record.requested_asset());
        assert_eq!(decoded.note_type(), record.note_type());
        assert_eq!(decoded.order_id(), record.order_id());
        assert_eq!(decoded.current_tip_note_id, record.current_tip_note_id);
        assert_eq!(decoded.current_depth, record.current_depth);
        assert_eq!(decoded.remaining_offered, record.remaining_offered);
        assert_eq!(decoded.remaining_requested, record.remaining_requested);
        assert_eq!(decoded.remaining_offered.amount(), AssetAmount::new(70).unwrap());
        assert_eq!(decoded.remaining_requested.amount(), AssetAmount::new(35).unwrap());
        assert_eq!(decoded.state, record.state);
        assert_eq!(decoded.created_at_block, record.created_at_block);
        assert_eq!(decoded.updated_at_block, record.updated_at_block);
    }
}
