use miden_client::Word as NativeWord;
use miden_client::note::{
    NoteRecipient as NativeNoteRecipient,
    NoteScript as NativeNoteScript,
    NoteStorage as NativeNoteStorage,
};
use wasm_bindgen::prelude::*;

use super::note_script::NoteScript;
use super::note_storage::NoteStorage;
use super::word::Word;
use crate::models::miden_arrays::NoteRecipientArray as RecipientArray;

/// Value that describes under which condition a note can be consumed.
///
/// The recipient is not an account address, instead it is a value that describes when a note can be
/// consumed. Because not all notes have predetermined consumer addresses, e.g. swap notes can be
/// consumed by anyone, the recipient is defined as the code and its storage, that when successfully
/// executed results in the note's consumption.
///
/// Recipient is computed as a nested hash of the serial number, the script root, and the storage
/// commitment, ensuring the recipient digest binds all three pieces of data together.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteRecipient(NativeNoteRecipient);

#[wasm_bindgen]
impl NoteRecipient {
    /// Creates a note recipient from its serial number, script, and storage.
    #[wasm_bindgen(constructor)]
    pub fn new(
        serial_num: &Word,
        note_script: &NoteScript,
        storage: &NoteStorage,
    ) -> NoteRecipient {
        let native_serial_num: NativeWord = serial_num.into();
        let native_note_script: NativeNoteScript = note_script.into();
        let native_note_storage: NativeNoteStorage = storage.into();
        let native_note_recipient =
            NativeNoteRecipient::new(native_serial_num, native_note_script, native_note_storage);

        NoteRecipient(native_note_recipient)
    }

    /// Returns the digest of the recipient data (used in the note commitment).
    pub fn digest(&self) -> Word {
        self.0.digest().into()
    }

    /// Returns the serial number that prevents double spends.
    #[wasm_bindgen(js_name = "serialNum")]
    pub fn serial_num(&self) -> Word {
        self.0.serial_num().into()
    }

    /// Returns the script that controls consumption.
    pub fn script(&self) -> NoteScript {
        self.0.script().into()
    }

    /// Returns the storage provided to the script.
    pub fn storage(&self) -> NoteStorage {
        self.0.storage().into()
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

impl From<&RecipientArray> for Vec<NativeNoteRecipient> {
    fn from(recipient_array: &RecipientArray) -> Self {
        recipient_array.__inner.iter().map(NativeNoteRecipient::from).collect()
    }
}
