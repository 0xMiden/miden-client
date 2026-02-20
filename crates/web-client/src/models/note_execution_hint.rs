use miden_client::note::NoteExecutionHint as NativeNoteExecutionHint;
use crate::prelude::*;

/// Hint describing when a note can be consumed.
#[bindings]
#[derive(Clone, Copy)]
pub struct NoteExecutionHint(pub(crate) NativeNoteExecutionHint);

#[bindings]
impl NoteExecutionHint {
    #[bindings(factory, js_name = "none")]
    pub fn none_hint() -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::None)
    }

    #[bindings(factory)]
    pub fn always() -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::Always)
    }

    #[bindings(factory)]
    pub fn after_block(block_num: u32) -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::after_block(block_num.into()))
    }

    #[bindings(factory)]
    pub fn on_block_slot(epoch_len: u8, slot_len: u8, slot_offset: u8) -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::on_block_slot(epoch_len, slot_len, slot_offset))
    }
}

// Platform-specific methods that differ
#[cfg(feature = "wasm")]
impl NoteExecutionHint {
    /// Reconstructs a hint from its encoded tag and payload.
    
    pub fn from_parts(tag: u8, payload: u32) -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::from_parts(tag, payload).unwrap())
    }

    /// Returns whether the note can be consumed at the provided block height.
    
    pub fn can_be_consumed(&self, block_num: u32) -> bool {
        self.0.can_be_consumed(block_num.into()).unwrap()
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl NoteExecutionHint {
    /// Reconstructs a hint from its encoded tag and payload.
    #[napi(factory)]
    pub fn from_parts(tag: u8, payload: u32) -> JsResult<NoteExecutionHint> {
        NativeNoteExecutionHint::from_parts(tag, payload)
            .map(NoteExecutionHint)
            .map_err(|err| platform::error_with_context(err, "failed to create NoteExecutionHint"))
    }

    /// Returns whether the note can be consumed at the provided block height.
    /// Returns None if the hint does not provide enough information to determine consumption.
    pub fn can_be_consumed(&self, block_num: u32) -> Option<bool> {
        self.0.can_be_consumed(block_num.into())
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NoteExecutionHint> for NativeNoteExecutionHint {
    fn from(note_execution_hint: NoteExecutionHint) -> Self {
        note_execution_hint.0
    }
}

impl From<&NoteExecutionHint> for NativeNoteExecutionHint {
    fn from(note_execution_hint: &NoteExecutionHint) -> Self {
        note_execution_hint.0
    }
}
