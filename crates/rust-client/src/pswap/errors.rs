//! Errors specific to PSWAP chain tracking.

use alloc::string::String;

use miden_protocol::Felt;
use miden_protocol::account::AccountId;
use miden_protocol::errors::{AssetError, NoteError};

use super::lineage::PswapLineageState;
use crate::store::StoreError;

/// Failures raised by the PSWAP chain-tracking subsystem.
#[derive(Debug, thiserror::Error)]
pub enum PswapLineageError {
    /// No tracked PSWAP lineage for the given `order_id`.
    #[error("no PSWAP lineage tracked for order_id {0}")]
    NotFound(Felt),

    /// The lineage exists but is already in a terminal state.
    #[error("PSWAP lineage is not active (state = {0:?}); no further rounds expected")]
    NotActive(PswapLineageState),

    /// The lineage's creator is not a local account — reclaim requires
    /// the creator's signing authority.
    #[error(
        "PSWAP creator account {0} is not local; reclaim requires the creator's signing authority"
    )]
    CreatorNotLocal(AccountId),

    /// The current tip is missing from the store — the tracked lineage is
    /// out of sync with the stored notes.
    #[error("current tip note is missing from the local store; the tracked lineage is out of sync")]
    TipMissing,

    /// `PswapNote::payback_note` / `remainder_note` reconstruction failed.
    #[error("PSWAP note reconstruction failed: {0}")]
    Reconstruction(#[source] NoteError),

    /// `FungibleAsset::new` rejected an attachment-derived amount (the stored
    /// value exceeds the protocol's max). Indicates either a malformed
    /// attachment from the network or a corrupted stored lineage record.
    #[error("PSWAP attachment amount is out of range for FungibleAsset: {0}")]
    AssetError(#[from] AssetError),

    /// A stored lineage's `state` byte has no matching [`PswapLineageState`] variant.
    #[error("unknown PSWAP lineage state byte: {0}")]
    UnknownState(u8),

    /// A stored lineage record's fields are mutually inconsistent.
    #[error("PSWAP lineage record is internally inconsistent: {0}")]
    InconsistentRecord(String),

    /// More chain notes share an `(order_id, depth)` than the protocol's
    /// payback + remainder maximum of two. Reachable when an unrelated note
    /// carries a colliding attachment, so it is handled (the round is skipped)
    /// rather than asserted.
    #[error("PSWAP round at depth {depth} has {count} candidate notes (expected at most 2)")]
    AmbiguousRound { depth: u32, count: usize },

    /// A store call from the PSWAP layer failed.
    #[error("PSWAP store call failed: {0}")]
    Store(#[from] StoreError),
}
