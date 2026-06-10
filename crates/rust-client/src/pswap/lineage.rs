//! Persistent record and per-round transition types for one PSWAP order.
//!
//! See module-level docs on [`crate::pswap`].

use alloc::format;
use alloc::string::ToString;

use miden_protocol::Felt;
use miden_protocol::account::AccountId;
use miden_protocol::asset::FungibleAsset;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::note::{Note, NoteId, NoteInclusionProof, NoteTag, NoteType};
use miden_standards::note::PswapNote;

use super::errors::PswapLineageError;
use crate::utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable};

// PSWAP LINEAGE STATE
// ================================================================================================

/// Lifecycle state of a PSWAP order. Discriminants are part of the
/// on-disk encoding — do not renumber.
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
    /// incompatible row encodings.
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

/// Persistent record of one PSWAP order's chain state. Immutable details
/// (creator, assets, serial number, note types) live on `original_pswap`;
/// the accessors below delegate to it.
#[derive(Debug, Clone)]
pub struct PswapLineageRecord {
    /// Source of truth for every immutable "initial" field.
    pub original_pswap: PswapNote,

    /// Current tip's note id. Equals `original_pswap.id()` at depth 0;
    /// otherwise a remainder we didn't originate.
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
    /// Stable identifier (== `original_pswap.serial[1]`) shared by every
    /// note in the chain.
    pub fn order_id(&self) -> Felt {
        self.original_pswap.order_id()
    }

    /// `order_id()` wrapped as a `BTreeMap`-compatible key.
    pub(crate) fn order_id_key(&self) -> super::types::OrderIdKey {
        super::types::OrderIdKey::from(self.order_id())
    }

    /// Account that created the order (recipient of every payback).
    pub fn creator_account_id(&self) -> AccountId {
        self.original_pswap.storage().creator_account_id()
    }

    pub fn offered_asset(&self) -> &FungibleAsset {
        self.original_pswap.offered_asset()
    }

    pub fn requested_asset(&self) -> &FungibleAsset {
        self.original_pswap.storage().requested_asset()
    }

    pub fn note_type(&self) -> NoteType {
        self.original_pswap.note_type()
    }

    /// Asset-pair tag — sync returns every remainder in this chain via it.
    pub fn asset_pair_tag(&self) -> NoteTag {
        PswapNote::create_tag(self.note_type(), self.offered_asset(), self.requested_asset())
    }
}

// PSWAP LINEAGE ROUND UPDATE
// ================================================================================================

/// One round's transition. Fill = payback + remainder (≤1 each); reclaim
/// = no outputs. Applied atomically by `Store::apply_pswap_round`.
#[derive(Debug, Clone)]
pub struct PswapLineageRoundUpdate {
    pub order_id: Felt,
    pub round_depth: u32,
    // Post-round state — all fields below describe the lineage AFTER this round.
    pub remaining_offered: FungibleAsset,
    pub remaining_requested: FungibleAsset,
    pub state: PswapLineageState,
    /// New tip; `None` for terminal rounds.
    pub tip_note_id: Option<NoteId>,
    pub at_block: BlockNumber,
    /// Header for the commit block, used by `apply_pswap_round` to insert payback/remainder as
    /// `Committed`. `None` only in store-tier fixtures.
    pub at_block_header: Option<BlockHeader>,
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

/// Builds a [`PswapLineageRecord`] from raw column data. Lives here (not
/// in the `SQLite` crate) so alternative backends can reuse it.
pub fn build_record_from_columns(
    original_pswap: PswapNote,
    current_tip_note_id: NoteId,
    current_depth: u32,
    remaining_offered: u64,
    remaining_requested: u64,
    state_byte: u8,
    created_at_block: BlockNumber,
    updated_at_block: BlockNumber,
) -> Result<PswapLineageRecord, PswapLineageError> {
    // Faucets live on `original_pswap` (chain-invariant); SQL stores only amounts.
    let offered_faucet = original_pswap.offered_asset().faucet_id();
    let requested_faucet = original_pswap.storage().requested_asset().faucet_id();
    let remaining_offered = FungibleAsset::new(offered_faucet, remaining_offered).map_err(|err| {
        PswapLineageError::InconsistentRow(format!(
            "remaining_offered = {remaining_offered} (faucet {offered_faucet}) failed FungibleAsset construction: {err}"
        ))
    })?;
    let remaining_requested = FungibleAsset::new(requested_faucet, remaining_requested).map_err(|err| {
        PswapLineageError::InconsistentRow(format!(
            "remaining_requested = {remaining_requested} (faucet {requested_faucet}) failed FungibleAsset construction: {err}"
        ))
    })?;

    Ok(PswapLineageRecord {
        original_pswap,
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

/// Encodes the same columns the `SQLite` backend persisted, in declaration
/// order: the `original_pswap` blob first, then the mutable tip state. The
/// `Note` is written via the streaming `write_into`/`read_from` (not
/// `read_from_bytes`) because more fields follow it in the stream.
impl Serializable for PswapLineageRecord {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        Note::from(self.original_pswap.clone()).write_into(target);
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
        let note = Note::read_from(source)?;
        let original_pswap = PswapNote::try_from(&note)
            .map_err(|err| DeserializationError::InvalidValue(err.to_string()))?;
        let current_tip_note_id = NoteId::read_from(source)?;
        let current_depth = u32::read_from(source)?;
        let remaining_offered = u64::read_from(source)?;
        let remaining_requested = u64::read_from(source)?;
        let state_byte = u8::read_from(source)?;
        let created_at_block = u32::read_from(source)?;
        let updated_at_block = u32::read_from(source)?;
        build_record_from_columns(
            original_pswap,
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

    /// Stable byte encoding of `PswapLineageState`. The values are
    /// persisted in the `pswap_lineages.state` SQL column; reordering
    /// would silently corrupt existing databases.
    #[test]
    fn state_byte_encoding_is_stable() {
        assert_eq!(PswapLineageState::Active.as_u8(), 0);
        assert_eq!(PswapLineageState::FullyFilled.as_u8(), 1);
        assert_eq!(PswapLineageState::Reclaimed.as_u8(), 2);
    }

    /// Round-trip every state via `try_from_u8`. Belt-and-suspenders
    /// against a future renumbering breaking the on-disk format.
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

    /// Happy path for `build_record_from_columns` at depth 0.
    #[test]
    fn build_record_from_columns_accepts_valid_depth_zero_row() {
        let (sender, creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let pswap = build_test_pswap(sender, creator, offered_faucet, 100, requested_faucet, 50);
        let initial_note_id = miden_protocol::note::Note::from(pswap.clone()).id();

        let record = build_record_from_columns(
            pswap,
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
    fn build_record_from_columns_accepts_valid_advanced_row() {
        let (sender, creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let pswap = build_test_pswap(sender, creator, offered_faucet, 100, requested_faucet, 50);
        let note = miden_protocol::note::Note::from(pswap.clone());
        let record = build_record_from_columns(
            pswap,
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

    /// Unknown state discriminant in the row bubbles up as `UnknownState`.
    #[test]
    fn build_record_from_columns_rejects_unknown_state() {
        let (sender, creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let pswap = build_test_pswap(sender, creator, offered_faucet, 100, requested_faucet, 50);
        let note = miden_protocol::note::Note::from(pswap.clone());
        match build_record_from_columns(
            pswap,
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

    /// `asset_pair_tag()` and `order_id()` accessors delegate to the
    /// stored `PswapNote` rather than persisting the values
    /// separately. Verifies the delegation is consistent (no column
    /// duplication that could drift from the blob).
    #[test]
    fn accessors_delegate_to_stored_pswap_note() {
        let (sender, creator, offered_faucet, requested_faucet) = fixed_account_ids();
        let pswap = build_test_pswap(sender, creator, offered_faucet, 100, requested_faucet, 50);

        let expected_order_id = pswap.order_id();
        let expected_tag = PswapNote::create_tag(
            pswap.note_type(),
            pswap.offered_asset(),
            pswap.storage().requested_asset(),
        );

        let note = miden_protocol::note::Note::from(pswap.clone());
        let remaining_offered = *pswap.offered_asset();
        let remaining_requested = *pswap.storage().requested_asset();
        let record = PswapLineageRecord {
            original_pswap: pswap,
            current_tip_note_id: note.id(),
            current_depth: 0,
            remaining_offered,
            remaining_requested,
            state: PswapLineageState::Active,
            created_at_block: BlockNumber::from(0),
            updated_at_block: BlockNumber::from(0),
        };

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
        let record = build_record_from_columns(
            pswap,
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
