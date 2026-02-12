#![allow(clippy::items_after_statements)]

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::note::{BlockNumber, NoteTag};
use miden_client::store::StoreError;
use miden_client::sync::{NoteTagRecord, NoteTagSource, StateSyncUpdate};
use miden_client::utils::{Deserializable, Serializable};

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub(crate) mod rusqlite_impl {
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;
    use std::sync::{Arc, RwLock};

    use miden_client::Word;
    use miden_client::note::{BlockNumber, NoteTag};
    use miden_client::store::StoreError;
    use miden_client::sync::{NoteTagRecord, NoteTagSource, StateSyncUpdate};
    use miden_client::utils::{Deserializable, Serializable};
    use rusqlite::{Connection, Transaction, params};

    use crate::SqliteStore;
    use crate::note::apply_note_updates_tx;
    use crate::smt_forest::AccountSmtForest;
    use crate::sql_error::SqlResultExt;
    use crate::transaction::upsert_transaction_record;

    impl SqliteStore {
        pub(crate) fn get_note_tags(
            conn: &mut Connection,
        ) -> Result<Vec<NoteTagRecord>, StoreError> {
            const QUERY: &str = "SELECT tag, source FROM tags";

            conn.prepare_cached(QUERY)
                .into_store_error()?
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .expect("no binding parameters used in query")
                .map(|result| {
                    let (tag, source): (Vec<u8>, Vec<u8>) = result.into_store_error()?;
                    Ok(NoteTagRecord {
                        tag: NoteTag::read_from_bytes(&tag)
                            .map_err(StoreError::DataDeserializationError)?,
                        source: NoteTagSource::read_from_bytes(&source)
                            .map_err(StoreError::DataDeserializationError)?,
                    })
                })
                .collect::<Result<Vec<NoteTagRecord>, _>>()
        }

        pub(crate) fn get_unique_note_tags(
            conn: &mut Connection,
        ) -> Result<BTreeSet<NoteTag>, StoreError> {
            const QUERY: &str = "SELECT DISTINCT tag FROM tags";

            conn.prepare_cached(QUERY)
                .into_store_error()?
                .query_map([], |row| row.get(0))
                .expect("no binding parameters used in query")
                .map(|result| {
                    let tag: Vec<u8> = result.into_store_error()?;
                    NoteTag::read_from_bytes(&tag).map_err(StoreError::DataDeserializationError)
                })
                .collect::<Result<BTreeSet<NoteTag>, _>>()
        }

        pub(crate) fn add_note_tag(
            conn: &mut Connection,
            tag: NoteTagRecord,
        ) -> Result<bool, StoreError> {
            if Self::get_note_tags(conn)?.contains(&tag) {
                return Ok(false);
            }

            let tx = conn.transaction().into_store_error()?;
            add_note_tag_tx(&tx, &tag)?;

            tx.commit().into_store_error()?;

            Ok(true)
        }

        pub(crate) fn remove_note_tag(
            conn: &mut Connection,
            tag: NoteTagRecord,
        ) -> Result<usize, StoreError> {
            let tx = conn.transaction().into_store_error()?;
            let removed_tags = remove_note_tag_tx(&tx, tag)?;

            tx.commit().into_store_error()?;

            Ok(removed_tags)
        }

        pub(crate) fn get_sync_height(conn: &mut Connection) -> Result<BlockNumber, StoreError> {
            const QUERY: &str = "SELECT block_num FROM state_sync";

            conn.prepare_cached(QUERY)
                .into_store_error()?
                .query_map([], |row| row.get(0))
                .expect("no binding parameters used in query")
                .map(|result| {
                    let v: i64 = result.into_store_error()?;
                    Ok(BlockNumber::from(
                        u32::try_from(v).expect("block number is always positive"),
                    ))
                })
                .next()
                .expect("state sync block number exists")
        }

        pub(crate) fn apply_state_sync(
            conn: &mut Connection,
            smt_forest: &Arc<RwLock<AccountSmtForest>>,
            state_sync_update: StateSyncUpdate,
        ) -> Result<(), StoreError> {
            let StateSyncUpdate {
                block_num,
                block_updates,
                note_updates,
                transaction_updates,
                account_updates,
            } = state_sync_update;

            let tx = conn.transaction().into_store_error()?;

            // Update state sync block number only if moving forward
            const BLOCK_NUMBER_QUERY: &str =
                "UPDATE state_sync SET block_num = ? WHERE block_num < ?";
            tx.execute(
                BLOCK_NUMBER_QUERY,
                params![i64::from(block_num.as_u32()), i64::from(block_num.as_u32())],
            )
            .into_store_error()?;

            for (block_header, block_has_relevant_notes, new_mmr_peaks) in
                block_updates.block_headers()
            {
                Self::insert_block_header_tx(
                    &tx,
                    block_header,
                    new_mmr_peaks,
                    *block_has_relevant_notes,
                )?;
            }

            // Insert new authentication nodes (inner nodes of the PartialBlockchain)
            Self::insert_partial_blockchain_nodes_tx(
                &tx,
                block_updates.new_authentication_nodes(),
            )?;

            // Update notes
            apply_note_updates_tx(&tx, &note_updates)?;

            // Remove tags
            let tags_to_remove = note_updates
                .updated_input_notes()
                .filter_map(|note_update| {
                    let note = note_update.inner();
                    if note.is_committed() {
                        Some(NoteTagRecord {
                            tag: note
                                .metadata()
                                .expect("Committed notes should have metadata")
                                .tag(),
                            source: NoteTagSource::Note(note.id()),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            for tag in tags_to_remove {
                remove_note_tag_tx(&tx, tag)?;
            }

            for transaction_record in transaction_updates
                .committed_transactions()
                .chain(transaction_updates.discarded_transactions())
            {
                upsert_transaction_record(&tx, transaction_record)?;
            }

            // Remove the accounts that are originated from the discarded transactions
            let account_hashes_to_delete: Vec<Word> = transaction_updates
                .discarded_transactions()
                .map(|tx| tx.details.final_account_state)
                .collect();

            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
            Self::undo_account_state(&tx, &mut smt_forest, &account_hashes_to_delete)?;

            // Update public accounts on the db that have been updated onchain
            for account in account_updates.updated_public_accounts() {
                Self::update_account_state(&tx, &mut smt_forest, account)?;
            }
            drop(smt_forest);

            for (account_id, digest) in account_updates.mismatched_private_accounts() {
                Self::lock_account_on_unexpected_commitment(&tx, account_id, digest)?;
            }

            // Commit the updates
            tx.commit().into_store_error()?;

            Ok(())
        }
    }

    pub(crate) fn add_note_tag_tx(
        tx: &Transaction<'_>,
        tag: &NoteTagRecord,
    ) -> Result<(), StoreError> {
        const QUERY: &str = insert_sql!(tags { tag, source });
        tx.execute(QUERY, params![tag.tag.to_bytes(), tag.source.to_bytes()])
            .into_store_error()?;

        Ok(())
    }

    pub(crate) fn remove_note_tag_tx(
        tx: &Transaction<'_>,
        tag: NoteTagRecord,
    ) -> Result<usize, StoreError> {
        const QUERY: &str = "DELETE FROM tags WHERE tag = ? AND source = ?";
        let removed_tags = tx
            .execute(QUERY, params![tag.tag.to_bytes(), tag.source.to_bytes()])
            .into_store_error()?;

        Ok(removed_tags)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use rusqlite_impl::{add_note_tag_tx, remove_note_tag_tx};

// SHARED SqlConnection-based functions
// ================================================================================================
use crate::sql_types::{SqlConnection, SqlParam};

/// Retrieve all note tags using [`SqlConnection`].
pub(crate) fn get_note_tags_shared(
    conn: &dyn SqlConnection,
) -> Result<Vec<NoteTagRecord>, StoreError> {
    let rows = conn.query_all("SELECT tag, source FROM tags", &[])?;
    rows.into_iter()
        .map(|row| {
            let tag = NoteTag::read_from_bytes(row.get_blob(0)?)
                .map_err(StoreError::DataDeserializationError)?;
            let source = NoteTagSource::read_from_bytes(row.get_blob(1)?)
                .map_err(StoreError::DataDeserializationError)?;
            Ok(NoteTagRecord { tag, source })
        })
        .collect()
}

/// Retrieve unique note tags using [`SqlConnection`].
pub(crate) fn get_unique_note_tags_shared(
    conn: &dyn SqlConnection,
) -> Result<BTreeSet<NoteTag>, StoreError> {
    let rows = conn.query_all("SELECT DISTINCT tag FROM tags", &[])?;
    rows.into_iter()
        .map(|row| {
            NoteTag::read_from_bytes(row.get_blob(0)?).map_err(StoreError::DataDeserializationError)
        })
        .collect()
}

/// Get the current sync height using [`SqlConnection`].
pub(crate) fn get_sync_height_shared(conn: &dyn SqlConnection) -> Result<BlockNumber, StoreError> {
    let row = conn
        .query_one("SELECT block_num FROM state_sync", &[])?
        .expect("state sync block number exists");
    let block_num = row.get_u32(0)?;
    Ok(BlockNumber::from(block_num))
}

/// Add a note tag using [`SqlConnection`]. Returns `true` if the tag was added.
pub(crate) fn add_note_tag_shared(
    conn: &dyn SqlConnection,
    tag: &NoteTagRecord,
) -> Result<bool, StoreError> {
    // Check if tag already exists
    let existing = get_note_tags_shared(conn)?;
    if existing.contains(tag) {
        return Ok(false);
    }

    insert_note_tag_shared(conn, tag)?;
    Ok(true)
}

/// Insert a note tag using [`SqlConnection`].
pub(crate) fn insert_note_tag_shared(
    conn: &dyn SqlConnection,
    tag: &NoteTagRecord,
) -> Result<(), StoreError> {
    const QUERY: &str = insert_sql!(tags { tag, source });
    conn.execute(
        QUERY,
        &[SqlParam::Blob(tag.tag.to_bytes()), SqlParam::Blob(tag.source.to_bytes())],
    )?;
    Ok(())
}

/// Remove a note tag using [`SqlConnection`]. Returns number of removed tags.
pub(crate) fn remove_note_tag_shared(
    conn: &dyn SqlConnection,
    tag: &NoteTagRecord,
) -> Result<usize, StoreError> {
    const QUERY: &str = "DELETE FROM tags WHERE tag = ? AND source = ?";
    let removed = conn.execute(
        QUERY,
        &[SqlParam::Blob(tag.tag.to_bytes()), SqlParam::Blob(tag.source.to_bytes())],
    )?;
    Ok(removed)
}

/// Apply a full state sync update using [`SqlConnection`] (no SMT).
///
/// This is the shared version that works across both native and WASM backends.
/// On native, the SMT-aware version in `SqliteStore::apply_state_sync` is used instead.
#[allow(dead_code)]
pub(crate) fn apply_state_sync_shared(
    conn: &dyn SqlConnection,
    state_sync_update: StateSyncUpdate,
) -> Result<(), StoreError> {
    let StateSyncUpdate {
        block_num,
        block_updates,
        note_updates,
        transaction_updates,
        account_updates,
    } = state_sync_update;

    // Update state sync block number only if moving forward
    conn.execute(
        "UPDATE state_sync SET block_num = ? WHERE block_num < ?",
        &[SqlParam::from(block_num.as_u32()), SqlParam::from(block_num.as_u32())],
    )?;

    // Insert block headers
    for (block_header, has_client_notes, new_mmr_peaks) in block_updates.block_headers() {
        crate::chain_data::insert_block_header_shared(
            conn,
            block_header,
            new_mmr_peaks,
            *has_client_notes,
        )?;
    }

    // Insert new authentication nodes
    crate::chain_data::insert_partial_blockchain_nodes_shared(
        conn,
        block_updates.new_authentication_nodes(),
    )?;

    // Update notes
    crate::note::apply_note_updates_shared(conn, &note_updates)?;

    // Remove tags for committed input notes
    for note_update in note_updates.updated_input_notes() {
        let note = note_update.inner();
        if note.is_committed() {
            let tag = NoteTagRecord {
                tag: note.metadata().expect("Committed notes should have metadata").tag(),
                source: NoteTagSource::Note(note.id()),
            };
            remove_note_tag_shared(conn, &tag)?;
        }
    }

    // Upsert transactions
    for transaction_record in transaction_updates
        .committed_transactions()
        .chain(transaction_updates.discarded_transactions())
    {
        crate::transaction::upsert_transaction_record_shared(conn, transaction_record)?;
    }

    // Undo account states from discarded transactions
    let account_hashes_to_delete: Vec<Word> = transaction_updates
        .discarded_transactions()
        .map(|tx| tx.details.final_account_state)
        .collect();
    crate::account::shared::undo_account_states_shared(conn, &account_hashes_to_delete)?;

    // Update public accounts
    for account in account_updates.updated_public_accounts() {
        crate::account::shared::update_account_state_shared(conn, account)?;
    }

    // Lock mismatched private accounts
    for (account_id, digest) in account_updates.mismatched_private_accounts() {
        crate::account::shared::lock_account_on_unexpected_commitment_shared(
            conn, account_id, digest,
        )?;
    }

    Ok(())
}
