use miden_protocol::note::NoteExecutionHint as NativeNoteExecutionHint;
use wasm_bindgen::prelude::*;

/// Hint describing when a note can be consumed.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteExecutionHint(NativeNoteExecutionHint);

#[wasm_bindgen]
impl NoteExecutionHint {
    /// Creates a hint that does not specify any execution constraint.
    pub fn none() -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::None)
    }

    /// Creates a hint indicating the note can always be consumed.
    pub fn always() -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::Always)
    }

    /// Creates a hint that activates after the given block number.
    #[wasm_bindgen(js_name = "afterBlock")]
    pub fn after_block(block_num: u32) -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::after_block(block_num.into()))
    }

    /// Creates a hint that allows execution in a specific slot of a round.
    #[wasm_bindgen(js_name = "onBlockSlot")]
    pub fn on_block_slot(epoch_len: u8, slot_len: u8, slot_offset: u8) -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::on_block_slot(epoch_len, slot_len, slot_offset))
    }

    /// Reconstructs a hint from its encoded tag and payload.
    #[wasm_bindgen(js_name = "fromParts")]
    pub fn from_parts(tag: u8, payload: u32) -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::from_parts(tag, payload).unwrap())
    }

    /// Returns whether the note can be consumed at the provided block height.
    #[wasm_bindgen(js_name = "canBeConsumed")]
    pub fn can_be_consumed(&self, block_num: u32) -> bool {
        self.0.can_be_consumed(block_num.into()).unwrap()
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
