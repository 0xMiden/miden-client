use miden_client::note::NoteExecutionMode as NativeNoteExecutionMode;
use wasm_bindgen::prelude::*;

/// Specifies whether a note is executable locally or across the network.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteExecutionMode(NativeNoteExecutionMode);

#[wasm_bindgen]
impl NoteExecutionMode {
    /// Creates a note execution mode that targets the local account.
    #[wasm_bindgen(js_name = "newLocal")]
    pub fn new_local() -> NoteExecutionMode {
        NoteExecutionMode(NativeNoteExecutionMode::Local)
    }

    /// Creates a note execution mode that targets any network account.
    #[wasm_bindgen(js_name = "newNetwork")]
    pub fn new_network() -> NoteExecutionMode {
        NoteExecutionMode(NativeNoteExecutionMode::Network)
    }

    /// Returns a human-readable representation of the mode.
    #[wasm_bindgen(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        let note_execution_mode_as_str = match self.0 {
            NativeNoteExecutionMode::Local => "Local",
            NativeNoteExecutionMode::Network => "Network",
        };
        note_execution_mode_as_str.to_string()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteExecutionMode> for NoteExecutionMode {
    fn from(native_note_execution_mode: NativeNoteExecutionMode) -> Self {
        NoteExecutionMode(native_note_execution_mode)
    }
}

impl From<&NativeNoteExecutionMode> for NoteExecutionMode {
    fn from(native_note_execution_mode: &NativeNoteExecutionMode) -> Self {
        NoteExecutionMode(*native_note_execution_mode)
    }
}

impl From<NoteExecutionMode> for NativeNoteExecutionMode {
    fn from(note_execution_mode: NoteExecutionMode) -> Self {
        note_execution_mode.0
    }
}

impl From<&NoteExecutionMode> for NativeNoteExecutionMode {
    fn from(note_execution_mode: &NoteExecutionMode) -> Self {
        note_execution_mode.0
    }
}
