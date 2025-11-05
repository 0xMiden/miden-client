use miden_client::note::NoteExecutionHint as NativeNoteExecutionHint;
use wasm_bindgen::prelude::*;

/// Scheduling hint describing when a note becomes consumable.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteExecutionHint(NativeNoteExecutionHint);

#[wasm_bindgen]
impl NoteExecutionHint {
    /// Returns a hint with no additional restrictions.
    pub fn none() -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::None)
    }

    /// Returns a hint that allows consumption at any time.
    pub fn always() -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::Always)
    }

    #[wasm_bindgen(js_name = "afterBlock")]
    /// Returns a hint that defers consumption until after the given block number.
    pub fn after_block(block_num: u32) -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::after_block(block_num.into()).unwrap())
    }

    #[wasm_bindgen(js_name = "onBlockSlot")]
    /// Returns a hint that limits consumption to a specific slot schedule.
    pub fn on_block_slot(epoch_len: u8, slot_len: u8, slot_offset: u8) -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::on_block_slot(epoch_len, slot_len, slot_offset))
    }

    #[wasm_bindgen(js_name = "fromParts")]
    /// Recreates a hint from low-level tag/payload components.
    pub fn from_parts(tag: u8, payload: u32) -> NoteExecutionHint {
        NoteExecutionHint(NativeNoteExecutionHint::from_parts(tag, payload).unwrap())
    }

    #[wasm_bindgen(js_name = "canBeConsumed")]
    /// Returns whether the hint allows consumption at the provided block.
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
