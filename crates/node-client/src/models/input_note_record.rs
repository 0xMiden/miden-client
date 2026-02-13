use miden_client::note::Note as NativeNote;
use miden_client::store::InputNoteRecord as NativeInputNoteRecord;
use napi::bindgen_prelude::*;

use super::note::Note;
use super::note_id::NoteId;
use super::{napi_delegate, napi_wrap};

napi_wrap!(clone InputNoteRecord wraps NativeInputNoteRecord, one_way);

napi_delegate!(impl InputNoteRecord {
    /// Returns the note ID.
    delegate id -> NoteId;
    /// Returns true if the record contains authentication data (proof).
    delegate is_authenticated -> bool;
    /// Returns true if the note has already been consumed.
    delegate is_consumed -> bool;
    /// Returns true if the note is currently being processed.
    delegate is_processing -> bool;
});

#[napi]
impl InputNoteRecord {
    /// Returns the consumer transaction ID if the note has been consumed.
    #[napi]
    pub fn consumer_transaction_id(&self) -> Option<String> {
        self.0.consumer_transaction_id().map(ToString::to_string)
    }

    /// Returns the nullifier for this note.
    #[napi]
    pub fn nullifier(&self) -> String {
        self.0.nullifier().to_hex()
    }

    /// Converts the record into a Note.
    #[napi]
    pub fn to_note(&self) -> Result<Note> {
        let note: NativeNote = self.0.clone().try_into().map_err(|err| {
            napi::Error::from_reason(format!("could not create Note from InputNoteRecord: {err}"))
        })?;
        Ok(Note::from(note))
    }
}
