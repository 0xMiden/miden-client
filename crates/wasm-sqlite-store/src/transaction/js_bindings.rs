use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;

// Transaction SQLite Operations
#[wasm_bindgen(module = "/src/js/transactions.js")]
extern "C" {
    #[wasm_bindgen(js_name = getTransactions)]
    pub fn js_get_transactions(db_id: &str, filter: String) -> JsValue;

    #[wasm_bindgen(js_name = insertTransactionScript)]
    pub fn js_insert_transaction_script(
        db_id: &str,
        script_root: Vec<u8>,
        tx_script: Option<Vec<u8>>,
    );

    #[wasm_bindgen(js_name = upsertTransactionRecord)]
    pub fn js_upsert_transaction_record(
        db_id: &str,
        transaction_id: String,
        details: Vec<u8>,
        block_num: u32,
        status_variant: u8,
        status: Vec<u8>,
        script_root: Option<Vec<u8>>,
    );
}
