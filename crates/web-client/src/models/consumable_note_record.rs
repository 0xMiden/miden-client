use miden_client::note::{
    NoteConsumability as NativeNoteConsumability,
    NoteConsumptionStatus as NativeNoteConsumptionStatus,
};
use miden_client::store::InputNoteRecord as NativeInputNoteRecord;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::input_note_record::InputNoteRecord;

/// Describes if a note could be consumed under a specific conditions: target account state and
/// block height.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteConsumptionStatus(NativeNoteConsumptionStatus);

#[wasm_bindgen]
impl NoteConsumptionStatus {
    /// Constructs a `NoteConsumptionStatus` that is consumable.
    #[wasm_bindgen(js_name = "consumable")]
    pub fn consumable() -> Self {
        Self(NativeNoteConsumptionStatus::Consumable)
    }

    /// Constructs a `NoteConsumptionStatus` that is consumable with authorization.
    #[wasm_bindgen(js_name = "consumableWithAuthorization")]
    pub fn consumable_with_authorization() -> Self {
        Self(NativeNoteConsumptionStatus::ConsumableWithAuthorization)
    }

    /// Constructs a `NoteConsumptionStatus` that is consumable after a specific block height.
    #[wasm_bindgen(js_name = "consumableAfter")]
    pub fn consumable_after(block_height: u32) -> Self {
        Self(NativeNoteConsumptionStatus::ConsumableAfter(block_height.into()))
    }

    /// Constructs a `NoteConsumptionStatus` that is never consumable.
    #[wasm_bindgen(js_name = "neverConsumable")]
    pub fn never_consumable(err: String) -> Self {
        Self(NativeNoteConsumptionStatus::NeverConsumable(err.into()))
    }

    /// Constructs a `NoteConsumptionStatus` that is unconsumable due to conditions.
    #[wasm_bindgen(js_name = "unconsumableConditions")]
    pub fn unconsumable_conditions() -> Self {
        Self(NativeNoteConsumptionStatus::UnconsumableConditions)
    }

    /// Returns the block number at which the note can be consumed.
    /// Returns None if the note is already consumable or never possible
    #[wasm_bindgen(js_name = "consumableAfterBlock")]
    pub fn consumable_after_block(&self) -> Option<u32> {
        match self.0 {
            NativeNoteConsumptionStatus::ConsumableAfter(block_height) => {
                Some(block_height.as_u32())
            },
            _ => None,
        }
    }
}

impl From<NativeNoteConsumptionStatus> for NoteConsumptionStatus {
    fn from(native_note_consumption_status: NativeNoteConsumptionStatus) -> Self {
        NoteConsumptionStatus(native_note_consumption_status)
    }
}

/// Input note record annotated with consumption conditions.
#[derive(Clone)]
#[wasm_bindgen]
pub struct ConsumableNoteRecord {
    input_note_record: InputNoteRecord,
    note_consumability: Vec<NoteConsumability>,
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteConsumability {
    account_id: AccountId,

    // The status of the note, consumable immediately,
    // after a certain block number, etc.
    consumption_status: NoteConsumptionStatus,
}

#[wasm_bindgen]
impl NoteConsumability {
    pub(crate) fn new(
        account_id: AccountId,
        consumption_status: NoteConsumptionStatus,
    ) -> NoteConsumability {
        NoteConsumability { account_id, consumption_status }
    }

    /// Returns the account that can consume the note.
    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    /// Returns the consumption status of the note.
    #[wasm_bindgen(js_name = "consumptionStatus")]
    pub fn consumption_status(&self) -> NoteConsumptionStatus {
        self.consumption_status.clone()
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
        NoteConsumability::new(note_consumability.0.into(), note_consumability.1.into())
    }
}
