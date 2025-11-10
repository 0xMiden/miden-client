#[wasm_bindgen(
    module = "/src/js/transactions.js",
    inline_js = __INLINE_JS__
)]
extern "C" {
    // GETS
    // ================================================================================================

    #[wasm_bindgen(js_name = getTransactions)]
    pub fn idxdb_get_transactions(filter: String) -> js_sys::Promise;

    // INSERTS
    // ================================================================================================

    #[wasm_bindgen(js_name = insertTransactionScript)]
    pub fn idxdb_insert_transaction_script(
        script_root: Vec<u8>,
        tx_script: Option<Vec<u8>>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertTransactionRecord)]
    pub fn idxdb_upsert_transaction_record(
        transaction_id: String,
        details: Vec<u8>,
        block_num: String,
        statusVariant: u8,
        status: Vec<u8>,
        scriptRoot: Option<Vec<u8>>,
    ) -> js_sys::Promise;
}
