use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;

// Transactions IndexedDB Operations
#[wasm_bindgen(module = "/src/js/transactions.js")]

extern "C" {
    // GETS
    // ================================================================================================

    #[wasm_bindgen(js_name = getTransactions)]
    pub fn idxdb_get_transactions(db_id: &str, filter: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = insertTransactionScript)]
    pub fn idxdb_insert_transaction_script(
        db_id: &str,
        script_root: Vec<u8>,
        tx_script: Option<Vec<u8>>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertTransactionRecord)]
    pub fn idxdb_upsert_transaction_record(
        db_id: &str,
        transaction_id: String,
        details: Vec<u8>,
        block_num: u32,
        statusVariant: u8,
        status: Vec<u8>,
        scriptRoot: Option<Vec<u8>>,
    ) -> js_sys::Promise;
}
