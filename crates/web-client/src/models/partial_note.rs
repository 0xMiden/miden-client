use miden_client::note::PartialNote as NativePartialNote;

use crate::prelude::*;

use super::note_assets::NoteAssets;
use super::note_id::NoteId;
use super::note_metadata::NoteMetadata;
use super::word::Word;

/// Partial information about a note.
///
/// Partial note consists of `NoteMetadata`, `NoteAssets`, and a recipient digest (see
/// `NoteRecipient`). However, it does not contain detailed recipient info, including
/// note script, note inputs, and note's serial number. This means that a partial note is sufficient
/// to compute note ID and note header, but not sufficient to compute note nullifier, and generally
/// does not have enough info to execute the note.
#[derive(Clone)]
#[bindings]
pub struct PartialNote(NativePartialNote);

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl PartialNote {
    // TODO: new

    /// Returns the identifier of the partial note.
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the metadata attached to the note.
    pub fn metadata(&self) -> NoteMetadata {
        self.0.metadata().into()
    }

    /// Returns the digest of the recipient information.
    #[wasm_bindgen(js_name = "recipientDigest")]
    pub fn recipient_digest(&self) -> Word {
        self.0.recipient_digest().into()
    }

    /// Returns the assets locked in the note.
    pub fn assets(&self) -> NoteAssets {
        self.0.assets().into()
    }
}

#[cfg(feature = "napi")]
impl PartialNote {
    /// Returns the identifier of the partial note.
    #[bindings(napi)]
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the metadata attached to the note.
    #[bindings(napi)]
    pub fn metadata(&self) -> NoteMetadata {
        self.0.metadata().into()
    }

    /// Returns the digest of the recipient information.
    pub fn recipient_digest(&self) -> Word {
        self.0.recipient_digest().into()
    }

    /// Returns the assets locked in the note.
    #[bindings(napi)]
    pub fn assets(&self) -> NoteAssets {
        self.0.assets().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativePartialNote> for PartialNote {
    fn from(native_note: NativePartialNote) -> Self {
        PartialNote(native_note)
    }
}

impl From<&NativePartialNote> for PartialNote {
    fn from(native_note: &NativePartialNote) -> Self {
        PartialNote(native_note.clone())
    }
}

impl From<PartialNote> for NativePartialNote {
    fn from(note: PartialNote) -> Self {
        note.0
    }
}

impl From<&PartialNote> for NativePartialNote {
    fn from(note: &PartialNote) -> Self {
        note.0.clone()
    }
}
