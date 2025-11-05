use miden_client::Word as NativeWord;
use miden_client::note::{
    NoteInputs as NativeNoteInputs,
    NoteRecipient as NativeNoteRecipient,
    NoteScript as NativeNoteScript,
};
use wasm_bindgen::prelude::*;

use super::note_inputs::NoteInputs;
use super::note_script::NoteScript;
use super::word::Word;

/// Target recipient information for a note, including script and serial number.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteRecipient(NativeNoteRecipient);

#[wasm_bindgen]
impl NoteRecipient {
    #[wasm_bindgen(constructor)]
    /// Creates a new note recipient from serial number, script, and inputs.
    pub fn new(serial_num: &Word, note_script: &NoteScript, inputs: &NoteInputs) -> NoteRecipient {
        let native_serial_num: NativeWord = serial_num.into();
        let native_note_script: NativeNoteScript = note_script.into();
        let native_note_inputs: NativeNoteInputs = inputs.into();
        let native_note_recipient =
            NativeNoteRecipient::new(native_serial_num, native_note_script, native_note_inputs);

        NoteRecipient(native_note_recipient)
    }

    /// Returns the digest identifying this recipient.
    pub fn digest(&self) -> Word {
        self.0.digest().into()
    }

    #[wasm_bindgen(js_name = "serialNum")]
    /// Returns the recipient serial number word.
    pub fn serial_num(&self) -> Word {
        self.0.serial_num().into()
    }

    /// Returns the script associated with this recipient.
    pub fn script(&self) -> NoteScript {
        self.0.script().into()
    }

    /// Returns the inputs passed to the recipient script.
    pub fn inputs(&self) -> NoteInputs {
        self.0.inputs().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteRecipient> for NoteRecipient {
    fn from(native_note_recipient: NativeNoteRecipient) -> Self {
        NoteRecipient(native_note_recipient)
    }
}

impl From<&NativeNoteRecipient> for NoteRecipient {
    fn from(native_note_recipient: &NativeNoteRecipient) -> Self {
        NoteRecipient(native_note_recipient.clone())
    }
}

impl From<NoteRecipient> for NativeNoteRecipient {
    fn from(note_recipient: NoteRecipient) -> Self {
        note_recipient.0
    }
}

impl From<&NoteRecipient> for NativeNoteRecipient {
    fn from(note_recipient: &NoteRecipient) -> Self {
        note_recipient.0.clone()
    }
}

// RECIPIENT ARRAY
// ================================================================================================

/// Convenience wrapper for arrays of note recipients.
#[derive(Clone)]
#[wasm_bindgen]
pub struct RecipientArray(Vec<NoteRecipient>);

#[wasm_bindgen]
impl RecipientArray {
    #[wasm_bindgen(constructor)]
    /// Creates a new recipient array.
    pub fn new(recipient_array: Option<Vec<NoteRecipient>>) -> RecipientArray {
        let recipients = recipient_array.unwrap_or_default();
        RecipientArray(recipients)
    }

    /// Appends a recipient to the array.
    pub fn push(&mut self, recipient: &NoteRecipient) {
        self.0.push(recipient.clone());
    }
}

// CONVERSIONS
// ================================================================================================

impl From<Vec<NativeNoteRecipient>> for RecipientArray {
    fn from(recipients: Vec<NativeNoteRecipient>) -> Self {
        RecipientArray(recipients.into_iter().map(NoteRecipient::from).collect())
    }
}

impl From<&RecipientArray> for Vec<NativeNoteRecipient> {
    fn from(recipient_array: &RecipientArray) -> Self {
        recipient_array.0.iter().map(NativeNoteRecipient::from).collect()
    }
}
