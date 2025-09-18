use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;

// Transport IndexedDB Operations
#[wasm_bindgen(module = "/src/js/transport.js")]
extern "C" {
    #[wasm_bindgen(js_name = getTransportLayerCursor)]
    pub fn idxdb_get_transport_layer_cursor() -> js_sys::Promise;

    #[wasm_bindgen(js_name = updateTransportLayerCursor)]
    pub fn idxdb_update_transport_layer_cursor(cursor: u64) -> js_sys::Promise;
}
