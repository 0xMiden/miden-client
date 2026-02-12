use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;

// Chain data SQLite Operations
#[wasm_bindgen(module = "/src/js/chainData.js")]
extern "C" {
    #[wasm_bindgen(js_name = getBlockHeaders)]
    pub fn js_get_block_headers(db_id: &str, block_numbers: Vec<u32>) -> JsValue;

    #[wasm_bindgen(js_name = getTrackedBlockHeaders)]
    pub fn js_get_tracked_block_headers(db_id: &str) -> JsValue;

    #[wasm_bindgen(js_name = getPartialBlockchainNodesAll)]
    pub fn js_get_partial_blockchain_nodes_all(db_id: &str) -> JsValue;

    #[wasm_bindgen(js_name = getPartialBlockchainNodes)]
    pub fn js_get_partial_blockchain_nodes(db_id: &str, ids: Vec<String>) -> JsValue;

    #[wasm_bindgen(js_name = getPartialBlockchainNodesUpToInOrderIndex)]
    pub fn js_get_partial_blockchain_nodes_up_to_inorder_index(
        db_id: &str,
        max_in_order_index: String,
    ) -> JsValue;

    #[wasm_bindgen(js_name = getPartialBlockchainPeaksByBlockNum)]
    pub fn js_get_partial_blockchain_peaks_by_block_num(db_id: &str, block_num: u32) -> JsValue;

    #[wasm_bindgen(js_name = insertBlockHeader)]
    pub fn js_insert_block_header(
        db_id: &str,
        block_num: u32,
        header: Vec<u8>,
        partial_blockchain_peaks: Vec<u8>,
        has_client_notes: bool,
    );

    #[wasm_bindgen(js_name = insertPartialBlockchainNodes)]
    pub fn js_insert_partial_blockchain_nodes(db_id: &str, ids: Vec<String>, nodes: Vec<JsValue>);

    #[wasm_bindgen(js_name = pruneIrrelevantBlocks)]
    pub fn js_prune_irrelevant_blocks(db_id: &str);
}
