use miden_client::note::NoteExecutionMode as NativeNoteExecutionMode;
use wasm_bindgen::prelude::*;

/// Determines where a note should be executed (locally or on the network).
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteExecutionMode(NativeNoteExecutionMode);

#[wasm_bindgen]
impl NoteExecutionMode {
    #[wasm_bindgen(js_name = "newLocal")]
    /// Returns the local execution mode.
    pub fn new_local() -> NoteExecutionMode {
        NoteExecutionMode(NativeNoteExecutionMode::Local)
    }

    #[wasm_bindgen(js_name = "newNetwork")]
    /// Returns the network execution mode.
    pub fn new_network() -> NoteExecutionMode {
        NoteExecutionMode(NativeNoteExecutionMode::Network)
    }

    #[wasm_bindgen(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    /// Returns a human-readable representation of the execution mode.
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
