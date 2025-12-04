use miden_client::account::AccountId as NativeAccountId;
use miden_client::note::{NoteExecutionMode as NativeNoteExecutionMode, NoteTag as NativeNoteTag};
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::note_execution_mode::NoteExecutionMode;

/// Note tags are best-effort filters for notes registered with the network. They hint whether a
/// note is meant for network or local execution and optionally embed a target (like part of an
/// [`AccountId`]) or a use-case payload. Public notes are required for network execution so that
/// full details are available for validation.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteTag(pub(crate) NativeNoteTag);

#[wasm_bindgen]
impl NoteTag {
    /// Builds a single-target tag derived from an account ID.
    #[wasm_bindgen(js_name = "fromAccountId")]
    pub fn from_account_id(account_id: &AccountId) -> NoteTag {
        let native_account_id: NativeAccountId = account_id.into();
        let native_note_tag = NativeNoteTag::from_account_id(native_account_id);
        NoteTag(native_note_tag)
    }

    /// Builds a tag for a public use case with an explicit payload and execution mode.
    #[wasm_bindgen(js_name = "forPublicUseCase")]
    pub fn for_public_use_case(
        use_case_id: u16,
        payload: u16,
        execution: &NoteExecutionMode,
    ) -> NoteTag {
        let native_execution: NativeNoteExecutionMode = execution.into();
        let native_note_tag =
            NativeNoteTag::for_public_use_case(use_case_id, payload, native_execution).unwrap();
        NoteTag(native_note_tag)
    }

    /// Builds a tag for a local-only use case.
    #[wasm_bindgen(js_name = "forLocalUseCase")]
    pub fn for_local_use_case(use_case_id: u16, payload: u16) -> NoteTag {
        let native_note_tag = NativeNoteTag::for_local_use_case(use_case_id, payload).unwrap();
        NoteTag(native_note_tag)
    }

    /// Returns true if the tag targets a single account.
    #[wasm_bindgen(js_name = "isSingleTarget")]
    pub fn is_single_target(&self) -> bool {
        self.0.is_single_target()
    }

    /// Returns the execution mode encoded in this tag.
    #[wasm_bindgen(js_name = "executionMode")]
    pub fn execution_mode(&self) -> NoteExecutionMode {
        self.0.execution_mode().into()
    }

    /// Returns the underlying 32-bit representation.
    #[wasm_bindgen(js_name = "asU32")]
    pub fn as_u32(&self) -> u32 {
        self.0.as_u32()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteTag> for NoteTag {
    fn from(native_note_tag: NativeNoteTag) -> Self {
        NoteTag(native_note_tag)
    }
}

impl From<&NativeNoteTag> for NoteTag {
    fn from(native_note_tag: &NativeNoteTag) -> Self {
        NoteTag(*native_note_tag)
    }
}

impl From<NoteTag> for NativeNoteTag {
    fn from(note_tag: NoteTag) -> Self {
        note_tag.0
    }
}

impl From<&NoteTag> for NativeNoteTag {
    fn from(note_tag: &NoteTag) -> Self {
        note_tag.0
    }
}
