use wasm_bindgen::prelude::*;

/// Specifies whether a note is executable locally or across the network.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteExecutionMode(bool);

#[wasm_bindgen]
impl NoteExecutionMode {
    /// Creates a note execution mode that targets the local account.
    #[wasm_bindgen(js_name = "newLocal")]
    pub fn new_local() -> NoteExecutionMode {
        NoteExecutionMode(false)
    }

    /// Creates a note execution mode that targets any network account.
    #[wasm_bindgen(js_name = "newNetwork")]
    pub fn new_network() -> NoteExecutionMode {
        NoteExecutionMode(true)
    }

    /// Returns a human-readable representation of the mode.
    #[wasm_bindgen(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        if self.0 { "Network" } else { "Local" }.to_string()
    }
}

impl NoteExecutionMode {
    pub(crate) fn is_network(&self) -> bool {
        self.0
    }
}
