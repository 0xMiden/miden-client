use miden_client::note::{NoteConsumability as NativeNoteConsumability, NoteRelevance};
use miden_client::store::InputNoteRecord as NativeInputNoteRecord;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::input_note_record::InputNoteRecord;

/// Input note record annotated with consumption conditions.
#[derive(Clone)]
#[wasm_bindgen]
pub struct ConsumableNoteRecord {
    input_note_record: InputNoteRecord,
    note_consumability: Vec<NoteConsumability>,
}

#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteConsumability {
    account_id: AccountId,

    // The block number after which the note can be consumed,
    // if None then the note can be consumed immediately
    consumable_after_block: Option<u32>,
}

#[wasm_bindgen]
impl NoteConsumability {
    pub(crate) fn new(
        account_id: AccountId,
        consumable_after_block: Option<u32>,
    ) -> NoteConsumability {
        NoteConsumability { account_id, consumable_after_block }
    }

    /// Returns the account that can consume the note.
    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    /// Returns the block number after which the note becomes consumable (if any).
    #[wasm_bindgen(js_name = "consumableAfterBlock")]
    pub fn consumable_after_block(&self) -> Option<u32> {
        self.consumable_after_block
    }
}

#[wasm_bindgen]
impl ConsumableNoteRecord {
    /// Creates a new consumable note record from an input note record and consumability metadata.
    #[wasm_bindgen(constructor)]
    pub fn new(
        input_note_record: InputNoteRecord,
        note_consumability: Vec<NoteConsumability>,
    ) -> ConsumableNoteRecord {
        ConsumableNoteRecord { input_note_record, note_consumability }
    }

    /// Returns the underlying input note record.
    #[wasm_bindgen(js_name = "inputNoteRecord")]
    pub fn input_note_record(&self) -> InputNoteRecord {
        self.input_note_record.clone()
    }

    /// Returns the consumability entries.
    #[wasm_bindgen(js_name = "noteConsumability")]
    pub fn note_consumability(&self) -> Vec<NoteConsumability> {
        self.note_consumability.clone()
    }
}

// CONVERSIONS
// ================================================================================================
impl From<(NativeInputNoteRecord, Vec<NativeNoteConsumability>)> for ConsumableNoteRecord {
    fn from(
        (input_note_record, note_consumability): (
            NativeInputNoteRecord,
            Vec<NativeNoteConsumability>,
        ),
    ) -> Self {
        ConsumableNoteRecord::new(
            input_note_record.into(),
            note_consumability.into_iter().map(Into::into).collect(),
        )
    }
}

impl From<NativeNoteConsumability> for NoteConsumability {
    fn from(note_consumability: NativeNoteConsumability) -> Self {
        NoteConsumability::new(
            note_consumability.0.into(),
            match note_consumability.1 {
                NoteRelevance::After(block) => Some(block),
                NoteRelevance::Now => None,
            },
        )
    }
}
