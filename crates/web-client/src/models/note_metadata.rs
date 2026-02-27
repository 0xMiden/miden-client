use miden_client::note::NoteMetadata as NativeNoteMetadata;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::note_attachment::NoteAttachment;
use super::{NoteTag, NoteType};

/// Metadata associated with a note.
///
/// This metadata includes the sender, note type, tag, and an optional attachment.
/// Attachments provide additional context about how notes should be processed.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteMetadata(NativeNoteMetadata);

#[wasm_bindgen]
impl NoteMetadata {
    /// Creates metadata for a note.
    #[wasm_bindgen(constructor)]
    pub fn new(sender: &AccountId, note_type: NoteType, note_tag: &NoteTag) -> NoteMetadata {
        let native_note_metadata =
            NativeNoteMetadata::new(sender.into(), note_type.into()).with_tag(note_tag.into());
        NoteMetadata(native_note_metadata)
    }

    /// Returns the account that created the note.
    pub fn sender(&self) -> AccountId {
        self.0.sender().into()
    }

    /// Returns the tag associated with the note.
    pub fn tag(&self) -> NoteTag {
        self.0.tag().into()
    }

    /// Returns whether the note is private, encrypted, or public.
    #[wasm_bindgen(js_name = "noteType")]
    pub fn note_type(&self) -> NoteType {
        self.0.note_type().into()
    }

    /// Returns the attachment of the note.
    pub fn attachment(&self) -> NoteAttachment {
        self.0.attachment().into()
    }

    /// Adds an attachment to this metadata and returns the updated metadata.
    ///
    /// Attachments provide additional context about how notes should be processed.
    /// For example, a `NetworkAccountTarget` attachment indicates that the note
    /// should be consumed by a specific network account.
    #[wasm_bindgen(js_name = "withAttachment")]
    pub fn with_attachment(&self, attachment: &NoteAttachment) -> NoteMetadata {
        let native_attachment = attachment.into();
        NoteMetadata(self.clone().0.with_attachment(native_attachment))
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteMetadata> for NoteMetadata {
    fn from(native_note_metadata: NativeNoteMetadata) -> Self {
        NoteMetadata(native_note_metadata)
    }
}

impl From<&NativeNoteMetadata> for NoteMetadata {
    fn from(native_note_metadata: &NativeNoteMetadata) -> Self {
        NoteMetadata(native_note_metadata.clone())
    }
}

impl From<NoteMetadata> for NativeNoteMetadata {
    fn from(note_metadata: NoteMetadata) -> Self {
        note_metadata.0
    }
}

impl From<&NoteMetadata> for NativeNoteMetadata {
    fn from(note_metadata: &NoteMetadata) -> Self {
        note_metadata.0.clone()
    }
}
