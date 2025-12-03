use miden_client::transaction::{InputNote as NativeInputNote, InputNotes as NativeInputNotes};
use wasm_bindgen::prelude::*;

use super::input_note::InputNote;
use super::word::Word;

/// Collection of input notes consumed by a transaction.
#[derive(Clone)]
#[wasm_bindgen]
pub struct InputNotes(NativeInputNotes<NativeInputNote>);

#[wasm_bindgen]
impl InputNotes {
    /// Returns the commitment to all input notes.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns the number of input notes.
    #[wasm_bindgen(js_name = "numNotes")]
    pub fn num_notes(&self) -> u8 {
        u8::try_from(self.0.num_notes()).expect("only 256 input notes is allowed")
    }

    /// Returns true if there are no input notes.
    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the input note at the specified index.
    #[wasm_bindgen(js_name = "getNote")]
    pub fn get_note(&self, index: u8) -> InputNote {
        self.0.get_note(index as usize).into()
    }

    /// Returns all input notes as a vector.
    pub fn notes(&self) -> Vec<InputNote> {
        self.0.iter().cloned().map(Into::into).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeInputNotes<NativeInputNote>> for InputNotes {
    fn from(native_notes: NativeInputNotes<NativeInputNote>) -> Self {
        InputNotes(native_notes)
    }
}

impl From<&NativeInputNotes<NativeInputNote>> for InputNotes {
    fn from(native_notes: &NativeInputNotes<NativeInputNote>) -> Self {
        InputNotes(native_notes.clone())
    }
}
