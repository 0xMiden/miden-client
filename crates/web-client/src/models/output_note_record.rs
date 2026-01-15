use miden_client::store::OutputNoteRecord as NativeOutputNoteRecord;
use wasm_bindgen::prelude::*;

use super::note_assets::NoteAssets;
use super::note_id::NoteId;
use super::note_inclusion_proof::NoteInclusionProof;
use super::note_metadata::NoteMetadata;
use super::note_recipient::NoteRecipient;
use super::output_note_state::OutputNoteState;
use super::word::Word;

/// Represents an output note tracked by the client store.
#[derive(Clone)]
#[wasm_bindgen]
pub struct OutputNoteRecord(NativeOutputNoteRecord);

#[wasm_bindgen]
impl OutputNoteRecord {
    /// Returns the note ID.
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the current processing state for this note.
    pub fn state(&self) -> OutputNoteState {
        self.0.state().into()
    }

    /// Returns the recipient digest committed for the note.
    #[wasm_bindgen(js_name = "recipientDigest")]
    pub fn recipient_digest(&self) -> Word {
        self.0.recipient_digest().into()
    }

    /// Returns the note assets.
    pub fn assets(&self) -> NoteAssets {
        self.0.assets().into()
    }

    /// Returns the note metadata.
    pub fn metadata(&self) -> NoteMetadata {
        self.0.metadata().into()
    }

    /// Returns the inclusion proof when the note is committed.
    #[wasm_bindgen(js_name = "inclusionProof")]
    pub fn inclusion_proof(&self) -> Option<NoteInclusionProof> {
        self.0.inclusion_proof().map(Into::into)
    }

    /// Returns the recipient details if available.
    pub fn recipient(&self) -> Option<NoteRecipient> {
        self.0.recipient().map(Into::into)
    }

    /// Returns the expected block height for the note.
    #[wasm_bindgen(js_name = "expectedHeight")]
    pub fn expected_height(&self) -> u32 {
        self.0.expected_height().as_u32()
    }

    /// Returns the nullifier when the recipient is known.
    pub fn nullifier(&self) -> Option<String> {
        self.0.nullifier().map(|nullifier| nullifier.to_hex())
    }

    /// Returns true if the note has been consumed on chain.
    #[wasm_bindgen(js_name = "isConsumed")]
    pub fn is_consumed(&self) -> bool {
        self.0.is_consumed()
    }

    /// Returns true if the note is committed on chain.
    #[wasm_bindgen(js_name = "isCommitted")]
    pub fn is_committed(&self) -> bool {
        self.0.is_committed()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeOutputNoteRecord> for OutputNoteRecord {
    fn from(native_note: NativeOutputNoteRecord) -> Self {
        OutputNoteRecord(native_note)
    }
}

impl From<&NativeOutputNoteRecord> for OutputNoteRecord {
    fn from(native_note: &NativeOutputNoteRecord) -> Self {
        OutputNoteRecord(native_note.clone())
    }
}
