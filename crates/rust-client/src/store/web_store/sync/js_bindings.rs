use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use miden_objects::{account::Account, asset::Asset};
use miden_tx::utils::Serializable;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, wasm_bindgen};

use crate::{
    store::web_store::{
        account::utils::insert_account_storage,
        note::utils::{SerializedInputNoteData, SerializedOutputNoteData},
        transaction::utils::SerializedTransactionData,
    },
    sync::StateSyncUpdate,
};

use super::flattened_vec::FlattenedU8Vec;

// Sync IndexedDB Operations
#[wasm_bindgen(module = "/src/store/web_store/js/sync.js")]

extern "C" {
    // GETS
    // ================================================================================================

    #[wasm_bindgen(js_name = getSyncHeight)]
    pub fn idxdb_get_sync_height() -> js_sys::Promise;

    #[wasm_bindgen(js_name = getNoteTags)]
    pub fn idxdb_get_note_tags() -> js_sys::Promise;

    // INSERTS
    // ================================================================================================

    #[wasm_bindgen(js_name = addNoteTag)]
    pub fn idxdb_add_note_tag(
        tag: Vec<u8>,
        source_note_id: Option<String>,
        source_account_id: Option<String>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = applyStateSync)]
    pub fn idxdb_apply_state_sync(state_update: JsStateSyncUpdate) -> js_sys::Promise;

    // DELETES
    // ================================================================================================
    #[wasm_bindgen(js_name = removeNoteTag)]
    pub fn idxdb_remove_note_tag(
        tag: Vec<u8>,
        source_note_id: Option<String>,
        source_account_id: Option<String>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = discardTransactions)]
    pub fn idxdb_discard_transactions(transactions: Vec<String>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = receiveStateSync)]
    pub fn idxdb_receive_state_sync(state_update: JsStateSyncUpdate) -> js_sys::Promise;
}

#[wasm_bindgen(getter_with_clone)]
// FIXME: Add docstrings for fields
pub struct JsStateSyncUpdate {
    pub block_num: String,
    pub flattened_new_block_headers: FlattenedU8Vec,
    pub new_block_nums: Vec<String>,
    pub flattened_partial_blockchain_peaks: FlattenedU8Vec,
    pub block_has_relevant_notes: Vec<u8>,
    pub serialized_node_ids: Vec<String>,
    pub serialized_nodes: Vec<String>,
    pub note_tags_to_remove: Vec<String>,
    pub serialized_input_notes: Vec<SerializedInputNoteData>,
    pub serialized_output_notes: Vec<SerializedOutputNoteData>,
    pub account_updates: Vec<JsAccountUpdate>,
    pub transaction_updates: Vec<SerializedTransactionData>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct JsAccountUpdate {
    // An account storage's new root, aka commitment.
    storage_root: String,
    storage_slots: Vec<u8>,
    // The asset's vault new root, aka commitment.
    asset_vault_root: String,
    // This account's assets, as bytes
    asset_bytes: Vec<u8>,
    account_id: String,
    code_root: String,
    commited: bool,
    nonce: String,
    account_commitment: String,
}

impl JsAccountUpdate {
    pub fn from_account(account: &Account) -> Self {
        let asset_vault = account.vault();
        Self {
            storage_root: account.storage().commitment().to_string(),
            storage_slots: account.storage().to_bytes(),
            asset_vault_root: asset_vault.root().to_string(),
            asset_bytes: asset_vault.assets().collect::<Vec<_>>().to_bytes(),
            account_id: account.id().to_string(),
            code_root: account.code().commitment().to_string(),
            commited: account.is_public(),
            nonce: account.nonce().to_string(),
            account_commitment: account.commitment().to_string(),
        }
    }
}
