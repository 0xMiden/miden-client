use miden_client::account::AccountId as NativeAccountId;
use miden_client::note::{
    NetworkAccountTarget as NativeNetworkAccountTarget,
    NoteAttachment as NativeNoteAttachment,
    NoteAttachmentScheme as NativeNoteAttachmentScheme,
};
use miden_client::{Felt as NativeFelt, Word as NativeWord};
use miden_protocol::note::NoteAttachmentContent;

use super::account_id::AccountId;
use super::felt::Felt;
use super::note_attachment_kind::NoteAttachmentKind;
use super::note_execution_hint::NoteExecutionHint;
use super::word::Word;
#[cfg(feature = "wasm")]
use crate::models::miden_arrays::FeltArray;
use crate::prelude::*;

// NOTE ATTACHMENT SCHEME
// ================================================================================================

/// Describes the type of a note attachment.
///
/// Value `0` is reserved to signal that the scheme is none or absent. Whenever the kind of
/// attachment is not standardized or interoperability is unimportant, this none value can be used.
#[bindings]
#[derive(Clone, Copy)]
pub struct NoteAttachmentScheme(NativeNoteAttachmentScheme);

#[bindings]
impl NoteAttachmentScheme {
    #[bindings(constructor)]
    pub fn new(scheme: u32) -> NoteAttachmentScheme {
        NoteAttachmentScheme(NativeNoteAttachmentScheme::new(scheme))
    }

    #[bindings(factory, js_name = "none")]
    pub fn none_scheme() -> NoteAttachmentScheme {
        NoteAttachmentScheme(NativeNoteAttachmentScheme::none())
    }

    #[bindings]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

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
#[bindings]
#[derive(Clone, Default)]
pub struct NoteAttachment(pub(crate) NativeNoteAttachment);

#[bindings]
impl NoteAttachment {
    #[bindings(constructor)]
    pub fn new() -> NoteAttachment {
        NoteAttachment(NativeNoteAttachment::default())
    }

    pub fn attachment_scheme(&self) -> NoteAttachmentScheme {
        self.0.attachment_scheme().into()
    }

    pub fn attachment_kind(&self) -> NoteAttachmentKind {
        self.0.attachment_kind().into()
    }

    pub fn as_word(&self) -> Option<Word> {
        match self.0.content() {
            NoteAttachmentContent::Word(word) => Some((*word).into()),
            _ => None,
        }
    }

    #[bindings(factory)]
    pub fn new_word(scheme: &NoteAttachmentScheme, word: &Word) -> NoteAttachment {
        let native_word: NativeWord = word.into();
        NoteAttachment(NativeNoteAttachment::new_word(scheme.into(), native_word))
    }

    #[bindings]
    pub fn new_network_account_target(
        target_id: &AccountId,
        exec_hint: &NoteExecutionHint,
    ) -> JsResult<NoteAttachment> {
        let native_account_id: NativeAccountId = target_id.into();
        let native_target = NativeNetworkAccountTarget::new(native_account_id, exec_hint.into())
            .map_err(|err| platform::error_with_context(err, "failed to create network account target"))?;
        let native_attachment: NativeNoteAttachment = native_target.into();
        Ok(NoteAttachment(native_attachment))
    }
}

// Platform-specific methods
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl NoteAttachment {
    /// Creates a new note attachment with Array content from the provided elements.
    
    pub fn new_array(
        scheme: &NoteAttachmentScheme,
        elements: &FeltArray,
    ) -> JsResult<NoteAttachment> {
        let wrapper_elements: Vec<Felt> = elements.into();
        let native_elements: Vec<NativeFelt> = wrapper_elements.into_iter().map(Into::into).collect();
        NativeNoteAttachment::new_array(scheme.into(), native_elements)
            .map(NoteAttachment)
            .map_err(|err| platform::error_with_context(err, "failed to create note attachment array"))
    }

    /// Returns the content as an array of Felts if the attachment kind is Array, otherwise None.
    
    pub fn as_array(&self) -> Option<FeltArray> {
        match self.0.content() {
            NoteAttachmentContent::Array(array) => {
                let felts: Vec<Felt> = array.as_slice().iter().map(|f| (*f).into()).collect();
                Some(felts.into())
            },
            _ => None,
        }
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl NoteAttachment {
    /// Creates a new note attachment with Array content from the provided elements.
    #[napi(factory)]
    pub fn new_array(
        scheme: &NoteAttachmentScheme,
        elements: Vec<&Felt>,
    ) -> JsResult<NoteAttachment> {
        let native_elements: Vec<NativeFelt> = elements.into_iter().map(|f| f.into()).collect();
        NativeNoteAttachment::new_array(scheme.into(), native_elements)
            .map(NoteAttachment)
            .map_err(|err| platform::error_with_context(err, "failed to create note attachment array"))
    }

    /// Returns the content as an array of Felts if the attachment kind is Array, otherwise None.
    pub fn as_array(&self) -> Option<Vec<Felt>> {
        match self.0.content() {
            NoteAttachmentContent::Array(array) => {
                let felts: Vec<Felt> = array.as_slice().iter().map(|f| (*f).into()).collect();
                Some(felts)
            },
            _ => None,
        }
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
