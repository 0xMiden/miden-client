use miden_client::note::NoteMetadata as NativeNoteMetadata;
use crate::prelude::*;

use super::account_id::AccountId;
use super::note_attachment::NoteAttachment;
use super::note_tag::NoteTag;
use super::note_type::NoteType;

/// Metadata associated with a note.
///
/// This metadata includes the sender, note type, tag, and an optional attachment.
/// Attachments provide additional context about how notes should be processed.
#[bindings]
#[derive(Clone)]
pub struct NoteMetadata(NativeNoteMetadata);

#[bindings]
impl NoteMetadata {
    #[bindings(constructor)]
    pub fn new(sender: &AccountId, note_type: NoteType, note_tag: &NoteTag) -> NoteMetadata {
        let native_note_metadata =
            NativeNoteMetadata::new(sender.into(), note_type.into(), note_tag.into());
        NoteMetadata(native_note_metadata)
    }

    pub fn sender(&self) -> AccountId {
        self.0.sender().into()
    }

    pub fn tag(&self) -> NoteTag {
        self.0.tag().into()
    }

    pub fn note_type(&self) -> NoteType {
        self.0.note_type().into()
    }

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
