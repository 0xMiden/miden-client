use miden_client::account::AccountId as NativeAccountId;
use miden_client::note::NoteTag as NativeNoteTag;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::note_execution_mode::NoteExecutionMode;

const NETWORK_ACCOUNT: u32 = 0x0000_0000;
const NETWORK_PUBLIC_USE_CASE: u32 = 0x4000_0000;
const LOCAL_PUBLIC_ANY: u32 = 0x8000_0000;
const LOCAL_ANY: u32 = 0xc000_0000;
const MAX_USE_CASE_ID_EXPONENT: u8 = 14;

/// Note tags are best-effort filters for notes registered with the network. They hint whether a
/// note is meant for network or local execution and optionally embed a target (like part of an
/// `AccountId`) or a use-case payload. Public notes are required for network execution so that full
/// details are available for validation.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteTag(pub(crate) NativeNoteTag);

#[wasm_bindgen]
impl NoteTag {
    /// Builds a single-target tag derived from an account ID.
    #[wasm_bindgen(js_name = "fromAccountId")]
    pub fn from_account_id(account_id: &AccountId) -> NoteTag {
        let native_account_id: NativeAccountId = account_id.into();
        let native_note_tag = NativeNoteTag::with_account_target(native_account_id);
        NoteTag(native_note_tag)
    }

    /// Builds a tag for a public use case with an explicit payload and execution mode.
    #[wasm_bindgen(js_name = "forPublicUseCase")]
    pub fn for_public_use_case(
        use_case_id: u16,
        payload: u16,
        execution: &NoteExecutionMode,
    ) -> NoteTag {
        if (use_case_id >> MAX_USE_CASE_ID_EXPONENT) != 0 {
            panic!("note use case id must fit in {} bits", MAX_USE_CASE_ID_EXPONENT);
        }

        let tag = if execution.is_network() {
            NETWORK_PUBLIC_USE_CASE | ((use_case_id as u32) << 16) | payload as u32
        } else {
            LOCAL_PUBLIC_ANY | ((use_case_id as u32) << 16) | payload as u32
        };

        NoteTag(NativeNoteTag::new(tag))
    }

    /// Builds a tag for a local-only use case.
    #[wasm_bindgen(js_name = "forLocalUseCase")]
    pub fn for_local_use_case(use_case_id: u16, payload: u16) -> NoteTag {
        if (use_case_id >> MAX_USE_CASE_ID_EXPONENT) != 0 {
            panic!("note use case id must fit in {} bits", MAX_USE_CASE_ID_EXPONENT);
        }

        let tag = LOCAL_ANY | ((use_case_id as u32) << 16) | payload as u32;
        NoteTag(NativeNoteTag::new(tag))
    }

    /// Builds a note tag from its raw u32 representation.
    #[wasm_bindgen(js_name = "fromU32")]
    pub fn from_u32(raw: u32) -> NoteTag {
        NoteTag(NativeNoteTag::from(raw))
    }

    /// Builds a note tag from a hex-encoded string (with or without 0x prefix).
    #[wasm_bindgen(js_name = "fromHex")]
    pub fn from_hex(hex: &str) -> Result<NoteTag, JsValue> {
        let trimmed = hex.strip_prefix("0x").unwrap_or(hex);
        let raw = u32::from_str_radix(trimmed, 16)
            .map_err(|err| JsValue::from_str(&format!("invalid note tag hex: {err}")))?;
        Ok(NoteTag(NativeNoteTag::from(raw)))
    }

    /// Returns true if the tag targets a single account.
    #[wasm_bindgen(js_name = "isSingleTarget")]
    pub fn is_single_target(&self) -> bool {
        (self.0.as_u32() & 0xc000_0000) == NETWORK_ACCOUNT
    }

    /// Returns the execution mode encoded in this tag.
    #[wasm_bindgen(js_name = "executionMode")]
    pub fn execution_mode(&self) -> NoteExecutionMode {
        let prefix = self.0.as_u32() & 0xc000_0000;
        match prefix {
            NETWORK_ACCOUNT | NETWORK_PUBLIC_USE_CASE => NoteExecutionMode::new_network(),
            LOCAL_PUBLIC_ANY | LOCAL_ANY => NoteExecutionMode::new_local(),
            _ => NoteExecutionMode::new_network(),
        }
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
