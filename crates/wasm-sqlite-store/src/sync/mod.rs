use alloc::string::ToString;
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::account::{Account, StorageSlotContent};
use miden_client::note::BlockNumber;
use miden_client::store::StoreError;
use miden_client::sync::{NoteTagRecord, NoteTagSource, StateSyncUpdate};
use miden_client::utils::{Deserializable, Serializable};
use serde::Serialize;

use super::WasmSqliteStore;

mod js_bindings;
use js_bindings::{
    js_add_note_tag,
    js_apply_state_sync,
    js_get_note_tags,
    js_get_sync_height,
    js_remove_note_tag,
};

mod models;
use models::{NoteTagObject, SyncHeightObject};

impl WasmSqliteStore {
    #[allow(clippy::unused_async)]
    pub(crate) async fn get_note_tags(&self) -> Result<Vec<NoteTagRecord>, StoreError> {
        let js_value = js_get_note_tags(self.db_id());
        let tags: Vec<NoteTagObject> = serde_wasm_bindgen::from_value(js_value).map_err(|err| {
            StoreError::DatabaseError(format!("failed to deserialize note tags: {err:?}"))
        })?;

        tags.into_iter()
            .map(|t| {
                let tag = miden_client::note::NoteTag::read_from_bytes(&t.tag)?;
                let source = NoteTagSource::read_from_bytes(&t.source)?;
                Ok(NoteTagRecord { tag, source })
            })
            .collect::<Result<Vec<_>, StoreError>>()
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_sync_height(&self) -> Result<BlockNumber, StoreError> {
        let js_value = js_get_sync_height(self.db_id());
        if js_value.is_null() || js_value.is_undefined() {
            return Ok(BlockNumber::from(0u32));
        }

        let sync_height: SyncHeightObject =
            serde_wasm_bindgen::from_value(js_value).map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize sync height: {err:?}"))
            })?;

        Ok(sync_height.block_num.into())
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn add_note_tag(&self, tag: NoteTagRecord) -> Result<bool, StoreError> {
        let tag_bytes = tag.tag.to_bytes();
        let source_bytes = tag.source.to_bytes();
        let added = js_add_note_tag(self.db_id(), tag_bytes, source_bytes);
        Ok(added)
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn remove_note_tag(&self, tag: NoteTagRecord) -> Result<usize, StoreError> {
        let tag_bytes = tag.tag.to_bytes();
        let source_bytes = tag.source.to_bytes();
        let removed = js_remove_note_tag(self.db_id(), tag_bytes, source_bytes);
        Ok(removed as usize)
    }

    #[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
    pub(crate) async fn apply_state_sync(
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

        // Handle mismatched private accounts before the atomic update
        for (account_id, digest) in account_updates.mismatched_private_accounts() {
            self.lock_account_on_unexpected_commitment(account_id, digest).await.map_err(
                |err| {
                    StoreError::DatabaseError(format!("failed to check account mismatch: {err:?}"))
                },
            )?;
        }

        // Undo account states from discarded transactions before the atomic update
        let account_states_to_rollback: Vec<Word> = transaction_updates
            .discarded_transactions()
            .map(|tx_record| tx_record.details.final_account_state)
            .collect();

        self.undo_account_states(&account_states_to_rollback).await?;

        // Serialize block headers
        let mut block_headers = Vec::with_capacity(block_updates.block_headers().len());
        for (header, has_client_notes, peaks) in block_updates.block_headers() {
            block_headers.push(JsBlockHeader {
                block_num: header.block_num().as_u32(),
                header: header.to_bytes(),
                peaks: peaks.peaks().to_vec().to_bytes(),
                has_relevant_notes: *has_client_notes,
            });
        }
        let block_headers_js =
            serde_wasm_bindgen::to_value(&block_headers).expect("serialization should succeed");

        // Serialize authentication nodes
        let mut node_ids = Vec::new();
        let mut node_values = Vec::new();
        for (id, node) in block_updates.new_authentication_nodes() {
            node_ids.push(id.inner().to_string());
            let node_bytes = node.to_bytes();
            let js_array = js_sys::Uint8Array::from(node_bytes.as_slice());
            node_values.push(js_array.into());
        }

        // Serialize input notes
        let input_notes: Vec<JsInputNote> = note_updates
            .updated_input_notes()
            .map(|note_update| {
                let note = note_update.inner();
                let details = note.details();
                let recipient = details.recipient();
                JsInputNote {
                    note_id: note.id().to_hex(),
                    assets: note.assets().to_bytes(),
                    serial_number: details.serial_num().to_bytes(),
                    inputs: details.inputs().to_bytes(),
                    script_root: recipient.script().root().to_hex(),
                    serialized_note_script: recipient.script().to_bytes(),
                    nullifier: details.nullifier().to_hex(),
                    created_at: crate::current_timestamp_u64().to_string(),
                    state_discriminant: note.state().discriminant(),
                    state: note.state().to_bytes(),
                }
            })
            .collect();
        let input_notes_js =
            serde_wasm_bindgen::to_value(&input_notes).expect("serialization should succeed");

        // Serialize output notes
        let output_notes: Vec<JsOutputNote> = note_updates
            .updated_output_notes()
            .map(|note_update| {
                let note = note_update.inner();
                JsOutputNote {
                    note_id: note.id().to_hex(),
                    assets: note.assets().to_bytes(),
                    recipient_digest: note.recipient_digest().to_hex(),
                    metadata: note.metadata().to_bytes(),
                    nullifier: note.nullifier().map(|n| n.to_hex()),
                    expected_height: note.expected_height().as_u32(),
                    state_discriminant: note.state().discriminant(),
                    state: note.state().to_bytes(),
                }
            })
            .collect();
        let output_notes_js =
            serde_wasm_bindgen::to_value(&output_notes).expect("serialization should succeed");

        // Serialize transaction updates
        let tx_updates: Vec<JsTransactionUpdate> = transaction_updates
            .committed_transactions()
            .chain(transaction_updates.discarded_transactions())
            .map(|tx_record| JsTransactionUpdate {
                id: tx_record.id.to_hex(),
                details: tx_record.details.to_bytes(),
                block_num: tx_record.details.block_num.as_u32(),
                status_variant: tx_record.status.variant() as u8,
                status: tx_record.status.to_bytes(),
                script_root: tx_record.script.as_ref().map(|script| script.root().to_bytes()),
            })
            .collect();
        let tx_updates_js =
            serde_wasm_bindgen::to_value(&tx_updates).expect("serialization should succeed");

        // Serialize account updates
        let acct_updates: Vec<JsAccountUpdate> = account_updates
            .updated_public_accounts()
            .iter()
            .map(JsAccountUpdate::from_account)
            .collect();
        let acct_updates_js =
            serde_wasm_bindgen::to_value(&acct_updates).expect("serialization should succeed");

        // Serialize tags to remove (committed input notes)
        let tags_to_remove: Vec<JsTagToRemove> = note_updates
            .updated_input_notes()
            .filter_map(|note_update| {
                let note = note_update.inner();
                if note.is_committed() {
                    let tag = note.metadata().expect("Committed notes should have metadata").tag();
                    let tag_record = NoteTagRecord {
                        tag,
                        source: NoteTagSource::Note(note.id()),
                    };
                    Some(JsTagToRemove {
                        tag: tag_record.tag.to_bytes(),
                        source: tag_record.source.to_bytes(),
                    })
                } else {
                    None
                }
            })
            .collect();
        let tags_to_remove_js =
            serde_wasm_bindgen::to_value(&tags_to_remove).expect("serialization should succeed");

        // Account states to undo (already done above, pass empty since it's already handled)
        let account_states_to_undo: Vec<String> = Vec::new();

        // Accounts to lock (already handled above, pass empty)
        let accounts_to_lock: Vec<JsAccountToLock> = Vec::new();
        let accounts_to_lock_js =
            serde_wasm_bindgen::to_value(&accounts_to_lock).expect("serialization should succeed");

        js_apply_state_sync(
            self.db_id(),
            block_num.as_u32(),
            block_headers_js,
            node_ids,
            node_values,
            input_notes_js,
            output_notes_js,
            tx_updates_js,
            acct_updates_js,
            tags_to_remove_js,
            account_states_to_undo,
            accounts_to_lock_js,
        );

        Ok(())
    }
}

// Serialization structs for passing data to JS
// ================================================================================================

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsBlockHeader {
    block_num: u32,
    header: Vec<u8>,
    peaks: Vec<u8>,
    has_relevant_notes: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsInputNote {
    note_id: String,
    assets: Vec<u8>,
    serial_number: Vec<u8>,
    inputs: Vec<u8>,
    script_root: String,
    serialized_note_script: Vec<u8>,
    nullifier: String,
    created_at: String,
    state_discriminant: u8,
    state: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsOutputNote {
    note_id: String,
    assets: Vec<u8>,
    recipient_digest: String,
    metadata: Vec<u8>,
    nullifier: Option<String>,
    expected_height: u32,
    state_discriminant: u8,
    state: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsTransactionUpdate {
    id: String,
    details: Vec<u8>,
    block_num: u32,
    status_variant: u8,
    status: Vec<u8>,
    script_root: Option<Vec<u8>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsAccountUpdate {
    account_id: String,
    code_commitment: String,
    code: Vec<u8>,
    storage_commitment: String,
    storage_slots: Vec<JsStorageSlot>,
    storage_map_entries: Vec<JsStorageMapEntry>,
    vault_root: String,
    assets: Vec<JsVaultAsset>,
    nonce: String,
    committed: bool,
    account_commitment: String,
    account_seed: Option<Vec<u8>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsStorageSlot {
    commitment: String,
    slot_name: String,
    slot_value: String,
    slot_type: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsStorageMapEntry {
    root: String,
    key: String,
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsVaultAsset {
    root: String,
    vault_key: String,
    faucet_id_prefix: String,
    asset: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsTagToRemove {
    tag: Vec<u8>,
    source: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsAccountToLock {
    account_id: String,
    mismatched_digest: String,
}

impl JsAccountUpdate {
    fn from_account(account: &Account) -> Self {
        let asset_vault = account.vault();
        Self {
            account_id: account.id().to_string(),
            code_commitment: account.code().commitment().to_string(),
            code: account.code().to_bytes(),
            storage_commitment: account.storage().to_commitment().to_string(),
            storage_slots: account
                .storage()
                .slots()
                .iter()
                .map(|slot| JsStorageSlot {
                    commitment: account.storage().to_commitment().to_hex(),
                    slot_name: slot.name().to_string(),
                    slot_value: slot.value().to_hex(),
                    slot_type: slot.slot_type().to_bytes()[0],
                })
                .collect(),
            storage_map_entries: account
                .storage()
                .slots()
                .iter()
                .filter_map(|slot| {
                    if let StorageSlotContent::Map(map) = slot.content() {
                        Some(
                            map.entries()
                                .map(|(key, value)| JsStorageMapEntry {
                                    root: map.root().to_hex(),
                                    key: key.to_hex(),
                                    value: value.to_hex(),
                                })
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        None
                    }
                })
                .flatten()
                .collect(),
            vault_root: asset_vault.root().to_string(),
            assets: asset_vault
                .assets()
                .map(|asset| JsVaultAsset {
                    root: asset_vault.root().to_hex(),
                    vault_key: Word::from(asset.vault_key()).to_hex(),
                    faucet_id_prefix: asset.faucet_id_prefix().to_hex(),
                    asset: Word::from(asset).to_hex(),
                })
                .collect(),
            nonce: account.nonce().to_string(),
            committed: account.is_public(),
            account_commitment: account.commitment().to_string(),
            account_seed: account.seed().map(|seed| seed.to_bytes()),
        }
    }
}
