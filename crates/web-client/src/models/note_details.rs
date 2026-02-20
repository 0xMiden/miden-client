use miden_client::Word as NativeWord;
use miden_client::note::NoteDetails as NativeNoteDetails;
use crate::prelude::*;

use super::note_assets::NoteAssets;
use super::note_id::NoteId;
use super::note_recipient::NoteRecipient;
use super::word::Word;

/// Details of a note consisting of assets, script, inputs, and a serial number.
///
/// See the `Note` type for more details.
#[bindings]
#[derive(Clone)]
pub struct NoteDetails(pub(crate) NativeNoteDetails);

#[bindings]
impl NoteDetails {
    #[bindings(constructor)]
    pub fn new(note_assets: &NoteAssets, note_recipient: &NoteRecipient) -> NoteDetails {
        NoteDetails(NativeNoteDetails::new(note_assets.into(), note_recipient.into()))
    }

    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    pub fn assets(&self) -> NoteAssets {
        self.0.assets().into()
    }

    pub fn recipient(&self) -> NoteRecipient {
        self.0.recipient().into()
    }

    pub fn nullifier(&self) -> Word {
        let nullifier = self.0.nullifier();
        let elements: [miden_client::Felt; 4] =
            nullifier.as_elements().try_into().expect("nullifier has 4 elements");
        let native_word: NativeWord = NativeWord::from(&elements);
        native_word.into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NoteDetails> for NativeNoteDetails {
    fn from(note_details: NoteDetails) -> Self {
        note_details.0
    }
}

impl From<&NoteDetails> for NativeNoteDetails {
    fn from(note_details: &NoteDetails) -> Self {
        note_details.0.clone()
    }
}

impl From<NativeNoteDetails> for NoteDetails {
    fn from(note_details: NativeNoteDetails) -> NoteDetails {
        NoteDetails(note_details)
    }
}

impl From<&NativeNoteDetails> for NoteDetails {
    fn from(note_details: &NativeNoteDetails) -> NoteDetails {
        NoteDetails(note_details.clone())
    }
}
