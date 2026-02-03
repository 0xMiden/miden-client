use miden_client::account::AccountId as NativeAccountId;
use miden_client::note::{
    NetworkAccountTarget as NativeNetworkAccountTarget, NoteAttachment as NativeNoteAttachment,
    NoteAttachmentScheme as NativeNoteAttachmentScheme,
};
use miden_client::{Felt as NativeFelt, Word as NativeWord};
use miden_protocol::note::NoteAttachmentContent;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::felt::Felt;
use super::note_attachment_kind::NoteAttachmentKind;
use super::note_execution_hint::NoteExecutionHint;
use super::word::Word;
use crate::models::miden_arrays::FeltArray;

// NOTE ATTACHMENT SCHEME
// ================================================================================================

/// Describes the type of a note attachment.
///
/// Value `0` is reserved to signal that the scheme is none or absent. Whenever the kind of
/// attachment is not standardized or interoperability is unimportant, this none value can be used.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteAttachmentScheme(NativeNoteAttachmentScheme);

#[wasm_bindgen]
impl NoteAttachmentScheme {
    /// Creates a new `NoteAttachmentScheme` from a u32.
    #[wasm_bindgen(constructor)]
    pub fn new(scheme: u32) -> NoteAttachmentScheme {
        NoteAttachmentScheme(NativeNoteAttachmentScheme::new(scheme))
    }

    /// Returns the `NoteAttachmentScheme` that signals the absence of an attachment scheme.
    pub fn none() -> NoteAttachmentScheme {
        NoteAttachmentScheme(NativeNoteAttachmentScheme::none())
    }

    /// Returns true if the attachment scheme is the reserved value that signals an absent scheme.
    #[wasm_bindgen(js_name = "isNone")]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// Returns the note attachment scheme as a u32.
    #[wasm_bindgen(js_name = "asU32")]
    pub fn as_u32(&self) -> u32 {
        self.0.as_u32()
    }
}

impl From<NativeNoteAttachmentScheme> for NoteAttachmentScheme {
    fn from(native: NativeNoteAttachmentScheme) -> Self {
        NoteAttachmentScheme(native)
    }
}

impl From<&NoteAttachmentScheme> for NativeNoteAttachmentScheme {
    fn from(scheme: &NoteAttachmentScheme) -> Self {
        scheme.0
    }
}

// NOTE ATTACHMENT
// ================================================================================================

/// An attachment to a note.
///
/// Note attachments provide additional context about how notes should be processed.
/// For example, a network account target attachment indicates that the note should
/// be consumed by a specific network account.
#[derive(Clone, Default)]
#[wasm_bindgen]
pub struct NoteAttachment(NativeNoteAttachment);

#[wasm_bindgen]
impl NoteAttachment {
    /// Creates a default (empty) note attachment.
    #[wasm_bindgen(constructor)]
    pub fn new() -> NoteAttachment {
        NoteAttachment(NativeNoteAttachment::default())
    }

    /// Creates a new note attachment with Word content from the provided word.
    #[wasm_bindgen(js_name = "newWord")]
    pub fn new_word(scheme: &NoteAttachmentScheme, word: &Word) -> NoteAttachment {
        let native_word: NativeWord = word.into();
        NoteAttachment(NativeNoteAttachment::new_word(scheme.into(), native_word))
    }

    /// Creates a new note attachment with Array content from the provided elements.
    #[wasm_bindgen(js_name = "newArray")]
    pub fn new_array(
        scheme: &NoteAttachmentScheme,
        elements: &FeltArray,
    ) -> Result<NoteAttachment, JsValue> {
        let native_elements: Vec<NativeFelt> = elements.into();
        NativeNoteAttachment::new_array(scheme.into(), native_elements)
            .map(NoteAttachment)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Returns the attachment scheme.
    #[wasm_bindgen(js_name = "attachmentScheme")]
    pub fn attachment_scheme(&self) -> NoteAttachmentScheme {
        self.0.attachment_scheme().into()
    }

    /// Returns the attachment kind.
    #[wasm_bindgen(js_name = "attachmentKind")]
    pub fn attachment_kind(&self) -> NoteAttachmentKind {
        self.0.attachment_kind().into()
    }

    /// Returns the content as a Word if the attachment kind is Word, otherwise None.
    #[wasm_bindgen(js_name = "asWord")]
    pub fn as_word(&self) -> Option<Word> {
        match self.0.content() {
            NoteAttachmentContent::Word(word) => Some((*word).into()),
            _ => None,
        }
    }

    /// Returns the content as an array of Felts if the attachment kind is Array, otherwise None.
    #[wasm_bindgen(js_name = "asArray")]
    pub fn as_array(&self) -> Option<FeltArray> {
        match self.0.content() {
            NoteAttachmentContent::Array(array) => {
                let felts: Vec<Felt> = array.as_slice().iter().map(|f| (*f).into()).collect();
                Some(felts.into())
            },
            _ => None,
        }
    }

    /// Creates a new note attachment for a network account target.
    ///
    /// This attachment indicates that the note should be consumed by a specific network account.
    /// Network accounts are accounts whose storage mode is `Network`, meaning the network (nodes)
    /// can execute transactions on behalf of the account.
    ///
    /// # Arguments
    /// * `target_id` - The ID of the network account that should consume the note
    /// * `exec_hint` - A hint about when the note can be executed
    ///
    /// # Errors
    /// Returns an error if the target account is not a network account.
    #[wasm_bindgen(js_name = "newNetworkAccountTarget")]
    pub fn new_network_account_target(
        target_id: &AccountId,
        exec_hint: &NoteExecutionHint,
    ) -> Result<NoteAttachment, JsValue> {
        let native_account_id: NativeAccountId = target_id.into();
        let native_target = NativeNetworkAccountTarget::new(native_account_id, exec_hint.into())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let native_attachment: NativeNoteAttachment = native_target.into();
        Ok(NoteAttachment(native_attachment))
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
