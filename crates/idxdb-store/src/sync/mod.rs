use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::account::{
    AccountDelta,
    AccountId,
    StorageMap,
    StorageSlotName,
    StorageSlotType,
};
use miden_client::asset::{Asset, AssetVaultKey};
use miden_client::note::{BlockNumber, NoteId, NoteTag};
use miden_client::store::{AccountStorageFilter, StoreError};
use miden_client::sync::{
    BlockUpdates,
    NoteTagRecord,
    NoteTagSource,
    PublicAccountDelta,
    PublicAccountUpdate,
    StateSyncUpdate,
};
use miden_client::utils::{Deserializable, Serializable};

use super::IdxdbStore;
use super::account::utils::{apply_transaction_delta, compute_storage_delta, compute_vault_delta};
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

impl IdxdbStore {
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

    #[allow(clippy::too_many_lines)]
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

        let committed_note_ids: Vec<String> = note_updates
            .updated_input_notes()
            .filter(|update| update.inner().is_committed())
            .map(|update| update.inner().id().to_string())
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

        // Separate full updates from delta updates
        let mut full_accounts = Vec::new();
        for update in account_updates.updated_public_accounts() {
            match update {
                PublicAccountUpdate::Full(account) => full_accounts.push(account),
                PublicAccountUpdate::Delta(delta) => {
                    self.apply_public_account_delta(delta).await?;
                },
            }
        }

        // Update SMT forest for full account updates (insert nodes + replace roots atomically)
        {
            let mut smt_forest = self.smt_forest.write();
            for account in &full_accounts {
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
            account_updates: full_accounts
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

    /// Reads the local account state, derives the [`AccountDelta`] from `delta`'s incremental
    /// payload, updates the SMT forest, and persists the changes through the JS layer.
    async fn apply_public_account_delta(
        &self,
        delta: &PublicAccountDelta,
    ) -> Result<(), StoreError> {
        let account_id = delta.id();

        // Read local state.
        let local_header = self
            .get_account_header(account_id)
            .await?
            .map(|(header, _)| header)
            .ok_or(StoreError::AccountDataNotFound(account_id))?;
        let local_storage = self.get_account_storage(account_id, AccountStorageFilter::All).await?;
        let local_vault = self.get_account_vault(account_id).await?;

        // Replay the incremental updates onto local state to derive the delta.
        let account_delta =
            delta.compute_account_delta(&local_header, &local_storage, &local_vault)?;

        // Load only the slices of state the apply path needs.
        let vault_keys: Vec<String> = account_delta
            .vault()
            .fungible()
            .iter()
            .map(|(vault_key, _)| vault_key.to_string())
            .collect();
        let old_vault_assets = self.get_vault_assets(account_id, vault_keys).await?;
        let map_slot_names: Vec<String> = account_delta
            .storage()
            .maps()
            .map(|(slot_name, _)| slot_name.to_string())
            .collect();
        let old_map_roots = self.get_storage_map_roots(account_id, map_slot_names).await?;

        let SmtForestDeltaUpdate {
            updated_storage_slots,
            updated_assets,
            removed_vault_keys,
        } = self.update_smt_forest_for_delta(
            account_id,
            &old_map_roots,
            &old_vault_assets,
            &account_delta,
        )?;

        apply_transaction_delta(
            self.db_id(),
            account_id,
            delta.new_header(),
            &updated_storage_slots,
            &updated_assets,
            &removed_vault_keys,
            &account_delta,
        )
        .await
        .map_err(|err| {
            StoreError::DatabaseError(format!("failed to apply sync account delta: {err:?}"))
        })
    }

    /// Updates the SMT forest with the storage and vault changes implied by `delta`. Returns
    /// the auxiliary data needed by `apply_transaction_delta` to persist the change.
    fn update_smt_forest_for_delta(
        &self,
        account_id: AccountId,
        old_map_roots: &BTreeMap<StorageSlotName, Word>,
        old_vault_assets: &[Asset],
        delta: &AccountDelta,
    ) -> Result<SmtForestDeltaUpdate, StoreError> {
        let mut smt_forest = self.smt_forest.write();

        let mut final_roots = smt_forest
            .get_roots(&account_id)
            .cloned()
            .ok_or(StoreError::AccountDataNotFound(account_id))?;

        let updated_storage_slots = compute_storage_delta(&mut smt_forest, old_map_roots, delta)?;

        let default_map_root = StorageMap::default().root();
        for (slot_name, (new_root, slot_type)) in &updated_storage_slots {
            if *slot_type == StorageSlotType::Map {
                let old_root = old_map_roots.get(slot_name).copied().unwrap_or(default_map_root);
                if let Some(root) = final_roots.iter_mut().find(|r| **r == old_root) {
                    *root = *new_root;
                } else {
                    final_roots.push(*new_root);
                }
            }
        }

        let old_vault_root = final_roots[0];
        let (updated_assets, removed_vault_keys) = compute_vault_delta(old_vault_assets, delta)?;
        let new_vault_root = smt_forest.update_asset_nodes(
            old_vault_root,
            updated_assets.iter().copied(),
            removed_vault_keys.iter().copied(),
        )?;
        final_roots[0] = new_vault_root;

        smt_forest.replace_roots(account_id, final_roots);

        Ok(SmtForestDeltaUpdate {
            updated_storage_slots,
            updated_assets,
            removed_vault_keys,
        })
    }
}

/// Result of `IdxdbStore::update_smt_forest_for_delta`.
struct SmtForestDeltaUpdate {
    /// New roots (and slot types) per map slot affected by the delta.
    updated_storage_slots: BTreeMap<StorageSlotName, (Word, StorageSlotType)>,
    /// Asset values implied by the delta — the values to insert or update in the vault.
    updated_assets: Vec<Asset>,
    /// Vault keys removed from the vault by the delta.
    removed_vault_keys: Vec<AssetVaultKey>,
}

type SerializedBlockData =
    (Vec<Vec<u8>>, Vec<Vec<u8>>, Vec<u32>, Vec<u8>, Vec<String>, Vec<String>);

fn serialize_block_updates(
    block_updates: &BlockUpdates,
) -> Result<SerializedBlockData, StoreError> {
    let mut block_headers_as_bytes = Vec::new();
    let mut new_mmr_peaks_as_bytes = Vec::new();
    let mut block_nums = Vec::new();
    let mut block_has_relevant_notes = Vec::new();

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
