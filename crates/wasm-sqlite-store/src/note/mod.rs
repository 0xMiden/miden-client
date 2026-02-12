use alloc::string::{String, ToString};
use alloc::vec::Vec;

use chrono::Utc;
use miden_client::Word;
use miden_client::note::{
    NoteAssets,
    NoteDetails,
    NoteInputs,
    NoteMetadata,
    NoteRecipient,
    NoteScript,
    NoteUpdateTracker,
    Nullifier,
};
use miden_client::store::{
    InputNoteRecord,
    InputNoteState,
    NoteFilter,
    OutputNoteRecord,
    OutputNoteState,
    StoreError,
};
use miden_client::utils::{Deserializable, Serializable};
use wasm_bindgen::JsValue;

use super::WasmSqliteStore;

mod js_bindings;
use js_bindings::{
    js_get_input_notes,
    js_get_input_notes_from_ids,
    js_get_input_notes_from_nullifiers,
    js_get_note_script,
    js_get_output_notes,
    js_get_output_notes_from_ids,
    js_get_output_notes_from_nullifiers,
    js_get_unspent_input_note_nullifiers,
    js_upsert_input_note,
    js_upsert_note_script,
    js_upsert_output_note,
};

mod models;
use models::{InputNoteObject, NoteScriptObject, OutputNoteObject};

impl WasmSqliteStore {
    #[allow(clippy::unused_async)]
    pub(crate) async fn get_input_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<InputNoteRecord>, StoreError> {
        let js_value = self.get_input_notes_js(&filter);
        let notes: Vec<InputNoteObject> =
            serde_wasm_bindgen::from_value(js_value).map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize input notes: {err:?}"))
            })?;

        notes.into_iter().map(parse_input_note_object).collect::<Result<Vec<_>, _>>()
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_output_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<OutputNoteRecord>, StoreError> {
        let js_value = self.get_output_notes_js(&filter);
        let notes: Vec<OutputNoteObject> =
            serde_wasm_bindgen::from_value(js_value).map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize output notes: {err:?}"))
            })?;

        notes.into_iter().map(parse_output_note_object).collect::<Result<Vec<_>, _>>()
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_note_script(
        &self,
        script_root: Word,
    ) -> Result<NoteScript, StoreError> {
        let script_root_hex = script_root.to_hex();
        let js_value = js_get_note_script(self.db_id(), script_root_hex);
        if js_value.is_null() || js_value.is_undefined() {
            return Err(StoreError::DatabaseError("note script not found".to_string()));
        }

        let script_obj: NoteScriptObject =
            serde_wasm_bindgen::from_value(js_value).map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize note script: {err:?}"))
            })?;

        NoteScript::read_from_bytes(&script_obj.serialized_note_script)
            .map_err(|e| StoreError::DatabaseError(e.to_string()))
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_unspent_input_note_nullifiers(
        &self,
    ) -> Result<Vec<Nullifier>, StoreError> {
        let js_value = js_get_unspent_input_note_nullifiers(self.db_id());
        let nullifiers_as_str: Vec<String> =
            serde_wasm_bindgen::from_value(js_value).map_err(|err| {
                StoreError::DatabaseError(format!(
                    "failed to deserialize unspent nullifiers: {err:?}"
                ))
            })?;

        nullifiers_as_str
            .into_iter()
            .map(|s| Word::try_from(s).map(Nullifier::from_raw).map_err(StoreError::WordError))
            .collect::<Result<Vec<Nullifier>, _>>()
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn upsert_input_notes(
        &self,
        notes: &[InputNoteRecord],
    ) -> Result<(), StoreError> {
        for note in notes {
            upsert_input_note(self.db_id(), note);
        }
        Ok(())
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn upsert_note_scripts(
        &self,
        note_scripts: &[NoteScript],
    ) -> Result<(), StoreError> {
        for note_script in note_scripts {
            let script_bytes = note_script.to_bytes();
            let script_root = note_script.root().to_hex();
            js_upsert_note_script(self.db_id(), script_root, script_bytes);
        }
        Ok(())
    }

    fn get_input_notes_js(&self, filter: &NoteFilter) -> JsValue {
        match filter {
            NoteFilter::All
            | NoteFilter::Consumed
            | NoteFilter::Committed
            | NoteFilter::Expected
            | NoteFilter::Processing
            | NoteFilter::Unspent
            | NoteFilter::Unverified => {
                let states: Vec<u8> = match filter {
                    NoteFilter::All => vec![],
                    NoteFilter::Consumed => vec![
                        InputNoteState::STATE_CONSUMED_AUTHENTICATED_LOCAL,
                        InputNoteState::STATE_CONSUMED_UNAUTHENTICATED_LOCAL,
                        InputNoteState::STATE_CONSUMED_EXTERNAL,
                    ],
                    NoteFilter::Committed => vec![InputNoteState::STATE_COMMITTED],
                    NoteFilter::Expected => vec![InputNoteState::STATE_EXPECTED],
                    NoteFilter::Processing => vec![
                        InputNoteState::STATE_PROCESSING_AUTHENTICATED,
                        InputNoteState::STATE_PROCESSING_UNAUTHENTICATED,
                    ],
                    NoteFilter::Unverified => vec![InputNoteState::STATE_UNVERIFIED],
                    NoteFilter::Unspent => vec![
                        InputNoteState::STATE_EXPECTED,
                        InputNoteState::STATE_COMMITTED,
                        InputNoteState::STATE_UNVERIFIED,
                        InputNoteState::STATE_PROCESSING_AUTHENTICATED,
                        InputNoteState::STATE_PROCESSING_UNAUTHENTICATED,
                    ],
                    _ => unreachable!(),
                };
                js_get_input_notes(self.db_id(), states)
            },
            NoteFilter::List(ids) => {
                let note_ids_as_str: Vec<String> =
                    ids.iter().map(|id| id.as_word().to_string()).collect();
                js_get_input_notes_from_ids(self.db_id(), note_ids_as_str)
            },
            NoteFilter::Unique(id) => {
                let note_id_as_str = id.as_word().to_string();
                js_get_input_notes_from_ids(self.db_id(), vec![note_id_as_str])
            },
            NoteFilter::Nullifiers(nullifiers) => {
                let nullifiers_as_str =
                    nullifiers.iter().map(ToString::to_string).collect::<Vec<String>>();
                js_get_input_notes_from_nullifiers(self.db_id(), nullifiers_as_str)
            },
        }
    }

    fn get_output_notes_js(&self, filter: &NoteFilter) -> JsValue {
        match filter {
            NoteFilter::All
            | NoteFilter::Consumed
            | NoteFilter::Committed
            | NoteFilter::Expected
            | NoteFilter::Unspent => {
                let states = match filter {
                    NoteFilter::All => vec![],
                    NoteFilter::Consumed => vec![OutputNoteState::STATE_CONSUMED],
                    NoteFilter::Committed => vec![
                        OutputNoteState::STATE_COMMITTED_FULL,
                        OutputNoteState::STATE_COMMITTED_PARTIAL,
                    ],
                    NoteFilter::Expected => vec![
                        OutputNoteState::STATE_EXPECTED_FULL,
                        OutputNoteState::STATE_EXPECTED_PARTIAL,
                    ],
                    NoteFilter::Unspent => vec![
                        OutputNoteState::STATE_EXPECTED_FULL,
                        OutputNoteState::STATE_COMMITTED_FULL,
                    ],
                    _ => unreachable!(),
                };
                js_get_output_notes(self.db_id(), states)
            },
            NoteFilter::Processing | NoteFilter::Unverified => {
                // These filters don't apply to output notes - return empty
                serde_wasm_bindgen::to_value::<Vec<OutputNoteObject>>(&vec![])
                    .expect("serialization should succeed")
            },
            NoteFilter::List(ids) => {
                let note_ids_as_str: Vec<String> =
                    ids.iter().map(|id| id.as_word().to_string()).collect();
                js_get_output_notes_from_ids(self.db_id(), note_ids_as_str)
            },
            NoteFilter::Unique(id) => {
                let note_id_as_str = id.as_word().to_string();
                js_get_output_notes_from_ids(self.db_id(), vec![note_id_as_str])
            },
            NoteFilter::Nullifiers(nullifiers) => {
                let nullifiers_as_str =
                    nullifiers.iter().map(ToString::to_string).collect::<Vec<String>>();
                js_get_output_notes_from_nullifiers(self.db_id(), nullifiers_as_str)
            },
        }
    }
}

fn upsert_input_note(db_id: &str, note: &InputNoteRecord) {
    let note_id = note.id().to_hex();
    let note_assets = note.assets().to_bytes();
    let details = note.details();
    let serial_number = details.serial_num().to_bytes();
    let inputs = details.inputs().to_bytes();
    let nullifier = details.nullifier().to_hex();
    let recipient = details.recipient();
    let note_script: Vec<u8> = recipient.script().to_bytes();
    let note_script_root = recipient.script().root().to_hex();
    let state_discriminant = note.state().discriminant();
    let state = note.state().to_bytes();
    let created_at = Utc::now().timestamp().to_string();

    js_upsert_input_note(
        db_id,
        note_id,
        note_assets,
        serial_number,
        inputs,
        note_script_root,
        note_script,
        nullifier,
        created_at,
        state_discriminant,
        state,
    );
}

pub(crate) fn upsert_output_note(db_id: &str, note: &OutputNoteRecord) {
    let note_id = note.id().to_hex();
    let note_assets = note.assets().to_bytes();
    let recipient_digest = note.recipient_digest().to_hex();
    let metadata = note.metadata().to_bytes();
    let nullifier = note.nullifier().map(|n| n.to_hex());
    let state_discriminant = note.state().discriminant();
    let state = note.state().to_bytes();
    let expected_height = note.expected_height().as_u32();

    js_upsert_output_note(
        db_id,
        note_id,
        note_assets,
        recipient_digest,
        metadata,
        nullifier,
        expected_height,
        state_discriminant,
        state,
    );
}

pub(crate) fn apply_note_updates(db_id: &str, note_updates: &NoteUpdateTracker) {
    for input_note in note_updates.updated_input_notes() {
        upsert_input_note(db_id, input_note.inner());
    }
    for output_note in note_updates.updated_output_notes() {
        upsert_output_note(db_id, output_note.inner());
    }
}

fn parse_input_note_object(note: InputNoteObject) -> Result<InputNoteRecord, StoreError> {
    let InputNoteObject {
        assets,
        serial_number,
        inputs,
        serialized_note_script,
        state,
        created_at,
    } = note;

    let assets = NoteAssets::read_from_bytes(&assets)?;
    let serial_number = Word::read_from_bytes(&serial_number)?;
    let script = NoteScript::read_from_bytes(&serialized_note_script)?;
    let inputs = NoteInputs::read_from_bytes(&inputs)?;
    let recipient = NoteRecipient::new(serial_number, script, inputs);
    let details = NoteDetails::new(assets, recipient);
    let state = InputNoteState::read_from_bytes(&state)?;
    let created_at = created_at
        .parse::<u64>()
        .map_err(|_| StoreError::QueryError("Failed to parse created_at timestamp".to_string()))?;

    Ok(InputNoteRecord::new(details, Some(created_at), state))
}

fn parse_output_note_object(note: OutputNoteObject) -> Result<OutputNoteRecord, StoreError> {
    let note_metadata = NoteMetadata::read_from_bytes(&note.metadata)?;
    let note_assets = NoteAssets::read_from_bytes(&note.assets)?;
    let recipient = Word::try_from(note.recipient_digest)?;
    let state = OutputNoteState::read_from_bytes(&note.state)?;

    Ok(OutputNoteRecord::new(
        recipient,
        note_assets,
        note_metadata,
        state,
        note.expected_height.into(),
    ))
}
