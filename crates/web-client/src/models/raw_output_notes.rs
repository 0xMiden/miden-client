use miden_client::transaction::RawOutputNotes as NativeRawOutputNotes;
use wasm_bindgen::prelude::*;

use super::raw_output_note::RawOutputNote;
use super::word::Word;

/// Contains a list of output notes of a transaction. The list can be empty if the transaction does
/// not produce any notes.
#[derive(Clone)]
#[wasm_bindgen]
pub struct RawOutputNotes(pub(crate) NativeRawOutputNotes);

#[wasm_bindgen]
impl RawOutputNotes {
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
    pub fn get_note(&self, index: u32) -> RawOutputNote {
        self.0.get_note(index as usize).into()
    }

    /// Returns all output notes as a vector.
    pub fn notes(&self) -> Vec<RawOutputNote> {
        self.0.iter().cloned().map(Into::into).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeRawOutputNotes> for RawOutputNotes {
    fn from(native_notes: NativeRawOutputNotes) -> Self {
        RawOutputNotes(native_notes)
    }
}

impl From<&NativeRawOutputNotes> for RawOutputNotes {
    fn from(native_notes: &NativeRawOutputNotes) -> Self {
        RawOutputNotes(native_notes.clone())
    }
}
