use miden_client::Word as NativeWord;
use miden_client::note::{NoteAttachment as NativeNoteAttachment, NoteAttachmentScheme};
use wasm_bindgen::prelude::*;

use super::word::Word;

/// An attachment to a note.
///
/// Note attachments provide additional context about how notes should be processed.
/// For example, a network account target attachment indicates that the note should
/// be consumed by a specific network account.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteAttachment(NativeNoteAttachment);

#[wasm_bindgen]
impl NoteAttachment {
    /// Creates a new note attachment with no scheme (scheme = 0).
    #[wasm_bindgen(js_name = "newWord")]
    pub fn new_word(word: &Word) -> NoteAttachment {
        let native_word: NativeWord = word.into();
        NoteAttachment(NativeNoteAttachment::new_word(NoteAttachmentScheme::none(), native_word))
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteAttachment> for NoteAttachment {
    fn from(native_note_attachment: NativeNoteAttachment) -> Self {
        NoteAttachment(native_note_attachment)
    }
}

impl From<&NativeNoteAttachment> for NoteAttachment {
    fn from(native_note_attachment: &NativeNoteAttachment) -> Self {
        NoteAttachment(native_note_attachment.clone())
    }
}

impl From<NoteAttachment> for NativeNoteAttachment {
    fn from(note_attachment: NoteAttachment) -> Self {
        note_attachment.0
    }
}

impl From<&NoteAttachment> for NativeNoteAttachment {
    fn from(note_attachment: &NoteAttachment) -> Self {
        note_attachment.0.clone()
    }
}
