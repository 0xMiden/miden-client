use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::account::{
    Account,
    AccountId,
    StorageMap,
    StorageSlotContent,
    StorageSlotName,
    StorageSlotType,
};
use miden_client::asset::{Asset, AssetVaultKey};
use miden_client::note::ToInputNoteCommitments;
use miden_client::store::{StoreError, TransactionFilter};
use miden_client::sync::NoteTagSource;
use miden_client::transaction::{
    TransactionDetails,
    TransactionId,
    TransactionRecord,
    TransactionScript,
    TransactionStatus,
    TransactionStoreUpdate,
};
use miden_client::utils::{Deserializable, Serializable};
use miden_client::{EMPTY_WORD, Word};
use serde::Serialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::js_sys;

use super::IdxdbStore;
use super::account::utils::{
    apply_full_account_state,
    apply_transaction_delta,
    compute_storage_delta,
    compute_vault_delta,
};
use super::account::{JsStorageMapEntry, JsStorageSlot, JsVaultAsset};
use super::note::utils::{
    SerializedInputNoteData,
    SerializedOutputNoteData,
    apply_note_updates_tx,
    serialize_input_note,
    serialize_output_note,
};
use crate::promise::await_js;

mod js_bindings;
use js_bindings::{idxdb_apply_transaction_batch, idxdb_get_transactions};

mod models;
use models::TransactionIdxdbObject;

pub mod utils;
use utils::{
    SerializedTransactionData,
    insert_proven_transaction_data,
    serialize_transaction_record,
};

// BATCH PAYLOAD TYPES
// ================================================================================================

/// Serializable representation of the TS `JsFullAccountState` interface.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchFullAccountState {
    account_id: String,
    nonce: String,
    storage_slots: Vec<JsStorageSlot>,
    storage_map_entries: Vec<JsStorageMapEntry>,
    assets: Vec<JsVaultAsset>,
    code_root: String,
    storage_root: String,
    vault_root: String,
    committed: bool,
    account_commitment: String,
    #[serde(with = "serde_bytes", skip_serializing_if = "Option::is_none")]
    account_seed: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum BatchAccountState {
    Full {
        account: BatchFullAccountState,
    },
    Delta {
        #[serde(rename = "accountId")]
        account_id: String,
        nonce: String,
        #[serde(rename = "updatedSlots")]
        updated_slots: Vec<JsStorageSlot>,
        #[serde(rename = "changedMapEntries")]
        changed_map_entries: Vec<JsStorageMapEntry>,
        #[serde(rename = "changedAssets")]
        changed_assets: Vec<JsVaultAsset>,
        #[serde(rename = "codeRoot")]
        code_root: String,
        #[serde(rename = "storageRoot")]
        storage_root: String,
        #[serde(rename = "vaultRoot")]
        vault_root: String,
        committed: bool,
        commitment: String,
    },
}

/// Serializable representation of the TS `JsBatchNoteTag` interface.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchNoteTag {
    #[serde(with = "serde_bytes")]
    tag: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_note_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_account_id: Option<String>,
}

/// Serializable representation of the TS `JsBatchUpdatePayload` interface.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchUpdatePayload {
    transaction_record: SerializedTransactionData,
    account_state: BatchAccountState,
    input_notes: Vec<SerializedInputNoteData>,
    output_notes: Vec<SerializedOutputNoteData>,
    tags: Vec<BatchNoteTag>,
}

impl IdxdbStore {
    pub async fn get_transactions(
        &self,
        filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, StoreError> {
        let filter_as_str = match filter {
            TransactionFilter::All => "All",
            TransactionFilter::Uncommitted => "Uncommitted",
            TransactionFilter::Ids(ids) => &{
                let ids_str =
                    ids.iter().map(ToString::to_string).collect::<Vec<String>>().join(",");
                format!("Ids:{ids_str}")
            },
            TransactionFilter::ExpiredBefore(block_number) => {
                &format!("ExpiredPending:{block_number}")
            },
        };

        let promise = idxdb_get_transactions(self.db_id(), filter_as_str.to_string());
        let transactions_idxdb: Vec<TransactionIdxdbObject> =
            await_js(promise, "failed to get transactions").await?;

        let transaction_records: Result<Vec<TransactionRecord>, StoreError> = transactions_idxdb
            .into_iter()
            .map(|tx_idxdb| {
                let id: Word = tx_idxdb.id.try_into()?;

                let details = TransactionDetails::read_from_bytes(&tx_idxdb.details)?;

                let script: Option<TransactionScript> = if tx_idxdb.script_root.is_some() {
                    let tx_script = tx_idxdb
                        .tx_script
                        .map(|script| TransactionScript::read_from_bytes(&script))
                        .transpose()?
                        .expect("Transaction script should be included in the row");

                    Some(tx_script)
                } else {
                    None
                };

                let status = TransactionStatus::read_from_bytes(&tx_idxdb.status)?;

                Ok(TransactionRecord {
                    id: TransactionId::from_raw(id),
                    details,
                    script,
                    status,
                })
            })
            .collect();

        transaction_records
    }

    pub async fn apply_transaction(
        &self,
        tx_update: TransactionStoreUpdate,
    ) -> Result<(), StoreError> {
        let executed_tx = tx_update.executed_transaction();

        // Transaction Data
        insert_proven_transaction_data(self.db_id(), executed_tx, tx_update.submission_height())
            .await?;

        let delta = executed_tx.account_delta();
        let account_id = executed_tx.account_id();

        if delta.is_full_state() {
            // Full-state path: the delta contains the complete account state.
            let account: Account =
                delta.try_into().expect("casting account from full state delta should not fail");
            apply_full_account_state(self.db_id(), &account).await.map_err(|err| {
                StoreError::DatabaseError(format!("failed to apply full account state: {err:?}"))
            })?;

            let mut smt_forest = self.smt_forest.write();
            smt_forest.insert_and_stage_account_state(
                account.id(),
                account.vault(),
                account.storage(),
            )?;
        } else {
            // Delta path: load only targeted data, avoid loading full Account.
            let vault_keys: Vec<String> = delta
                .vault()
                .fungible()
                .iter()
                .map(|(vault_key, _)| vault_key.to_string())
                .collect();
            let old_vault_assets = self.get_vault_assets(account_id, vault_keys).await?;
            let map_slot_names: Vec<String> =
                delta.storage().maps().map(|(slot_name, _)| slot_name.to_string()).collect();
            let old_map_roots = self.get_storage_map_roots(account_id, map_slot_names).await?;

            let final_header = executed_tx.final_account();

            // Compute storage and vault changes using SMT forest, then stage new roots.
            let (updated_storage_slots, updated_assets, removed_vault_keys) = {
                let mut smt_forest = self.smt_forest.write();

                // Get current tracked roots to build the final roots from
                let mut final_roots = smt_forest
                    .get_roots(&account_id)
                    .cloned()
                    .ok_or(StoreError::AccountDataNotFound(account_id))?;

                // Storage: compute new map roots via SMT forest
                let updated_storage_slots =
                    compute_storage_delta(&mut smt_forest, &old_map_roots, delta)?;

                // Update map roots in final_roots with new values from the delta
                let default_map_root = StorageMap::default().root();
                for (slot_name, (new_root, slot_type)) in &updated_storage_slots {
                    if *slot_type == StorageSlotType::Map {
                        let old_root =
                            old_map_roots.get(slot_name).copied().unwrap_or(default_map_root);
                        if let Some(root) = final_roots.iter_mut().find(|r| **r == old_root) {
                            *root = *new_root;
                        } else {
                            // New map slot not in the old roots — append it
                            final_roots.push(*new_root);
                        }
                    }
                }

                // Vault: compute new asset values and update SMT forest
                let old_vault_root = final_roots[0];
                let (updated_assets, removed_vault_keys) =
                    compute_vault_delta(&old_vault_assets, delta)?;
                let new_vault_root = smt_forest.update_asset_nodes(
                    old_vault_root,
                    updated_assets.iter().copied(),
                    removed_vault_keys.iter().copied(),
                )?;

                if new_vault_root != final_header.vault_root() {
                    return Err(StoreError::DatabaseError(format!(
                        "computed vault root {} does not match final account header {}",
                        new_vault_root.to_hex(),
                        final_header.vault_root().to_hex(),
                    )));
                }

                // Update vault root in final_roots (first element is always vault root)
                final_roots[0] = new_vault_root;

                // Stage the new roots for later commit/discard during sync
                smt_forest.stage_roots(account_id, final_roots);

                (updated_storage_slots, updated_assets, removed_vault_keys)
            };

            apply_transaction_delta(
                self.db_id(),
                account_id,
                final_header,
                &updated_storage_slots,
                &updated_assets,
                &removed_vault_keys,
                delta,
            )
            .await
            .map_err(|err| {
                StoreError::DatabaseError(format!("failed to apply transaction delta: {err:?}"))
            })?;
        }

        // Updates for notes
        apply_note_updates_tx(self.db_id(), tx_update.note_updates()).await?;

        for tag_record in tx_update.new_tags() {
            self.add_note_tag(*tag_record).await?;
        }

        Ok(())
    }

    /// Applies multiple transaction updates atomically in a single Dexie transaction.
    pub async fn apply_transaction_batch_atomic(
        &self,
        tx_updates: Vec<TransactionStoreUpdate>,
    ) -> Result<(), StoreError> {
        if tx_updates.is_empty() {
            return Ok(());
        }

        let mut payloads: Vec<BatchUpdatePayload> = Vec::with_capacity(tx_updates.len());
        let mut staged_accounts: Vec<AccountId> = Vec::with_capacity(tx_updates.len());

        // Simulates read-writes across batch transactions, since nothing is persisted to
        // IndexedDB until the single Dexie transaction at the end.
        let mut vault_overlay: BTreeMap<AssetVaultKey, Option<Asset>> = BTreeMap::new();
        let mut map_roots_overlay: BTreeMap<StorageSlotName, Word> = BTreeMap::new();

        for update in &tx_updates {
            let (payload, account_id) = self
                .prepare_update_for_batch(update, &mut vault_overlay, &mut map_roots_overlay)
                .await?;
            payloads.push(payload);
            staged_accounts.push(account_id);
        }

        // Serialize all payloads to a JS array of plain objects via serde_wasm_bindgen.
        // The default Serializer (serialize_bytes_as_arrays: false) produces Uint8Array for
        // Vec<u8> fields annotated with #[serde(with = "serde_bytes")].
        let serializer = serde_wasm_bindgen::Serializer::new();
        let js_array = js_sys::Array::new();
        for payload in &payloads {
            let js_value = payload
                .serialize(&serializer)
                .map_err(|e| StoreError::DatabaseError(format!("serialization error: {e}")))?;
            js_array.push(&js_value);
        }

        let promise = idxdb_apply_transaction_batch(self.db_id(), JsValue::from(js_array));
        let js_result = crate::promise::await_ok(promise, "batch apply").await;

        // Commit or discard staged SMT forest state
        match js_result {
            Ok(()) => {
                let mut smt_forest = self.smt_forest.write();
                for acc_id in staged_accounts {
                    smt_forest.commit_roots(acc_id);
                }
                Ok(())
            },
            Err(err) => {
                let mut smt_forest = self.smt_forest.write();
                for acc_id in staged_accounts {
                    smt_forest.discard_roots(acc_id);
                }
                Err(err)
            },
        }
    }

    /// Pre-computes all SMT work for a single update and builds the serializable payload
    /// for the batch JS call. Mirrors `apply_transaction` but without the JS writes.
    #[allow(clippy::too_many_lines)]
    async fn prepare_update_for_batch(
        &self,
        update: &TransactionStoreUpdate,
        vault_overlay: &mut BTreeMap<AssetVaultKey, Option<Asset>>,
        map_roots_overlay: &mut BTreeMap<StorageSlotName, Word>,
    ) -> Result<(BatchUpdatePayload, AccountId), StoreError> {
        let executed_tx = update.executed_transaction();
        let delta = executed_tx.account_delta();
        let account_id = executed_tx.account_id();

        // Build transaction record payload via the shared serializer.
        let nullifiers: Vec<Word> =
            executed_tx.input_notes().iter().map(|x| x.nullifier().as_word()).collect();
        let details = TransactionDetails {
            account_id,
            init_account_state: executed_tx.initial_account().initial_commitment(),
            final_account_state: executed_tx.final_account().to_commitment(),
            input_note_nullifiers: nullifiers,
            output_notes: executed_tx.output_notes().clone(),
            block_num: executed_tx.block_header().block_num(),
            submission_height: update.submission_height(),
            expiration_block_num: executed_tx.expiration_block_num(),
            creation_timestamp: crate::current_timestamp_u64(),
        };
        let record = TransactionRecord::new(
            executed_tx.id(),
            details,
            executed_tx.tx_args().tx_script().cloned(),
            TransactionStatus::Pending,
        );
        let transaction_record = serialize_transaction_record(&record);

        // Build account state payload and stage SMT roots
        let account_state = if delta.is_full_state() {
            let account: Account =
                delta.try_into().expect("casting account from full state delta should not fail");

            {
                let mut smt_forest = self.smt_forest.write();
                smt_forest.insert_and_stage_account_state(
                    account.id(),
                    account.vault(),
                    account.storage(),
                )?;
            }

            // Seed overlays with the new full account state so subsequent non-full-state
            // preparations in the same batch see post-this-tx values.
            vault_overlay.clear();
            for asset in account.vault().assets() {
                vault_overlay.insert(asset.vault_key(), Some(asset));
            }
            map_roots_overlay.clear();
            for slot in account.storage().slots() {
                if let StorageSlotContent::Map(map) = slot.content() {
                    map_roots_overlay.insert(slot.name().clone(), map.root());
                }
            }

            let storage_slots: Vec<JsStorageSlot> =
                account.storage().slots().iter().map(JsStorageSlot::from_slot).collect();

            let storage_map_entries: Vec<JsStorageMapEntry> = account
                .storage()
                .slots()
                .iter()
                .filter_map(|slot| match slot.content() {
                    StorageSlotContent::Map(map) => {
                        Some(JsStorageMapEntry::from_map(map, &slot.name().to_string()))
                    },
                    StorageSlotContent::Value(_) => None,
                })
                .flatten()
                .collect();

            let assets: Vec<JsVaultAsset> =
                account.vault().assets().map(|a| JsVaultAsset::from_asset(&a)).collect();

            BatchAccountState::Full {
                account: BatchFullAccountState {
                    account_id: account.id().to_string(),
                    nonce: account.nonce().to_string(),
                    storage_slots,
                    storage_map_entries,
                    assets,
                    code_root: account.code().commitment().to_string(),
                    storage_root: account.storage().to_commitment().to_string(),
                    vault_root: account.vault().root().to_string(),
                    committed: account.is_public(),
                    account_commitment: account.to_commitment().to_string(),
                    account_seed: account.seed().map(|seed| seed.to_bytes()),
                },
            }
        } else {
            // Read each pre-tx vault key from `vault_overlay` first (which holds writes from
            // earlier txs in this batch), falling back to IndexedDB for keys this batch hasn't
            // touched yet.
            let mut old_vault_assets: Vec<Asset> = Vec::new();
            let mut vault_keys_to_fetch: Vec<String> = Vec::new();
            for (vault_key, _) in delta.vault().fungible().iter() {
                match vault_overlay.get(vault_key) {
                    Some(Some(asset)) => old_vault_assets.push(*asset),
                    Some(None) => { /* key was removed earlier in the batch; treat as empty */ },
                    None => vault_keys_to_fetch.push(vault_key.to_string()),
                }
            }
            if !vault_keys_to_fetch.is_empty() {
                let db_assets = self.get_vault_assets(account_id, vault_keys_to_fetch).await?;
                old_vault_assets.extend(db_assets);
            }

            let mut old_map_roots: BTreeMap<StorageSlotName, Word> = BTreeMap::new();
            let mut map_slot_names_to_fetch: Vec<String> = Vec::new();
            for (slot_name, _) in delta.storage().maps() {
                if let Some(root) = map_roots_overlay.get(slot_name) {
                    old_map_roots.insert(slot_name.clone(), *root);
                } else {
                    map_slot_names_to_fetch.push(slot_name.to_string());
                }
            }
            if !map_slot_names_to_fetch.is_empty() {
                let db_map_roots =
                    self.get_storage_map_roots(account_id, map_slot_names_to_fetch).await?;
                old_map_roots.extend(db_map_roots);
            }

            let final_header = executed_tx.final_account();

            let (js_slots, js_map_entries, js_assets) = {
                let mut smt_forest = self.smt_forest.write();

                let mut final_roots = smt_forest
                    .get_roots(&account_id)
                    .cloned()
                    .ok_or(StoreError::AccountDataNotFound(account_id))?;

                let updated_storage_slots =
                    compute_storage_delta(&mut smt_forest, &old_map_roots, delta)?;

                let default_map_root = StorageMap::default().root();
                for (slot_name, (new_root, slot_type)) in &updated_storage_slots {
                    if *slot_type == StorageSlotType::Map {
                        let old_root =
                            old_map_roots.get(slot_name).copied().unwrap_or(default_map_root);
                        if let Some(root) = final_roots.iter_mut().find(|r| **r == old_root) {
                            *root = *new_root;
                        } else {
                            final_roots.push(*new_root);
                        }
                    }
                }

                let old_vault_root = final_roots[0];
                let (updated_assets, removed_vault_keys) =
                    compute_vault_delta(&old_vault_assets, delta)?;
                let new_vault_root = smt_forest.update_asset_nodes(
                    old_vault_root,
                    updated_assets.iter().copied(),
                    removed_vault_keys.iter().copied(),
                )?;

                if new_vault_root != final_header.vault_root() {
                    return Err(StoreError::DatabaseError(format!(
                        "computed vault root {} does not match final account header {}",
                        new_vault_root.to_hex(),
                        final_header.vault_root().to_hex(),
                    )));
                }

                final_roots[0] = new_vault_root;
                smt_forest.stage_roots(account_id, final_roots);

                // Propagate this tx's post-state into the overlays so subsequent preparations
                // in the same batch see these updates.
                for asset in &updated_assets {
                    vault_overlay.insert(asset.vault_key(), Some(*asset));
                }
                for vault_key in &removed_vault_keys {
                    vault_overlay.insert(*vault_key, None);
                }
                for (slot_name, (new_root, slot_type)) in &updated_storage_slots {
                    if *slot_type == StorageSlotType::Map {
                        map_roots_overlay.insert(slot_name.clone(), *new_root);
                    }
                }

                let js_slots: Vec<JsStorageSlot> = updated_storage_slots
                    .iter()
                    .map(|(slot_name, (value, slot_type))| JsStorageSlot {
                        slot_name: slot_name.to_string(),
                        slot_value: value.to_hex(),
                        slot_type: *slot_type as u8,
                    })
                    .collect();

                let js_map_entries: Vec<JsStorageMapEntry> = delta
                    .storage()
                    .maps()
                    .flat_map(|(slot_name, map_delta)| {
                        map_delta.entries().iter().map(move |(key, value)| {
                            let value_str = if *value == EMPTY_WORD {
                                String::new()
                            } else {
                                value.to_hex()
                            };
                            JsStorageMapEntry {
                                slot_name: slot_name.to_string(),
                                key: Word::from(*key).to_hex(),
                                value: value_str,
                            }
                        })
                    })
                    .collect();

                let mut js_assets: Vec<JsVaultAsset> =
                    updated_assets.iter().map(JsVaultAsset::from_asset).collect();
                for vault_key in &removed_vault_keys {
                    js_assets.push(JsVaultAsset {
                        vault_key: vault_key.to_string(),
                        asset: String::new(),
                    });
                }

                (js_slots, js_map_entries, js_assets)
            };

            BatchAccountState::Delta {
                account_id: account_id.to_string(),
                nonce: final_header.nonce().to_string(),
                updated_slots: js_slots,
                changed_map_entries: js_map_entries,
                changed_assets: js_assets,
                code_root: final_header.code_commitment().to_string(),
                storage_root: final_header.storage_commitment().to_string(),
                vault_root: final_header.vault_root().to_string(),
                committed: account_id.is_public(),
                commitment: final_header.to_commitment().to_string(),
            }
        };

        let input_notes: Vec<SerializedInputNoteData> = update
            .note_updates()
            .updated_input_notes()
            .map(|u| serialize_input_note(u.inner()))
            .collect();

        let output_notes: Vec<SerializedOutputNoteData> = update
            .note_updates()
            .updated_output_notes()
            .map(|u| serialize_output_note(u.inner()))
            .collect();

        let tags: Vec<BatchNoteTag> = update
            .new_tags()
            .iter()
            .map(|tag_record| {
                let (source_note_id, source_account_id) = match &tag_record.source {
                    NoteTagSource::Note(note_id) => (Some(note_id.to_hex()), None),
                    NoteTagSource::Account(acc_id) => (None, Some(acc_id.to_hex())),
                    NoteTagSource::User => (None, None),
                };
                BatchNoteTag {
                    tag: tag_record.tag.to_bytes(),
                    source_note_id,
                    source_account_id,
                }
            })
            .collect();

        let payload = BatchUpdatePayload {
            transaction_record,
            account_state,
            input_notes,
            output_notes,
            tags,
        };

        Ok((payload, account_id))
    }
}
