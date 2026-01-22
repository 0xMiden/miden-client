use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;

// ChainData IndexedDB Operations
#[wasm_bindgen(module = "/src/js/chainData.js")]
extern "C" {
    // GETS
    // ================================================================================================

    #[wasm_bindgen(js_name = getBlockHeaders)]
    pub fn idxdb_get_block_headers(db_id: &str, block_numbers: Vec<u32>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getTrackedBlockHeaders)]
    pub fn idxdb_get_tracked_block_headers(db_id: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getPartialBlockchainNodesAll)]
    pub fn idxdb_get_partial_blockchain_nodes_all(db_id: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getPartialBlockchainNodes)]
    pub fn idxdb_get_partial_blockchain_nodes(db_id: &str, ids: Vec<String>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getPartialBlockchainPeaksByBlockNum)]
    pub fn idxdb_get_partial_blockchain_peaks_by_block_num(
        db_id: &str,
        block_num: u32,
    ) -> js_sys::Promise;

    // INSERTS
    // ================================================================================================

    #[wasm_bindgen(js_name = insertBlockHeader)]
    pub fn idxdb_insert_block_header(
        db_id: &str,
        block_num: u32,
        header: Vec<u8>,
        partial_blockchain_peaks: Vec<u8>,
        has_client_notes: bool,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = insertPartialBlockchainNodes)]
    pub fn idxdb_insert_partial_blockchain_nodes(
        db_id: &str,
        ids: Vec<String>,
        nodes: Vec<String>,
    ) -> js_sys::Promise;

    // DELETES
    // ================================================================================================

    #[wasm_bindgen(js_name = pruneIrrelevantBlocks)]
    pub fn idxdb_prune_irrelevant_blocks(db_id: &str) -> js_sys::Promise;
}
