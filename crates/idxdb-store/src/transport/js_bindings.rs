use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;

// Transport IndexedDB Operations
#[wasm_bindgen(module = "/src/js/transport.js")]
extern "C" {
    #[wasm_bindgen(js_name = getNoteTransportCursor)]
    pub fn idxdb_get_note_transport_cursor() -> js_sys::Promise;

    #[wasm_bindgen(js_name = updateNoteTransportCursor)]
    pub fn idxdb_update_note_transport_cursor(cursor: u64) -> js_sys::Promise;
}
