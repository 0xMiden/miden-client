use miden_client::note::NoteInputs as NativeNoteInputs;
use wasm_bindgen::prelude::*;

use super::felt::{Felt, FeltArray};

/// Inputs made available to a note script during execution.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteInputs(NativeNoteInputs);

#[wasm_bindgen]
impl NoteInputs {
    #[wasm_bindgen(constructor)]
    /// Creates a note input set from the provided field elements.
    pub fn new(felt_array: &FeltArray) -> NoteInputs {
        let native_felts = felt_array.into();
        let native_note_inputs = NativeNoteInputs::new(native_felts).unwrap();
        NoteInputs(native_note_inputs)
    }

    /// Returns the underlying field elements.
    pub fn values(&self) -> Vec<Felt> {
        self.0.values().iter().map(Into::into).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteInputs> for NoteInputs {
    fn from(native_note_inputs: NativeNoteInputs) -> Self {
        NoteInputs(native_note_inputs)
    }
}

impl From<&NativeNoteInputs> for NoteInputs {
    fn from(native_note_inputs: &NativeNoteInputs) -> Self {
        NoteInputs(native_note_inputs.clone())
    }
}

impl From<NoteInputs> for NativeNoteInputs {
    fn from(note_inputs: NoteInputs) -> Self {
        note_inputs.0
    }
}

impl From<&NoteInputs> for NativeNoteInputs {
    fn from(note_inputs: &NoteInputs) -> Self {
        note_inputs.0.clone()
    }
}
