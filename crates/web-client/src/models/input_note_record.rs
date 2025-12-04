use miden_client::store::InputNoteRecord as NativeInputNoteRecord;
use miden_client::transaction::InputNote as NativeInputNote;
use wasm_bindgen::prelude::*;

use super::input_note_state::InputNoteState;
use super::note_details::NoteDetails;
use super::note_id::NoteId;
use super::note_inclusion_proof::NoteInclusionProof;
use super::note_metadata::NoteMetadata;
use super::word::Word;
use crate::js_error_with_context;
use crate::models::input_note::InputNote;

/// Represents a Note of which the Store can keep track and retrieve.
///
/// An [`InputNoteRecord`] contains all the information of a [`NoteDetails`], in addition of
/// specific information about the note state.
///
/// Once a proof is received, the [`InputNoteRecord`] can be transformed into an [`InputNote`] and
/// used as input for transactions. It is also possible to convert [`Note`] and [`InputNote`] into
/// [`InputNoteRecord`] (we fill the `metadata` and `inclusion_proof` fields if possible).
///
/// Notes can also be consumed as unauthenticated notes, where their existence is verified by the
/// network.
#[derive(Clone)]
#[wasm_bindgen]
pub struct InputNoteRecord(NativeInputNoteRecord);

#[wasm_bindgen]
impl InputNoteRecord {
    /// Returns the note ID.
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the current processing state for this note.
    pub fn state(&self) -> InputNoteState {
        self.0.state().into()
    }

    /// Returns the note details, if present.
    pub fn details(&self) -> NoteDetails {
        self.0.details().into()
    }

    /// Returns the note metadata if available.
    pub fn metadata(&self) -> Option<NoteMetadata> {
        self.0.metadata().map(Into::into)
    }

    /// Returns the note commitment (id + metadata), if available.
    pub fn commitment(&self) -> Option<Word> {
        self.0.commitment().map(Into::into)
    }

    /// Returns the inclusion proof when the note is authenticated.
    #[wasm_bindgen(js_name = "inclusionProof")]
    pub fn inclusion_proof(&self) -> Option<NoteInclusionProof> {
        self.0.inclusion_proof().map(Into::into)
    }

    /// Returns the transaction ID that consumed this note, if any.
    #[wasm_bindgen(js_name = "consumerTransactionId")]
    pub fn consumer_transaction_id(&self) -> Option<String> {
        self.0.consumer_transaction_id().map(ToString::to_string)
    }

    /// Returns the nullifier for this note.
    pub fn nullifier(&self) -> String {
        self.0.nullifier().to_hex()
    }

    /// Returns true if the record contains authentication data (proof).
    #[wasm_bindgen(js_name = "isAuthenticated")]
    pub fn is_authenticated(&self) -> bool {
        self.0.is_authenticated()
    }

    /// Returns true if the note has already been consumed.
    #[wasm_bindgen(js_name = "isConsumed")]
    pub fn is_consumed(&self) -> bool {
        self.0.is_consumed()
    }

    /// Returns true if the note is currently being processed.
    #[wasm_bindgen(js_name = "isProcessing")]
    pub fn is_processing(&self) -> bool {
        self.0.is_processing()
    }

    /// Converts the record into an `InputNote` (including proof when available).
    #[wasm_bindgen(js_name = "toInputNote")]
    pub fn to_input_note(&self) -> Result<InputNote, JsValue> {
        let input_note: NativeInputNote = self.0.clone().try_into().map_err(|err| {
            js_error_with_context(err, "could not create InputNote from InputNoteRecord")
        })?;
        Ok(InputNote(input_note))
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeInputNoteRecord> for InputNoteRecord {
    fn from(native_note: NativeInputNoteRecord) -> Self {
        InputNoteRecord(native_note)
    }
}

impl From<&NativeInputNoteRecord> for InputNoteRecord {
    fn from(native_note: &NativeInputNoteRecord) -> Self {
        InputNoteRecord(native_note.clone())
    }
}
