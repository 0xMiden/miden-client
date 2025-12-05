use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::note::{NoteScript, Nullifier};
use miden_client::store::{
    InputNoteRecord,
    InputNoteState,
    NoteFilter,
    OutputNoteRecord,
    OutputNoteState,
    StoreError,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::js_sys::{Array, Promise};

use super::WebStore;
use crate::note::utils::upsert_note_script_tx;
use crate::promise::await_js;

mod js_bindings;
use js_bindings::{
    idxdb_get_input_notes,
    idxdb_get_input_notes_from_ids,
    idxdb_get_input_notes_from_nullifiers,
    idxdb_get_note_script,
    idxdb_get_output_notes,
    idxdb_get_output_notes_from_ids,
    idxdb_get_output_notes_from_nullifiers,
    idxdb_get_unspent_input_note_nullifiers,
};

mod models;
use models::{InputNoteIdxdbObject, NoteScriptIdxdbObject, OutputNoteIdxdbObject};

pub(crate) mod utils;
use utils::{
    parse_input_note_idxdb_object,
    parse_note_script_idxdb_object,
    parse_output_note_idxdb_object,
    upsert_input_note_tx,
};

impl WebStore {
    pub(crate) async fn get_input_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<InputNoteRecord>, StoreError> {
        let input_notes_idxdb: Vec<InputNoteIdxdbObject> =
            await_js(filter.to_input_notes_promise(), "failed to get input notes").await?;

        input_notes_idxdb
            .into_iter()
            .map(parse_input_note_idxdb_object) // Simplified closure
            .collect::<Result<Vec<_>, _>>() // Collect results into a single Result
    }

    pub(crate) async fn get_output_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<OutputNoteRecord>, StoreError> {
        let output_notes_idxdb: Vec<OutputNoteIdxdbObject> =
            await_js(filter.to_output_note_promise(), "failed to get output notes").await?;

        output_notes_idxdb
            .into_iter()
            .map(parse_output_note_idxdb_object) // Simplified closure
            .collect::<Result<Vec<_>, _>>() // Collect results into a single Result
    }

    pub(crate) async fn get_note_script(
        &self,
        script_root: Word,
    ) -> Result<NoteScript, StoreError> {
        let script_root = script_root.to_hex();
        let promise = idxdb_get_note_script(script_root);
        let script_idxdb: NoteScriptIdxdbObject =
            await_js(promise, "failed to get note script").await?;

        parse_note_script_idxdb_object(script_idxdb)
    }

    pub(crate) async fn get_unspent_input_note_nullifiers(
        &self,
    ) -> Result<Vec<Nullifier>, StoreError> {
        let promise = idxdb_get_unspent_input_note_nullifiers();
        let nullifiers_as_str: Vec<String> =
            await_js(promise, "failed to get unspent input note nullifiers").await?;

        nullifiers_as_str
            .into_iter()
            .map(|s| Word::try_from(s).map(Nullifier::new_unchecked).map_err(StoreError::WordError))
            .collect::<Result<Vec<Nullifier>, _>>()
    }

    pub(crate) async fn upsert_input_notes(
        &self,
        notes: &[InputNoteRecord],
    ) -> Result<(), StoreError> {
        for note in notes {
            upsert_input_note_tx(note).await?;
        }

        Ok(())
    }

    pub(crate) async fn upsert_note_scripts(
        &self,
        note_scripts: &[NoteScript],
    ) -> Result<(), StoreError> {
        for note_script in note_scripts {
            upsert_note_script_tx(note_script).await?;
        }

        Ok(())
    }
}

// Provide extension methods for NoteFilter via a local trait
pub(crate) trait NoteFilterExt {
    fn to_input_notes_promise(&self) -> Promise;
    fn to_output_note_promise(&self) -> Promise;
}

impl NoteFilterExt for NoteFilter {
    fn to_input_notes_promise(&self) -> Promise {
        match self {
            NoteFilter::All
            | NoteFilter::Consumed
            | NoteFilter::Committed
            | NoteFilter::Expected
            | NoteFilter::Processing
            | NoteFilter::Unspent
            | NoteFilter::Unverified => {
                let states: Vec<u8> = match self {
                    NoteFilter::All => vec![],
                    NoteFilter::Consumed => vec![
                        InputNoteState::STATE_CONSUMED_AUTHENTICATED_LOCAL,
                        InputNoteState::STATE_CONSUMED_UNAUTHENTICATED_LOCAL,
                        InputNoteState::STATE_CONSUMED_EXTERNAL,
                    ],
                    NoteFilter::Committed => vec![InputNoteState::STATE_COMMITTED],
                    NoteFilter::Expected => vec![InputNoteState::STATE_EXPECTED],
                    NoteFilter::Processing => {
                        vec![
                            InputNoteState::STATE_PROCESSING_AUTHENTICATED,
                            InputNoteState::STATE_PROCESSING_UNAUTHENTICATED,
                        ]
                    },
                    NoteFilter::Unverified => vec![InputNoteState::STATE_UNVERIFIED],
                    NoteFilter::Unspent => vec![
                        InputNoteState::STATE_EXPECTED,
                        InputNoteState::STATE_COMMITTED,
                        InputNoteState::STATE_UNVERIFIED,
                        InputNoteState::STATE_PROCESSING_AUTHENTICATED,
                        InputNoteState::STATE_PROCESSING_UNAUTHENTICATED,
                    ],
                    _ => unreachable!(), // Safety net, should never be reached
                };

                // Assuming `js_fetch_notes` is your JavaScript function that handles simple string
                // filters
                idxdb_get_input_notes(states)
            },
            NoteFilter::List(ids) => {
                let note_ids_as_str: Vec<String> =
                    ids.iter().map(|id| id.as_word().to_string()).collect();
                idxdb_get_input_notes_from_ids(note_ids_as_str)
            },
            NoteFilter::Unique(id) => {
                let note_id_as_str = id.as_word().to_string();
                let note_ids = vec![note_id_as_str];
                idxdb_get_input_notes_from_ids(note_ids)
            },
            NoteFilter::Nullifiers(nullifiers) => {
                let nullifiers_as_str =
                    nullifiers.iter().map(ToString::to_string).collect::<Vec<String>>();

                idxdb_get_input_notes_from_nullifiers(nullifiers_as_str)
            },
        }
    }

    fn to_output_note_promise(&self) -> Promise {
        match self {
            NoteFilter::All
            | NoteFilter::Consumed
            | NoteFilter::Committed
            | NoteFilter::Expected
            | NoteFilter::Unspent => {
                let states = match self {
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
                    _ => unreachable!(), // Safety net, should never be reached
                };

                idxdb_get_output_notes(states)
            },
            NoteFilter::Processing | NoteFilter::Unverified => {
                Promise::resolve(&JsValue::from(Array::new()))
            },
            NoteFilter::List(ids) => {
                let note_ids_as_str: Vec<String> =
                    ids.iter().map(|id| id.as_word().to_string()).collect();
                idxdb_get_output_notes_from_ids(note_ids_as_str)
            },
            NoteFilter::Unique(id) => {
                let note_id_as_str = id.as_word().to_string();
                let note_ids = vec![note_id_as_str];
                idxdb_get_output_notes_from_ids(note_ids)
            },
            NoteFilter::Nullifiers(nullifiers) => {
                let nullifiers_as_str =
                    nullifiers.iter().map(ToString::to_string).collect::<Vec<String>>();

                idxdb_get_output_notes_from_nullifiers(nullifiers_as_str)
            },
        }
    }
}
