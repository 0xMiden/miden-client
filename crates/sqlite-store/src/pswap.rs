//! SQLite-backed implementation of the PSWAP lineage methods on the
//! [`miden_client::store::Store`] trait.
//!
//! See `crates/rust-client/src/pswap/` for the design and types.

use std::string::String;
use std::vec::Vec;

#[cfg(test)]
use miden_client::account::AccountId;
use miden_client::note::{BlockNumber, Note, NoteId, PswapNote};
use miden_client::pswap::lineage::build_record_from_columns;
use miden_client::pswap::{
    PswapLineageFilter,
    PswapLineageRecord,
    PswapLineageRoundUpdate,
    PswapLineageState,
};
use miden_client::store::StoreError;
use miden_client::sync::{NoteTagRecord, NoteTagSource};
use miden_client::utils::{Deserializable, DeserializationError, Serializable};
use miden_protocol::Felt;
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use super::SqliteStore;
use crate::note::upsert_input_note_tx;
use crate::sql_error::SqlResultExt;
use crate::sync::remove_note_tag_tx;

impl SqliteStore {
    // ---------------------------------------------------------------------------------------
    // PSWAP LINEAGE — public entry points
    //
    // The `Store` trait impl in `lib.rs` delegates here via
    // `interact_with_connection`.
    // ---------------------------------------------------------------------------------------

    pub(crate) fn upsert_pswap_lineage(
        conn: &mut Connection,
        record: &PswapLineageRecord,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;
        upsert_pswap_lineage_tx(&tx, record)?;
        tx.commit().into_store_error()
    }

    pub(crate) fn get_pswap_lineage(
        conn: &mut Connection,
        order_id: Felt,
    ) -> Result<Option<PswapLineageRecord>, StoreError> {
        let mut stmt = conn
            .prepare_cached(&std::format!("{SELECT_LINEAGE_COLUMNS_PREFIX} WHERE order_id = ?"))
            .into_store_error()?;

        let mut rows = stmt.query(params![order_id.to_bytes()]).into_store_error()?;
        match rows.next().into_store_error()? {
            Some(row) => Ok(Some(record_from_row(row)?)),
            None => Ok(None),
        }
    }

    pub(crate) fn list_pswap_lineages(
        conn: &mut Connection,
        filter: &PswapLineageFilter,
    ) -> Result<Vec<PswapLineageRecord>, StoreError> {
        // `ActiveByTipNoteIds` has a dynamic IN-list — separate path avoids
        // bloating the prepared-statement cache.
        if let PswapLineageFilter::ActiveByTipNoteIds(note_ids) = filter {
            return list_active_by_tip_note_ids(conn, note_ids);
        }

        // `ByCreator` is filtered in Rust because the creator lives inside
        // the serialised `original_pswap` blob, not in its own column.
        let sql_filter = sql_filter_part(filter);
        let full_sql = std::format!("{SELECT_LINEAGE_COLUMNS_PREFIX}{sql_filter}");
        let mut stmt = conn.prepare_cached(&full_sql).into_store_error()?;

        let rows: Vec<PswapLineageRecord> = match filter {
            PswapLineageFilter::All | PswapLineageFilter::ByCreator(_) => {
                collect_rows(stmt.query([]).into_store_error()?)?
            },
            PswapLineageFilter::Active => collect_rows(
                stmt.query(params![PswapLineageState::Active.as_u8()]).into_store_error()?,
            )?,
            PswapLineageFilter::ActiveByTipNoteIds(_) => unreachable!("handled above"),
        };

        Ok(rows)
    }

    pub(crate) fn apply_pswap_round(
        conn: &mut Connection,
        update: &PswapLineageRoundUpdate,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;

        // 1. Mutate the lineage row.
        update_lineage_tip_tx(&tx, update)?;

        // 2. Insert payback + remainder into `input_notes`. See `insert_pswap_round_note_tx` for
        //    the skip-if-present rationale. The remainder insert ensures round-N+1 detection works
        //    for private PSWAPs (screener can't see private content). Each note is observed
        //    together with its inclusion proof in the same sync window, so they arrive paired.
        if let Some((payback_note, inclusion_proof)) = &update.payback {
            insert_pswap_round_note_tx(&tx, payback_note, inclusion_proof)?;
        }
        if let Some((remainder_note, inclusion_proof)) = &update.remainder {
            insert_pswap_round_note_tx(&tx, remainder_note, inclusion_proof)?;
        }

        // 3. Drop the asset-pair tag on terminal states (same tx — crash between row update and tag
        //    delete would leave a terminal lineage paying sync bandwidth).
        if matches!(update.state, PswapLineageState::FullyFilled | PswapLineageState::Reclaimed) {
            remove_pswap_asset_pair_tag_tx(&tx, update.order_id)?;
        }

        tx.commit().into_store_error()
    }
}

/// Removes the `(asset_pair_tag, Subscription(original_note_id))` row.
/// Recomputes the tag and original `NoteId` from the persisted PSWAP blob
/// (neither is carried on the round update). Idempotent.
fn remove_pswap_asset_pair_tag_tx(tx: &Transaction<'_>, order_id: Felt) -> Result<(), StoreError> {
    const SQL: &str = "SELECT original_pswap FROM pswap_lineages WHERE order_id = ?";
    let blob: Option<Vec<u8>> = tx
        .prepare_cached(SQL)
        .into_store_error()?
        .query_row(params![order_id.to_bytes()], |row| row.get(0))
        .optional()
        .into_store_error()?;
    let Some(blob) = blob else {
        return Ok(());
    };

    let note = Note::read_from_bytes(&blob).map_err(StoreError::DataDeserializationError)?;
    let pswap = PswapNote::try_from(&note)
        .map_err(|err| StoreError::DataDeserializationError(deser_err(err.to_string())))?;
    let tag = PswapNote::create_tag(
        pswap.note_type(),
        pswap.offered_asset(),
        pswap.storage().requested_asset(),
    );
    let original_note_id = note.id();

    remove_note_tag_tx(
        tx,
        NoteTagRecord {
            tag,
            source: NoteTagSource::Subscription(original_note_id),
        },
    )?;
    Ok(())
}

// -------------------------------------------------------------------------------------------
// QUERY HELPERS
// -------------------------------------------------------------------------------------------

const SELECT_LINEAGE_COLUMNS_PREFIX: &str = "\
SELECT order_id, original_pswap, current_tip_note_id, \
       current_depth, remaining_offered, remaining_requested, state, \
       created_at_block, updated_at_block \
FROM pswap_lineages";

fn sql_filter_part(filter: &PswapLineageFilter) -> &'static str {
    match filter {
        PswapLineageFilter::Active => " WHERE state = ?",
        // `ActiveByTipNoteIds` builds its SQL dynamically in
        // `list_active_by_tip_note_ids` and never routes through here; it
        // shares the no-extra-clause arm only because this fn is unreachable
        // for it.
        PswapLineageFilter::All
        | PswapLineageFilter::ByCreator(_)
        | PswapLineageFilter::ActiveByTipNoteIds(_) => "",
    }
}

/// Loads `Active` lineages whose `current_tip_note_id` is in `note_ids`.
/// `SQLite`'s default param limit (32 766) dwarfs typical sync windows; we
/// don't chunk. `prepare` (not `prepare_cached`) keeps per-N SQL out of
/// the cache.
fn list_active_by_tip_note_ids(
    conn: &mut Connection,
    note_ids: &[NoteId],
) -> Result<Vec<PswapLineageRecord>, StoreError> {
    if note_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = std::iter::repeat_n("?", note_ids.len()).collect::<Vec<_>>().join(",");
    let sql = std::format!(
        "{SELECT_LINEAGE_COLUMNS_PREFIX} \
         WHERE state = {state} AND current_tip_note_id IN ({placeholders})",
        state = PswapLineageState::Active.as_u8(),
    );
    let mut stmt = conn.prepare(&sql).into_store_error()?;
    let note_id_texts: Vec<String> = note_ids.iter().map(|n| n.as_word().to_string()).collect();
    let rows = stmt
        .query(rusqlite::params_from_iter(note_id_texts.iter()))
        .into_store_error()?;
    collect_rows(rows)
}

fn collect_rows(mut rows: rusqlite::Rows<'_>) -> Result<Vec<PswapLineageRecord>, StoreError> {
    let mut out = Vec::new();
    while let Some(row) = rows.next().into_store_error()? {
        out.push(record_from_row(row)?);
    }
    Ok(out)
}

fn record_from_row(row: &Row<'_>) -> Result<PswapLineageRecord, StoreError> {
    let original_pswap_bytes: Vec<u8> = row.get(1).into_store_error()?;
    let current_tip_text: String = row.get(2).into_store_error()?;
    let current_depth: u32 = row.get(3).into_store_error()?;
    let remaining_offered: u64 = row.get(4).into_store_error()?;
    let remaining_requested: u64 = row.get(5).into_store_error()?;
    let state_byte: u8 = row.get(6).into_store_error()?;
    let created_at_block: u32 = row.get(7).into_store_error()?;
    let updated_at_block: u32 = row.get(8).into_store_error()?;

    // `PswapNote` does not impl Serializable directly; persist as `Note`
    // and round-trip via the existing conversion.
    let note = Note::read_from_bytes(&original_pswap_bytes)
        .map_err(StoreError::DataDeserializationError)?;
    let original_pswap = PswapNote::try_from(&note)
        .map_err(|err| StoreError::DataDeserializationError(deser_err(err.to_string())))?;

    let current_tip_note_id = NoteId::try_from_hex(&current_tip_text)
        .map_err(|err| StoreError::DataDeserializationError(deser_err(err.to_string())))?;

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
    .map_err(|err| StoreError::DatabaseError(std::format!("pswap_lineage: {err}")))
}

// -------------------------------------------------------------------------------------------
// WRITE HELPERS
// -------------------------------------------------------------------------------------------

fn upsert_pswap_lineage_tx(
    tx: &Transaction<'_>,
    record: &PswapLineageRecord,
) -> Result<(), StoreError> {
    const SQL: &str = "\
INSERT OR REPLACE INTO pswap_lineages \
(order_id, original_pswap, current_tip_note_id, \
 current_depth, remaining_offered, remaining_requested, state, \
 created_at_block, updated_at_block) \
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";

    let original_pswap_bytes = Note::from(record.original_pswap.clone()).to_bytes();

    tx.prepare_cached(SQL)
        .into_store_error()?
        .execute(params![
            record.order_id().to_bytes(),
            original_pswap_bytes,
            record.current_tip_note_id.as_word().to_string(),
            record.current_depth,
            u64::from(record.remaining_offered.amount()),
            u64::from(record.remaining_requested.amount()),
            record.state.as_u8(),
            record.created_at_block.as_u32(),
            record.updated_at_block.as_u32(),
        ])
        .into_store_error()?;

    Ok(())
}

fn update_lineage_tip_tx(
    tx: &Transaction<'_>,
    update: &PswapLineageRoundUpdate,
) -> Result<(), StoreError> {
    // Confirm the row exists AND enforce the monotonic-depth invariant
    // (every round must advance by exactly 1). The store is the last line
    // of defense against correlator off-by-ones / duplicate deliveries.
    const DEPTH_SQL: &str = "SELECT current_depth FROM pswap_lineages WHERE order_id = ?";

    let order_id_bytes = update.order_id.to_bytes();
    let current_depth: Option<u32> = tx
        .prepare_cached(DEPTH_SQL)
        .into_store_error()?
        .query_row(params![order_id_bytes], |row| row.get(0))
        .optional()
        .into_store_error()?;
    let current_depth = current_depth.ok_or_else(|| {
        StoreError::DatabaseError(std::format!(
            "apply_pswap_round: no lineage row for order_id {}",
            update.order_id
        ))
    })?;
    if update.round_depth != current_depth + 1 {
        return Err(StoreError::DatabaseError(std::format!(
            "apply_pswap_round: round_depth {} for order_id {} does not advance by 1 \
             (current_depth {}); refusing to corrupt the reconstruction chain",
            update.round_depth,
            update.order_id,
            current_depth,
        )));
    }

    let updated_block = update.at_block.as_u32();

    let rows_changed = if let Some(note_id) = update.tip_note_id {
        // Active continuation — new tip overwrites the previous one.
        const SQL: &str = "\
UPDATE pswap_lineages SET \
 current_tip_note_id = ?, \
 current_depth = ?, remaining_offered = ?, remaining_requested = ?, \
 state = ?, updated_at_block = ? \
WHERE order_id = ?";
        tx.prepare_cached(SQL)
            .into_store_error()?
            .execute(params![
                note_id.as_word().to_string(),
                update.round_depth,
                u64::from(update.remaining_offered.amount()),
                u64::from(update.remaining_requested.amount()),
                update.state.as_u8(),
                updated_block,
                order_id_bytes,
            ])
            .into_store_error()?
    } else {
        // Terminal — no new tip note, so `current_tip_note_id` stays frozen at the
        // last live tip; `current_depth` advances to the round that terminated it.
        const SQL: &str = "\
UPDATE pswap_lineages SET \
 current_depth = ?, remaining_offered = ?, remaining_requested = ?, \
 state = ?, updated_at_block = ? \
WHERE order_id = ?";
        tx.prepare_cached(SQL)
            .into_store_error()?
            .execute(params![
                update.round_depth,
                u64::from(update.remaining_offered.amount()),
                u64::from(update.remaining_requested.amount()),
                update.state.as_u8(),
                updated_block,
                order_id_bytes,
            ])
            .into_store_error()?
    };

    if rows_changed == 0 {
        return Err(StoreError::DatabaseError(std::format!(
            "apply_pswap_round: zero rows updated for order_id {}",
            update.order_id
        )));
    }

    Ok(())
}

/// Inserts a reconstructed payback or remainder into `input_notes`. Skips
/// if a row for the same `note_id` already exists — for public notes the
/// screener has already inserted a `Committed` row that's richer than
/// ours; for private notes this is the only insertion site.
///
/// The inclusion proof is always available: a reconstructed note and its
/// proof are observed together in the same sync window. The note lands as
/// `Unverified`, which sync's state-promotion path turns into `Committed`
/// on the next run.
fn insert_pswap_round_note_tx(
    tx: &Transaction<'_>,
    note: &Note,
    inclusion_proof: &miden_client::note::NoteInclusionProof,
) -> Result<(), StoreError> {
    use miden_client::store::InputNoteRecord;
    use miden_client::store::input_note_states::UnverifiedNoteState;

    const EXISTS_SQL: &str = "SELECT 1 FROM input_notes WHERE note_id = ?";

    let note_id_text = note.id().as_word().to_string();
    let already_present: bool = tx
        .prepare_cached(EXISTS_SQL)
        .into_store_error()?
        .exists(params![note_id_text])
        .into_store_error()?;
    if already_present {
        return Ok(());
    }

    let metadata = *note.metadata();
    let details = miden_client::note::NoteDetails::from(note.clone());
    let attachments = note.attachments().clone();

    let record = InputNoteRecord::new(
        details,
        attachments,
        None,
        UnverifiedNoteState {
            metadata,
            inclusion_proof: inclusion_proof.clone(),
        }
        .into(),
    );
    upsert_input_note_tx(tx, &record)
}

// -------------------------------------------------------------------------------------------
// ERROR MAPPING
// -------------------------------------------------------------------------------------------

fn deser_err(msg: String) -> DeserializationError {
    DeserializationError::InvalidValue(msg)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use miden_client::asset::FungibleAsset;
    use miden_client::pswap::PswapLineageRoundUpdate;
    use miden_client::store::Store;
    use miden_protocol::Word;
    use miden_protocol::note::{Note, NoteType};
    use miden_protocol::testing::account_id::{
        ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
        ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_1,
        ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
        ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2,
    };
    use miden_standards::note::{PswapNote, PswapNoteStorage};

    use super::*;
    use crate::tests::create_test_store;

    /// Standalone copy of `pswap::lineage::test_helpers` —
    /// `pub(crate)` doesn't cross crates.
    fn build_test_pswap(offered_amount: u64, requested_amount: u64) -> PswapNote {
        let sender = AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE).unwrap();
        let creator =
            AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2).unwrap();
        let offered_faucet = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET).unwrap();
        let requested_faucet = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_1).unwrap();

        let storage = PswapNoteStorage::builder()
            .requested_asset(FungibleAsset::new(requested_faucet, requested_amount).unwrap())
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
            .offered_asset(FungibleAsset::new(offered_faucet, offered_amount).unwrap())
            .build()
            .unwrap()
    }

    fn build_initial_record(pswap: PswapNote) -> PswapLineageRecord {
        let note = Note::from(pswap.clone());
        let remaining_offered = *pswap.offered_asset();
        let remaining_requested = *pswap.storage().requested_asset();
        PswapLineageRecord {
            original_pswap: pswap,
            current_tip_note_id: note.id(),
            current_depth: 0,
            remaining_offered,
            remaining_requested,
            state: PswapLineageState::Active,
            created_at_block: BlockNumber::from(7),
            updated_at_block: BlockNumber::from(7),
        }
    }

    /// Round-trips a record through `SQLite` — catches drift in the
    /// `Note::to_bytes` ↔ `PswapNote::try_from(&note)` serde path.
    #[tokio::test]
    async fn lineage_round_trip_via_sqlite_store() -> anyhow::Result<()> {
        let store = create_test_store().await;
        let pswap = build_test_pswap(100, 50);
        let record = build_initial_record(pswap.clone());
        let order_id = record.order_id();

        store.upsert_pswap_lineage(&record).await?;

        let fetched = store
            .get_pswap_lineage(order_id)
            .await?
            .expect("just upserted, should be present");

        // Compare via the canonical accessors. We do not derive PartialEq
        // on PswapLineageRecord because PswapNote does not implement it
        // reliably across serialisation boundaries.
        assert_eq!(fetched.order_id(), record.order_id());
        assert_eq!(fetched.current_tip_note_id, record.current_tip_note_id);
        assert_eq!(fetched.current_depth, record.current_depth);
        assert_eq!(fetched.remaining_offered, record.remaining_offered);
        assert_eq!(fetched.remaining_requested, record.remaining_requested);
        assert_eq!(fetched.state, record.state);
        assert_eq!(fetched.creator_account_id(), record.creator_account_id());
        assert_eq!(fetched.offered_asset().amount(), record.offered_asset().amount());
        Ok(())
    }

    /// `round_depth` must equal `current_depth + 1` (monotonic-depth invariant).
    #[tokio::test]
    async fn apply_pswap_round_rejects_non_monotonic_depth() -> anyhow::Result<()> {
        let store = create_test_store().await;
        let pswap = build_test_pswap(100, 50);
        let record = build_initial_record(pswap.clone());
        let order_id = record.order_id();
        store.upsert_pswap_lineage(&record).await?;

        let consumer =
            AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE).unwrap();

        // `current_depth` is 0; only `round_depth == 1` should be accepted.
        // round_depth = 3 is a non-monotonic advance.
        let offered_faucet = pswap.offered_asset().faucet_id();
        let requested_faucet = pswap.storage().requested_asset().faucet_id();
        let fa = |faucet, amount: u64| {
            miden_protocol::asset::FungibleAsset::new(faucet, amount).unwrap()
        };
        let bad = PswapLineageRoundUpdate {
            order_id,
            round_depth: 3,
            consumer_account_id: consumer,
            fill_amount: fa(requested_faucet, 10),
            payout_amount: fa(offered_faucet, 20),
            remaining_offered: fa(offered_faucet, 80),
            remaining_requested: fa(requested_faucet, 40),
            state: PswapLineageState::Active,
            tip_note_id: Some(record.current_tip_note_id),
            at_block: BlockNumber::from(8),
            payback: None,
            remainder: None,
        };
        let result = store.apply_pswap_round(&bad).await;
        assert!(result.is_err(), "expected non-monotonic depth to be rejected");

        // And the lineage row must be untouched. This catches the
        // failure mode where the depth check fires but the UPDATE has
        // already partially applied (e.g. wrong transaction scope).
        let after = store.get_pswap_lineage(order_id).await?.expect("row still present");
        assert_eq!(after.current_depth, 0, "depth must not have advanced");
        assert_eq!(after.remaining_offered, record.remaining_offered);
        assert_eq!(after.remaining_requested, record.remaining_requested);
        assert_eq!(after.state, PswapLineageState::Active);
        Ok(())
    }

    /// Unknown `order_id` must error (catches correlator bugs).
    #[tokio::test]
    async fn apply_pswap_round_rejects_unknown_order_id() -> anyhow::Result<()> {
        let store = create_test_store().await;
        let consumer =
            AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE).unwrap();
        let phantom_order_id = miden_protocol::Felt::new(0xdead_beef).unwrap();

        // Use arbitrary fungible-asset faucets — the unknown-order check
        // fires before the values are inspected.
        let faucet = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET).unwrap();
        let fa = |amount: u64| miden_protocol::asset::FungibleAsset::new(faucet, amount).unwrap();
        let bogus = PswapLineageRoundUpdate {
            order_id: phantom_order_id,
            round_depth: 1,
            consumer_account_id: consumer,
            fill_amount: fa(10),
            payout_amount: fa(20),
            remaining_offered: fa(80),
            remaining_requested: fa(40),
            state: PswapLineageState::Active,
            tip_note_id: None,
            at_block: BlockNumber::from(8),
            payback: None,
            remainder: None,
        };
        let result = store.apply_pswap_round(&bogus).await;
        assert!(result.is_err(), "expected unknown order_id to be rejected");
        Ok(())
    }

    /// `Active` and `All` both return the single tracked row. Multi-row
    /// state filtering needs distinct `order_id`s (distinct serial numbers),
    /// which the shared `build_test_pswap` fixture doesn't vary; terminal-
    /// state exclusion is covered by the observer-pipeline tests below.
    #[tokio::test]
    async fn list_pswap_lineages_filters_by_state() -> anyhow::Result<()> {
        let store = create_test_store().await;

        let active_rec = build_initial_record(build_test_pswap(100, 50));
        store.upsert_pswap_lineage(&active_rec).await?;

        let active = store.list_pswap_lineages(PswapLineageFilter::Active).await?;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].state, PswapLineageState::Active);

        let all = store.list_pswap_lineages(PswapLineageFilter::All).await?;
        assert_eq!(all.len(), 1, "All filter returns every row");
        Ok(())
    }

    /// `ActiveByTipNoteIds` returns matching Active lineages only;
    /// well-behaved for empty + non-matching inputs.
    #[tokio::test]
    async fn list_pswap_lineages_filters_by_tip_note_ids() -> anyhow::Result<()> {
        let store = create_test_store().await;

        // Insert one Active lineage; capture its tip note id.
        let rec = build_initial_record(build_test_pswap(100, 50));
        let real_tip = rec.current_tip_note_id;
        store.upsert_pswap_lineage(&rec).await?;

        // Build a second PSWAP we DON'T insert — its tip serves as a
        // "not in store" sentinel.
        let phantom_tip = build_initial_record(build_test_pswap(999, 999)).current_tip_note_id;

        // Empty input: no rows.
        let empty = store
            .list_pswap_lineages(PswapLineageFilter::ActiveByTipNoteIds(Vec::new()))
            .await?;
        assert!(empty.is_empty(), "empty note-id set should return no rows");

        // Non-matching note id: no rows.
        let none = store
            .list_pswap_lineages(PswapLineageFilter::ActiveByTipNoteIds(vec![phantom_tip]))
            .await?;
        assert!(none.is_empty(), "non-matching note id should return no rows");

        // Matching note id: returns the row.
        let one = store
            .list_pswap_lineages(PswapLineageFilter::ActiveByTipNoteIds(vec![real_tip]))
            .await?;
        assert_eq!(one.len(), 1, "matching note id should return its lineage");
        assert_eq!(one[0].current_tip_note_id, real_tip);

        // Mixed set (real + phantom): returns just the real one.
        let mixed = store
            .list_pswap_lineages(PswapLineageFilter::ActiveByTipNoteIds(vec![
                phantom_tip,
                real_tip,
                phantom_tip,
            ]))
            .await?;
        assert_eq!(mixed.len(), 1, "mixed set should return only the matching lineage");
        assert_eq!(mixed[0].current_tip_note_id, real_tip);

        Ok(())
    }

    // =========================================================================
    // PRIVATE-PSWAP END-TO-END VIA THE OBSERVER PIPELINE
    // =========================================================================
    //
    // Drives the observer pipeline for a private PSWAP: Alice creates P0, Bob
    // partial-fills at depth 1, and Alice's wallet runs observe + apply,
    // advancing the lineage to depth 1.

    mod private_pswap_e2e {
        use std::sync::Arc;

        use miden_client::account::AccountId;
        use miden_client::asset::FungibleAsset;
        use miden_client::note::{BlockNumber, Note, NoteInclusionProof, PswapNote};
        use miden_client::pswap::{PswapChainObserver, PswapLineageFilter, PswapLineageState};
        use miden_client::store::Store;
        use miden_client::sync::{NoteObserver, StateSyncUpdate};
        use miden_protocol::Word;
        use miden_protocol::asset::AssetAmount;
        use miden_protocol::crypto::merkle::SparseMerklePath;
        use miden_protocol::note::NoteType;
        use miden_protocol::testing::account_id::{
            ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
            ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_1,
            ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
            ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2,
        };
        use miden_standards::note::{PswapNoteAttachment, PswapNoteStorage};

        use crate::tests::create_test_store;

        /// Builds a private-type PSWAP with fixed fixtures (deterministic
        /// `order_id`).
        fn build_private_test_pswap(offered_amount: u64, requested_amount: u64) -> PswapNote {
            let sender =
                AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE).unwrap();
            let creator =
                AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2).unwrap();
            let offered_faucet = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET).unwrap();
            let requested_faucet =
                AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_1).unwrap();

            let storage = PswapNoteStorage::builder()
                .requested_asset(FungibleAsset::new(requested_faucet, requested_amount).unwrap())
                .creator_account_id(creator)
                .build();
            PswapNote::builder()
                .sender(sender)
                .storage(storage)
                .serial_number(Word::from([
                    miden_protocol::Felt::new(7).unwrap(),
                    miden_protocol::Felt::new(8).unwrap(),
                    miden_protocol::Felt::new(9).unwrap(),
                    miden_protocol::Felt::new(10).unwrap(),
                ]))
                .note_type(NoteType::Private)
                .offered_asset(FungibleAsset::new(offered_faucet, offered_amount).unwrap())
                .build()
                .unwrap()
        }

        /// Minimum-valid inclusion proof (empty Merkle path, index 0). The
        /// observer / correlator pipeline never inspects the path bytes; only
        /// the `block_num` is read for downstream apply.
        fn dummy_inclusion_proof(block: u32) -> NoteInclusionProof {
            let path = SparseMerklePath::from_parts(0, std::vec::Vec::new())
                .expect("empty SparseMerklePath is valid");
            NoteInclusionProof::new(BlockNumber::from(block), 0, path)
                .expect("zero index is well below per-block notes ceiling")
        }

        // ----- Convenience constructors used across scenarios. -----

        /// Builds a `CommittedNote` (the per-note input observer sees).
        fn commit_note(
            note: &Note,
            inclusion_proof: &NoteInclusionProof,
        ) -> miden_client::rpc::domain::note::CommittedNote {
            miden_client::rpc::domain::note::CommittedNote::new(
                note.id(),
                *note.metadata(),
                inclusion_proof.clone(),
            )
        }

        /// Builds a `StateSyncUpdate` whose `note_updates` carries the
        /// given notes as already-consumed input notes. The PSWAP
        /// correlator reads them via `note_updates.consumed_note_ids()`.
        ///
        /// `ConsumedUnauthenticatedLocal` is required (not
        /// `ConsumedExternal`) because `NoteUpdateTracker::insert_input_note`
        /// needs the metadata that variant retains.
        fn consumed_notes_window(entries: Vec<(&Note, u32)>) -> StateSyncUpdate {
            use miden_client::account::AccountId;
            use miden_client::note::NoteDetails;
            use miden_client::store::InputNoteRecord;
            use miden_client::store::input_note_states::{
                ConsumedUnauthenticatedLocalNoteState,
                NoteSubmissionData,
            };
            use miden_protocol::Word;
            use miden_protocol::transaction::TransactionId;

            let dummy_consumer =
                AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE).unwrap();
            let dummy_tx = TransactionId::from_raw(Word::default());

            let updated_input_notes: Vec<InputNoteRecord> = entries
                .into_iter()
                .map(|(note, block)| {
                    let details = NoteDetails::from(note.clone());
                    let attachments = note.attachments().clone();
                    let state = ConsumedUnauthenticatedLocalNoteState {
                        metadata: *note.metadata(),
                        nullifier_block_height: BlockNumber::from(block),
                        submission_data: NoteSubmissionData {
                            submitted_at: None,
                            consumer_account: dummy_consumer,
                            consumer_transaction: dummy_tx,
                        },
                        consumed_tx_order: None,
                    }
                    .into();
                    InputNoteRecord::new(details, attachments, None, state)
                })
                .collect();

            let note_updates = miden_client::note::NoteUpdateTracker::for_transaction_updates(
                Vec::new(),
                updated_input_notes,
                Vec::new(),
            );
            StateSyncUpdate {
                note_updates,
                ..StateSyncUpdate::default()
            }
        }

        /// Bob's account id — used as the consumer in every scenario.
        fn bob() -> AccountId {
            AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE).unwrap()
        }

        /// End-to-end private-PSWAP partial-fill scenario.
        ///
        /// 1. Alice creates a private PSWAP P0 (offer 100 OA, request 50 RA).
        /// 2. Bob partial-fills with 20 RA → emits private payback (20 RA to Alice) + private
        ///    remainder P1 (offer 60 OA, request 30 RA).
        /// 3. Alice's wallet syncs:
        ///    - `observer.observe()` runs per note with the note's inline attachments → pushes both
        ///      to pending
        ///    - `observer.apply()` runs the correlator, advances lineage to depth 1
        /// 4. Assert: lineage in DB advanced to depth 1, state=Active, tip=remainder,
        ///    `remaining_offered=60`, `remaining_requested=30`.
        #[tokio::test]
        async fn private_pswap_partial_fill_advances_lineage_end_to_end() -> anyhow::Result<()> {
            let store: Arc<dyn Store> = Arc::new(create_test_store().await);
            let bob =
                AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE).unwrap();

            // 1. Alice's PSWAP + lineage row.
            let pswap = build_private_test_pswap(100, 50);
            let lineage_record = super::build_initial_record(pswap.clone());
            store.upsert_pswap_lineage(&lineage_record).await?;
            // PswapNote doesn't expose nullifier directly — derive it from the Note view.

            // 2. Bob's payback + remainder for the partial fill at depth 1.
            let fill_amount = AssetAmount::new(20).unwrap();
            let payout_amount = AssetAmount::new(40).unwrap();
            let new_offered = AssetAmount::new(60).unwrap(); // 100 - 40
            let new_requested = AssetAmount::new(30).unwrap(); // 50 - 20

            let payback_attach = PswapNoteAttachment::new(fill_amount, pswap.order_id(), 1);
            let payback = pswap.payback_note(bob, &payback_attach).unwrap();

            let remainder_attach = PswapNoteAttachment::new(payout_amount, pswap.order_id(), 1);
            let remainder = pswap
                .remainder_note(bob, &remainder_attach, new_offered, new_requested)
                .unwrap();

            // 3. Drive the observe / apply phases, passing each note's attachments.
            let inclusion_proof = dummy_inclusion_proof(5);
            let observer = PswapChainObserver::new(store.clone());

            observer
                .observe(&commit_note(&payback, &inclusion_proof), Some(payback.attachments()))
                .await?;
            observer
                .observe(&commit_note(&remainder, &inclusion_proof), Some(remainder.attachments()))
                .await?;

            // P0 was consumed by Bob this sync.
            let p0_note = Note::from(pswap.clone());
            observer.apply(&consumed_notes_window(vec![(&p0_note, 5)])).await?;

            // 5. Assert: lineage in store advanced to depth 1.
            let lineage = store
                .get_pswap_lineage(pswap.order_id())
                .await?
                .expect("lineage exists in store");
            assert_eq!(lineage.current_depth, 1, "lineage advanced to depth 1");
            assert_eq!(lineage.state, PswapLineageState::Active, "still Active after partial fill");
            assert_eq!(lineage.current_tip_note_id, remainder.id(), "tip moved to remainder");
            assert_eq!(lineage.remaining_offered.amount(), new_offered);
            assert_eq!(lineage.remaining_requested.amount(), new_requested);

            // 6. Lineage should now be visible by the new tip's note id (proving the remainder is
            //    correctly tracked for round N+1 detection — see layer-2 fix commit d6995a76).
            let by_tip = store
                .list_pswap_lineages(PswapLineageFilter::ActiveByTipNoteIds(vec![remainder.id()]))
                .await?;
            assert_eq!(by_tip.len(), 1, "lineage findable by new tip note id");

            Ok(())
        }

        // =================================================================
        // FUNCTIONAL SCENARIOS — full fill, reclaim, multi-round
        // =================================================================

        /// Full fill (depth 1) → lineage state becomes `FullyFilled`, no remainder.
        ///
        /// Bob exhausts Alice's offered side in one shot. Only the payback
        /// is emitted; the PSWAP script has nothing left to remainder.
        #[tokio::test]
        async fn private_pswap_full_fill_marks_fully_filled() -> anyhow::Result<()> {
            let store: Arc<dyn Store> = Arc::new(create_test_store().await);
            let pswap = build_private_test_pswap(100, 50);
            store.upsert_pswap_lineage(&super::build_initial_record(pswap.clone())).await?;

            // Bob fills the entire 50 RA → 1 payback note (50 RA to Alice).
            let fill_amount = AssetAmount::new(50).unwrap();
            let payback_attach = PswapNoteAttachment::new(fill_amount, pswap.order_id(), 1);
            let payback = pswap.payback_note(bob(), &payback_attach).unwrap();

            let inclusion_proof = dummy_inclusion_proof(7);
            let observer = PswapChainObserver::new(store.clone());
            observer
                .observe(&commit_note(&payback, &inclusion_proof), Some(payback.attachments()))
                .await?;
            let p0_note = Note::from(pswap.clone());
            observer.apply(&consumed_notes_window(vec![(&p0_note, 7)])).await?;

            let lineage = store.get_pswap_lineage(pswap.order_id()).await?.unwrap();
            assert_eq!(lineage.current_depth, 1);
            assert_eq!(lineage.state, PswapLineageState::FullyFilled);
            assert_eq!(lineage.remaining_offered.amount(), AssetAmount::ZERO);
            assert_eq!(lineage.remaining_requested.amount(), AssetAmount::ZERO);
            Ok(())
        }

        /// Reclaim → lineage state becomes Reclaimed. Reclaim emits zero
        /// notes; detection is nullifier-only (the creator's tx consumes the
        /// current tip via the PSWAP script's cancel branch).
        #[tokio::test]
        async fn private_pswap_reclaim_marks_reclaimed() -> anyhow::Result<()> {
            let store: Arc<dyn Store> = Arc::new(create_test_store().await);
            let pswap = build_private_test_pswap(100, 50);
            store.upsert_pswap_lineage(&super::build_initial_record(pswap.clone())).await?;

            // No notes emitted by reclaim → no `observe()` calls at all.
            let observer = PswapChainObserver::new(store.clone());
            let p0_note = Note::from(pswap.clone());
            observer.apply(&consumed_notes_window(vec![(&p0_note, 9)])).await?;

            let lineage = store.get_pswap_lineage(pswap.order_id()).await?.unwrap();
            assert_eq!(lineage.state, PswapLineageState::Reclaimed);
            assert_eq!(lineage.remaining_offered.amount(), AssetAmount::ZERO);
            assert_eq!(lineage.remaining_requested.amount(), AssetAmount::ZERO);
            Ok(())
        }

        /// Same-sync multi-fill — two consecutive rounds land in one sync
        /// window. The `while consumed_note_ids.contains(&tip)` loop in
        /// `discover_pswap_rounds` walks both via in-memory advancement.
        /// Without it, round 2's remainder tip would never reappear in a
        /// later consumed-note set → silently lost.
        #[tokio::test]
        async fn private_pswap_multi_round_same_sync_advances_twice() -> anyhow::Result<()> {
            let store: Arc<dyn Store> = Arc::new(create_test_store().await);
            let pswap = build_private_test_pswap(100, 50);
            store.upsert_pswap_lineage(&super::build_initial_record(pswap.clone())).await?;

            // Round 1: Bob partial-fill (fill=20, payout=40) → payback + remainder.
            let p1_attach =
                PswapNoteAttachment::new(AssetAmount::new(20).unwrap(), pswap.order_id(), 1);
            let payback_1 = pswap.payback_note(bob(), &p1_attach).unwrap();
            let r1_attach =
                PswapNoteAttachment::new(AssetAmount::new(40).unwrap(), pswap.order_id(), 1);
            let remainder_1 = pswap
                .remainder_note(
                    bob(),
                    &r1_attach,
                    AssetAmount::new(60).unwrap(),
                    AssetAmount::new(30).unwrap(),
                )
                .unwrap();

            // Round 2: a different consumer fully fills the remainder (fill=30, payout=60).
            let p2_attach =
                PswapNoteAttachment::new(AssetAmount::new(30).unwrap(), pswap.order_id(), 2);
            let payback_2 = pswap.payback_note(bob(), &p2_attach).unwrap();

            let inclusion_proof = dummy_inclusion_proof(15);
            let observer = PswapChainObserver::new(store.clone());
            observer
                .observe(&commit_note(&payback_1, &inclusion_proof), Some(payback_1.attachments()))
                .await?;
            observer
                .observe(
                    &commit_note(&remainder_1, &inclusion_proof),
                    Some(remainder_1.attachments()),
                )
                .await?;
            observer
                .observe(&commit_note(&payback_2, &inclusion_proof), Some(payback_2.attachments()))
                .await?;

            // BOTH P0 and the round-1 remainder are in the consumed window.
            let p0_note = Note::from(pswap.clone());
            observer
                .apply(&consumed_notes_window(vec![(&p0_note, 15), (&remainder_1, 15)]))
                .await?;

            let lineage = store.get_pswap_lineage(pswap.order_id()).await?.unwrap();
            assert_eq!(lineage.current_depth, 2, "advanced through both rounds in one sync");
            assert_eq!(lineage.state, PswapLineageState::FullyFilled);
            assert_eq!(lineage.remaining_offered.amount(), AssetAmount::ZERO);
            assert_eq!(lineage.remaining_requested.amount(), AssetAmount::ZERO);
            Ok(())
        }

        // =================================================================
        // SECURITY / ADVERSARIAL SCENARIOS
        // =================================================================

        /// **Security**: a PSWAP-attachment note belonging to an order we
        /// DON'T track must not affect our store. Defense-in-depth: even
        /// though the SQL filter in `discover_pswap_rounds`
        /// (`ActiveByTipNoteIds`) doesn't return it, the `apply()`
        /// active-lineage filter is the second line of defense — both
        /// must agree on "not ours, skip".
        #[tokio::test]
        async fn foreign_pswap_attachment_note_is_filtered_out() -> anyhow::Result<()> {
            let store: Arc<dyn Store> = Arc::new(create_test_store().await);

            // Build a PSWAP whose order_id this client does NOT track (we
            // never call upsert_pswap_lineage for it).
            let foreign = {
                let sender =
                    AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE_2)
                        .unwrap();
                let creator =
                    AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE).unwrap();
                let offered_faucet =
                    AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET).unwrap();
                let requested_faucet =
                    AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_1).unwrap();
                let storage = PswapNoteStorage::builder()
                    .requested_asset(FungibleAsset::new(requested_faucet, 50).unwrap())
                    .creator_account_id(creator)
                    .build();
                PswapNote::builder()
                    .sender(sender)
                    .storage(storage)
                    .serial_number(Word::from([
                        miden_protocol::Felt::new(99).unwrap(),
                        miden_protocol::Felt::new(98).unwrap(),
                        miden_protocol::Felt::new(97).unwrap(),
                        miden_protocol::Felt::new(96).unwrap(),
                    ]))
                    .note_type(NoteType::Private)
                    .offered_asset(FungibleAsset::new(offered_faucet, 100).unwrap())
                    .build()
                    .unwrap()
            };

            // Foreign filler emits a payback for the foreign PSWAP.
            let foreign_payback = foreign
                .payback_note(
                    bob(),
                    &PswapNoteAttachment::new(AssetAmount::new(20).unwrap(), foreign.order_id(), 1),
                )
                .unwrap();

            let inclusion_proof = dummy_inclusion_proof(5);
            let observer = PswapChainObserver::new(store.clone());

            // We see the note arrive in sync.
            observer
                .observe(
                    &commit_note(&foreign_payback, &inclusion_proof),
                    Some(foreign_payback.attachments()),
                )
                .await?;
            // And the foreign PSWAP's nullifier is in the consumed window.
            let foreign_p0_note = Note::from(foreign.clone());
            observer.apply(&consumed_notes_window(vec![(&foreign_p0_note, 5)])).await?;

            // Store must remain empty — we never tracked this lineage.
            assert!(
                store.get_pswap_lineage(foreign.order_id()).await?.is_none(),
                "foreign PSWAP must not be inserted into our store",
            );
            Ok(())
        }

        /// **Security**: a stale note replayed without its tip being
        /// consumed must not advance the lineage. With no consumed note in
        /// the window, `discover_pswap_rounds` loads no active lineage (its
        /// `ActiveByTipNoteIds` set is empty) and the replay is ignored.
        #[tokio::test]
        async fn stale_depth_payback_does_not_advance_lineage() -> anyhow::Result<()> {
            let store: Arc<dyn Store> = Arc::new(create_test_store().await);
            let pswap = build_private_test_pswap(100, 50);

            // Insert lineage as if round 1 ALREADY happened (current_depth=1).
            let mut record = super::build_initial_record(pswap.clone());
            let r1_attach =
                PswapNoteAttachment::new(AssetAmount::new(40).unwrap(), pswap.order_id(), 1);
            let already_at = pswap
                .remainder_note(
                    bob(),
                    &r1_attach,
                    AssetAmount::new(60).unwrap(),
                    AssetAmount::new(30).unwrap(),
                )
                .unwrap();
            record.current_depth = 1;
            record.current_tip_note_id = already_at.id();
            let offered_faucet = pswap.offered_asset().faucet_id();
            let requested_faucet = pswap.storage().requested_asset().faucet_id();
            record.remaining_offered =
                miden_protocol::asset::FungibleAsset::new(offered_faucet, 60).unwrap();
            record.remaining_requested =
                miden_protocol::asset::FungibleAsset::new(requested_faucet, 30).unwrap();
            store.upsert_pswap_lineage(&record).await?;

            // Sync replays an old depth-1 payback (stale).
            let stale_payback = pswap
                .payback_note(
                    bob(),
                    &PswapNoteAttachment::new(AssetAmount::new(20).unwrap(), pswap.order_id(), 1),
                )
                .unwrap();

            let inclusion_proof = dummy_inclusion_proof(20);
            let observer = PswapChainObserver::new(store.clone());
            observer
                .observe(
                    &commit_note(&stale_payback, &inclusion_proof),
                    Some(stale_payback.attachments()),
                )
                .await?;
            // NOTE: we deliberately do NOT include any nullifier — no new
            // round happened. Just a stale note replay.
            observer.apply(&consumed_notes_window(vec![])).await?;

            let lineage = store.get_pswap_lineage(pswap.order_id()).await?.unwrap();
            assert_eq!(lineage.current_depth, 1, "stale depth-1 must not re-advance to 2");
            assert_eq!(lineage.state, PswapLineageState::Active);
            Ok(())
        }

        /// **Security**: a terminal-state lineage (`FullyFilled` or `Reclaimed`)
        /// must NOT be advanced even if its old tip nullifier shows up in
        /// the window again (e.g. via re-org or replayed sync data).
        /// Two defenses: `ActiveByTipNoteIds` SQL filter excludes
        /// non-Active rows; the `apply()` active-lineage filter is the second
        /// line of defense.
        #[tokio::test]
        async fn terminal_lineage_is_not_re_advanced() -> anyhow::Result<()> {
            let store: Arc<dyn Store> = Arc::new(create_test_store().await);
            let pswap = build_private_test_pswap(100, 50);

            // Insert as FullyFilled.
            let mut record = super::build_initial_record(pswap.clone());
            record.state = PswapLineageState::FullyFilled;
            store.upsert_pswap_lineage(&record).await?;
            // Attempt to replay a fill on the terminal lineage.
            let zombie_payback = pswap
                .payback_note(
                    bob(),
                    &PswapNoteAttachment::new(AssetAmount::new(20).unwrap(), pswap.order_id(), 1),
                )
                .unwrap();
            let inclusion_proof = dummy_inclusion_proof(30);
            let observer = PswapChainObserver::new(store.clone());
            observer
                .observe(
                    &commit_note(&zombie_payback, &inclusion_proof),
                    Some(zombie_payback.attachments()),
                )
                .await?;
            let p0_note = Note::from(pswap.clone());
            observer.apply(&consumed_notes_window(vec![(&p0_note, 30)])).await?;

            // State unchanged: still FullyFilled at the same depth.
            let lineage = store.get_pswap_lineage(pswap.order_id()).await?.unwrap();
            assert_eq!(lineage.state, PswapLineageState::FullyFilled);
            assert_eq!(lineage.current_depth, record.current_depth);
            Ok(())
        }

        // Note: tampered-attachment security test deferred to phase 2 —
        // the protocol's `payback_note` / `remainder_note` calls in
        // pswap/discovery.rs don't yet verify the reconstructed id
        // against the on-chain id. Re-introduce when adding that defense.

        /// Defensive fast-path: empty sync window AND empty pending → no
        /// store query, no RPC call, return Ok early. Verifies the
        /// short-circuit doesn't accidentally touch the store.
        #[tokio::test]
        async fn empty_sync_is_no_op() -> anyhow::Result<()> {
            let store: Arc<dyn Store> = Arc::new(create_test_store().await);
            let pswap = build_private_test_pswap(100, 50);
            let record = super::build_initial_record(pswap.clone());
            store.upsert_pswap_lineage(&record).await?;

            let observer = PswapChainObserver::new(store.clone());
            // No observe() calls, empty consumed-notes window.
            observer.apply(&consumed_notes_window(vec![])).await?;

            // Lineage untouched.
            let lineage = store.get_pswap_lineage(pswap.order_id()).await?.unwrap();
            assert_eq!(lineage.current_depth, 0);
            assert_eq!(lineage.state, PswapLineageState::Active);
            Ok(())
        }

        /// Dedup: when the payback note is ALREADY present in `input_notes`
        /// (the public-note path, where the screener inserts a record during
        /// the same sync), `apply_pswap_round` must NOT overwrite it. The
        /// round still applies and the lineage advances; the pre-existing
        /// `Expected` record is left intact rather than clobbered with an
        /// `Unverified` one.
        #[tokio::test]
        async fn apply_round_skips_insert_when_payback_already_present() -> anyhow::Result<()> {
            use miden_client::store::{InputNoteRecord, InputNoteState, NoteFilter};

            let store: Arc<dyn Store> = Arc::new(create_test_store().await);
            let pswap = build_private_test_pswap(100, 50);
            store.upsert_pswap_lineage(&super::build_initial_record(pswap.clone())).await?;

            // Partial fill at depth 1: payback (20 RA) + remainder (60 OA / 30 RA).
            let payback_attach =
                PswapNoteAttachment::new(AssetAmount::new(20).unwrap(), pswap.order_id(), 1);
            let payback = pswap.payback_note(bob(), &payback_attach).unwrap();
            let remainder_attach =
                PswapNoteAttachment::new(AssetAmount::new(40).unwrap(), pswap.order_id(), 1);
            let remainder = pswap
                .remainder_note(
                    bob(),
                    &remainder_attach,
                    AssetAmount::new(60).unwrap(),
                    AssetAmount::new(30).unwrap(),
                )
                .unwrap();

            // Pre-seed the payback as an Expected note — mirrors the screener
            // having already inserted it for a public payback.
            store.upsert_input_notes(&[InputNoteRecord::from(payback.clone())]).await?;

            let inclusion_proof = dummy_inclusion_proof(5);
            let observer = PswapChainObserver::new(store.clone());
            observer
                .observe(&commit_note(&payback, &inclusion_proof), Some(payback.attachments()))
                .await?;
            observer
                .observe(&commit_note(&remainder, &inclusion_proof), Some(remainder.attachments()))
                .await?;
            let p0_note = Note::from(pswap.clone());
            observer.apply(&consumed_notes_window(vec![(&p0_note, 5)])).await?;

            // Round applied: lineage advanced to depth 1.
            let lineage = store.get_pswap_lineage(pswap.order_id()).await?.unwrap();
            assert_eq!(lineage.current_depth, 1);

            // Pre-existing payback record untouched: exactly one row, still Expected.
            let rows = store.get_input_notes(NoteFilter::List(vec![payback.id()])).await?;
            assert_eq!(rows.len(), 1, "payback must not be duplicated");
            assert!(
                matches!(rows[0].state(), InputNoteState::Expected(_)),
                "dedup must leave the pre-existing Expected record intact, not overwrite it",
            );
            Ok(())
        }
    }
}
