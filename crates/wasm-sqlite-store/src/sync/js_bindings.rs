use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;

// Sync SQLite Operations
#[wasm_bindgen(module = "/src/js/sync.js")]
extern "C" {
    // GETS
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = getNoteTags)]
    pub fn js_get_note_tags(db_id: &str) -> JsValue;

    #[wasm_bindgen(js_name = getSyncHeight)]
    pub fn js_get_sync_height(db_id: &str) -> JsValue;

    // INSERTS
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = addNoteTag)]
    pub fn js_add_note_tag(db_id: &str, tag: Vec<u8>, source: Vec<u8>) -> bool;

    // UPDATES
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = applyStateSync)]
    pub fn js_apply_state_sync(
        db_id: &str,
        block_num: u32,
        block_headers: JsValue,
        node_ids: Vec<String>,
        node_values: Vec<JsValue>,
        input_notes: JsValue,
        output_notes: JsValue,
        transaction_updates: JsValue,
        account_updates: JsValue,
        tags_to_remove: JsValue,
        account_states_to_undo: Vec<String>,
        accounts_to_lock: JsValue,
    );

    // DELETES
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = removeNoteTag)]
    pub fn js_remove_note_tag(db_id: &str, tag: Vec<u8>, source: Vec<u8>) -> u32;

    #[wasm_bindgen(js_name = discardTransactions)]
    pub fn js_discard_transactions(db_id: &str, transaction_ids: Vec<String>);
}
