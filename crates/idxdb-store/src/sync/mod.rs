use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::account::AccountId;
use miden_client::note::{BlockNumber, NoteId, NoteTag};
use miden_client::store::StoreError;
use miden_client::sync::{BlockUpdates, NoteTagRecord, NoteTagSource, StateSyncUpdate};
use miden_client::utils::{Deserializable, Serializable};

use super::WebStore;
use super::chain_data::utils::{
    SerializedPartialBlockchainNodeData,
    serialize_partial_blockchain_node,
};
use super::note::utils::{serialize_input_note, serialize_output_note};
use super::transaction::utils::serialize_transaction_record;
use crate::promise::{await_js, await_js_value};

mod js_bindings;
pub use js_bindings::JsAccountUpdate;
use js_bindings::{
    JsStateSyncUpdate,
    idxdb_add_note_tag,
    idxdb_apply_state_sync,
    idxdb_get_note_tags,
    idxdb_get_sync_height,
    idxdb_remove_note_tag,
};

mod models;
use models::{NoteTagIdxdbObject, SyncHeightIdxdbObject};

mod flattened_vec;
use flattened_vec::flatten_nested_u8_vec;

impl WebStore {
    pub(crate) async fn get_note_tags(&self) -> Result<Vec<NoteTagRecord>, StoreError> {
        let promise = idxdb_get_note_tags(self.db_id());
        let tags_idxdb: Vec<NoteTagIdxdbObject> =
            await_js(promise, "failed to get note tags").await?;

        let tags = tags_idxdb
            .into_iter()
            .map(|t| -> Result<NoteTagRecord, StoreError> {
                let source = match (t.source_account_id, t.source_note_id) {
                    (None, None) => NoteTagSource::User,
                    (Some(account_id), None) => {
                        NoteTagSource::Account(AccountId::from_hex(account_id.as_str())?)
                    },
                    (None, Some(note_id)) => {
                        NoteTagSource::Note(NoteId::try_from_hex(note_id.as_str())?)
                    },
                    _ => return Err(StoreError::ParsingError("Invalid NoteTagSource".to_string())),
                };

                Ok(NoteTagRecord {
                    tag: NoteTag::read_from_bytes(&t.tag)?,
                    source,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    pub(super) async fn get_sync_height(&self) -> Result<BlockNumber, StoreError> {
        let promise = idxdb_get_sync_height(self.db_id());
        let block_num_idxdb: SyncHeightIdxdbObject =
            await_js(promise, "failed to get sync height").await?;

        Ok(block_num_idxdb.block_num.into())
    }

    pub(super) async fn add_note_tag(&self, tag: NoteTagRecord) -> Result<bool, StoreError> {
        if self.get_note_tags().await?.contains(&tag) {
            return Ok(false);
        }

        let (source_note_id, source_account_id) = match tag.source {
            NoteTagSource::Note(note_id) => (Some(note_id.to_hex()), None),
            NoteTagSource::Account(account_id) => (None, Some(account_id.to_hex())),
            NoteTagSource::User => (None, None),
        };

        let promise =
            idxdb_add_note_tag(self.db_id(), tag.tag.to_bytes(), source_note_id, source_account_id);
        await_js_value(promise, "failed to add note tag").await?;

        Ok(true)
    }

    pub(super) async fn remove_note_tag(&self, tag: NoteTagRecord) -> Result<usize, StoreError> {
        let (source_note_id, source_account_id) = match tag.source {
            NoteTagSource::Note(note_id) => (Some(note_id.to_hex()), None),
            NoteTagSource::Account(account_id) => (None, Some(account_id.to_hex())),
            NoteTagSource::User => (None, None),
        };

        let promise = idxdb_remove_note_tag(
            self.db_id(),
            tag.tag.to_bytes(),
            source_note_id,
            source_account_id,
        );
        let removed_tags: usize = await_js(promise, "failed to remove note tag").await?;

        Ok(removed_tags)
    }

    pub(super) async fn apply_state_sync(
        &self,
        state_sync_update: StateSyncUpdate,
    ) -> Result<(), StoreError> {
        let StateSyncUpdate {
            block_num,
            block_updates,
            note_updates,
            transaction_updates,
            account_updates,
        } = state_sync_update;

        let (
            block_headers_as_bytes,
            new_mmr_peaks_as_bytes,
            block_nums,
            block_has_relevant_notes,
            serialized_node_ids,
            serialized_nodes,
        ) = serialize_block_updates(&block_updates)?;

        let (serialized_input_notes, serialized_output_notes): (Vec<_>, Vec<_>) = {
            let input_notes = note_updates.updated_input_notes();
            let output_notes = note_updates.updated_output_notes();
            (
                input_notes.into_iter().map(|note| serialize_input_note(note.inner())).collect(),
                output_notes
                    .into_iter()
                    .map(|note| serialize_output_note(note.inner()))
                    .collect(),
            )
        };

        // Tags to remove
        let committed_note_ids: Vec<String> = note_updates
            .updated_input_notes()
            .filter_map(|note_update| {
                let note = note_update.inner();
                if note.is_committed() {
                    Some(note.id().to_string())
                } else {
                    None
                }
            })
            .collect();

        for (account_id, digest) in account_updates.mismatched_private_accounts() {
            self.lock_account_on_unexpected_commitment(account_id, digest).await.map_err(
                |err| {
                    StoreError::DatabaseError(format!("failed to check account mismatch: {err:?}"))
                },
            )?;
        }

        let account_states_to_rollback = transaction_updates
            .discarded_transactions()
            .map(|tx_record| tx_record.details.final_account_state)
            .collect::<Vec<_>>();

        // Remove the account states and discard their SMT roots from the forest
        self.rollback_account_states(&account_states_to_rollback).await?;

        // Discard roots for rolled-back accounts
        {
            let mut smt_forest = self.smt_forest.write();
            for tx_record in transaction_updates.discarded_transactions() {
                smt_forest.discard_roots(tx_record.details.account_id);
            }
            // Commit roots for successfully committed transactions
            for tx_record in transaction_updates.committed_transactions() {
                smt_forest.commit_roots(tx_record.details.account_id);
            }
        }

        let transaction_updates: Vec<_> = transaction_updates
            .committed_transactions()
            .chain(transaction_updates.discarded_transactions())
            .map(serialize_transaction_record)
            .collect();

        // Update SMT forest for public account updates (insert nodes + replace roots atomically)
        {
            let mut smt_forest = self.smt_forest.write();
            for account in account_updates.updated_public_accounts() {
                smt_forest.insert_and_register_account_state(
                    account.id(),
                    account.vault(),
                    account.storage(),
                )?;
            }
        }

        let state_update = JsStateSyncUpdate {
            block_num: block_num.as_u32(),
            flattened_new_block_headers: flatten_nested_u8_vec(block_headers_as_bytes),
            new_block_nums: block_nums,
            flattened_partial_blockchain_peaks: flatten_nested_u8_vec(new_mmr_peaks_as_bytes),
            block_has_relevant_notes,
            serialized_node_ids,
            serialized_nodes,
            committed_note_ids,
            serialized_input_notes,
            serialized_output_notes,
            account_updates: account_updates
                .updated_public_accounts()
                .iter()
                .map(|account| JsAccountUpdate::from_account(account, None))
                .collect(),
            transaction_updates,
        };
        let promise = idxdb_apply_state_sync(self.db_id(), state_update);
        await_js_value(promise, "failed to apply state sync").await?;

        Ok(())
    }

    /// Rolls back account states by removing them from the DB.
    /// SMT root cleanup is handled separately via `discard_roots`.
    async fn rollback_account_states(
        &self,
        account_commitments: &[Word],
    ) -> Result<(), StoreError> {
        self.undo_account_states(account_commitments).await?;
        Ok(())
    }
}

type SerializedBlockData =
    (Vec<Vec<u8>>, Vec<Vec<u8>>, Vec<u32>, Vec<u8>, Vec<String>, Vec<String>);

fn serialize_block_updates(
    block_updates: &BlockUpdates,
) -> Result<SerializedBlockData, StoreError> {
    let block_headers_len = block_updates.block_headers().len();
    let mut block_headers_as_bytes = Vec::with_capacity(block_headers_len);
    let mut new_mmr_peaks_as_bytes = Vec::with_capacity(block_headers_len);
    let mut block_nums = Vec::with_capacity(block_headers_len);
    let mut block_has_relevant_notes = Vec::with_capacity(block_headers_len);

    for (block_header, has_client_notes, mmr_peaks) in block_updates.block_headers() {
        block_headers_as_bytes.push(block_header.to_bytes());
        new_mmr_peaks_as_bytes.push(mmr_peaks.peaks().to_vec().to_bytes());
        block_nums.push(block_header.block_num().as_u32());
        block_has_relevant_notes.push(u8::from(*has_client_notes));
    }

    let auth_nodes_len = block_updates.new_authentication_nodes().len();
    let mut serialized_node_ids = Vec::with_capacity(auth_nodes_len);
    let mut serialized_nodes = Vec::with_capacity(auth_nodes_len);
    for (id, node) in block_updates.new_authentication_nodes() {
        let SerializedPartialBlockchainNodeData { id, node } =
            serialize_partial_blockchain_node(*id, *node)?;
        serialized_node_ids.push(id);
        serialized_nodes.push(node);
    }

    Ok((
        block_headers_as_bytes,
        new_mmr_peaks_as_bytes,
        block_nums,
        block_has_relevant_notes,
        serialized_node_ids,
        serialized_nodes,
    ))
}
