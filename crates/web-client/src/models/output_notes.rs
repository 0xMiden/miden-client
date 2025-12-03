use miden_client::transaction::OutputNotes as NativeOutputNotes;
use wasm_bindgen::prelude::*;

use super::output_note::OutputNote;
use super::word::Word;

/// Collection of notes created by a transaction.
#[derive(Clone)]
#[wasm_bindgen]
pub struct OutputNotes(pub(crate) NativeOutputNotes);

#[wasm_bindgen]
impl OutputNotes {
    /// Returns the commitment to all output notes.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns the number of notes emitted.
    #[wasm_bindgen(js_name = "numNotes")]
    pub fn num_notes(&self) -> u32 {
        u32::try_from(self.0.num_notes())
            .expect("only 1024 output notes is allowed per transaction")
    }

    /// Returns true if there are no output notes.
    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the output note at the specified index.
    #[wasm_bindgen(js_name = "getNote")]
    pub fn get_note(&self, index: u32) -> OutputNote {
        self.0.get_note(index as usize).into()
    }

    /// Returns all output notes as a vector.
    pub fn notes(&self) -> Vec<OutputNote> {
        self.0.iter().cloned().map(Into::into).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeOutputNotes> for OutputNotes {
    fn from(native_notes: NativeOutputNotes) -> Self {
        OutputNotes(native_notes)
    }
}

impl From<&NativeOutputNotes> for OutputNotes {
    fn from(native_notes: &NativeOutputNotes) -> Self {
        OutputNotes(native_notes.clone())
    }
}
