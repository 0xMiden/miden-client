use miden_client::store::InputNoteRecord as NativeInputNoteRecord;
use miden_client::transaction::InputNote as NativeInputNote;
use wasm_bindgen::prelude::*;

use super::input_note_state::InputNoteState;
use super::note_details::NoteDetails;
use super::note_id::NoteId;
use super::note_inclusion_proof::NoteInclusionProof;
use super::note_metadata::NoteMetadata;
use crate::js_error_with_context;
use crate::models::input_note::InputNote;

#[derive(Clone)]
#[wasm_bindgen]
pub struct InputNoteRecord(NativeInputNoteRecord);

#[wasm_bindgen]
impl InputNoteRecord {
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    pub fn state(&self) -> InputNoteState {
        self.0.state().into()
    }

    pub fn details(&self) -> NoteDetails {
        self.0.details().into()
    }

    pub fn metadata(&self) -> Option<NoteMetadata> {
        self.0.metadata().map(Into::into)
    }

    #[wasm_bindgen(js_name = "inclusionProof")]
    pub fn inclusion_proof(&self) -> Option<NoteInclusionProof> {
        self.0.inclusion_proof().map(Into::into)
    }

    #[wasm_bindgen(js_name = "consumerTransactionId")]
    pub fn consumer_transaction_id(&self) -> Option<String> {
        self.0.consumer_transaction_id().map(ToString::to_string)
    }

    pub fn nullifier(&self) -> String {
        self.0.nullifier().to_hex()
    }

    #[wasm_bindgen(js_name = "isAuthenticated")]
    pub fn is_authenticated(&self) -> bool {
        self.0.is_authenticated()
    }

    #[wasm_bindgen(js_name = "isConsumed")]
    pub fn is_consumed(&self) -> bool {
        self.0.is_consumed()
    }

    #[wasm_bindgen(js_name = "isProcessing")]
    pub fn is_processing(&self) -> bool {
        self.0.is_processing()
    }

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
