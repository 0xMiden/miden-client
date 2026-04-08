use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::account::AccountId;
use miden_client::note::{BlockNumber, NoteScript, Nullifier};
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

use super::IdxdbStore;
use crate::note::utils::upsert_note_script_tx;
use crate::promise::await_js;

mod js_bindings;
use js_bindings::{
    idxdb_get_input_note_by_offset,
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

impl IdxdbStore {
    pub(crate) async fn get_input_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<InputNoteRecord>, StoreError> {
        let input_notes_idxdb: Vec<InputNoteIdxdbObject> =
            await_js(filter.to_input_notes_promise(self.db_id()), "failed to get input notes")
                .await?;

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
            await_js(filter.to_output_note_promise(self.db_id()), "failed to get output notes")
                .await?;

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
        let promise = idxdb_get_note_script(self.db_id(), script_root);
        let script_idxdb: NoteScriptIdxdbObject =
            await_js(promise, "failed to get note script").await?;

        parse_note_script_idxdb_object(script_idxdb)
    }

    pub(crate) async fn get_unspent_input_note_nullifiers(
        &self,
    ) -> Result<Vec<Nullifier>, StoreError> {
        let promise = idxdb_get_unspent_input_note_nullifiers(self.db_id());
        let nullifiers_as_str: Vec<String> =
            await_js(promise, "failed to get unspent input note nullifiers").await?;

        nullifiers_as_str
            .into_iter()
            .map(|s| Word::try_from(s).map(Nullifier::from_raw).map_err(StoreError::WordError))
            .collect::<Result<Vec<Nullifier>, _>>()
    }

    pub(crate) async fn get_input_note_by_offset(
        &self,
        filter: NoteFilter,
        consumer: AccountId,
        block_start: Option<BlockNumber>,
        block_end: Option<BlockNumber>,
        offset: u32,
    ) -> Result<Option<InputNoteRecord>, StoreError> {
        let states = input_note_state_discriminants(&filter).ok_or_else(|| {
            StoreError::QueryError(
                "get_input_note_by_offset only supports state-based filters".to_string(),
            )
        })?;
        let consumer_hex = consumer.to_hex();
        let promise = idxdb_get_input_note_by_offset(
            self.db_id(),
            states,
            consumer_hex,
            block_start.map(|b| b.as_u32()),
            block_end.map(|b| b.as_u32()),
            offset,
        );

        let notes: Vec<InputNoteIdxdbObject> =
            await_js(promise, "failed to get input note by offset").await?;

        notes.into_iter().next().map(parse_input_note_idxdb_object).transpose()
    }

    pub(crate) async fn upsert_input_notes(
        &self,
        notes: &[InputNoteRecord],
    ) -> Result<(), StoreError> {
        for note in notes {
            upsert_input_note_tx(self.db_id(), note).await?;
        }

        Ok(())
    }

    pub(crate) async fn upsert_note_scripts(
        &self,
        note_scripts: &[NoteScript],
    ) -> Result<(), StoreError> {
        for note_script in note_scripts {
            upsert_note_script_tx(self.db_id(), note_script).await?;
        }

        Ok(())
    }
}

/// Returns the input note state discriminants for a state-based [`NoteFilter`], or `None` for
/// filters that operate on IDs/nullifiers.
fn input_note_state_discriminants(filter: &NoteFilter) -> Option<Vec<u8>> {
    match filter {
        NoteFilter::All => Some(vec![]),
        NoteFilter::Consumed => Some(vec![
            InputNoteState::STATE_CONSUMED_AUTHENTICATED_LOCAL,
            InputNoteState::STATE_CONSUMED_UNAUTHENTICATED_LOCAL,
            InputNoteState::STATE_CONSUMED_EXTERNAL,
        ]),
        NoteFilter::Committed => Some(vec![InputNoteState::STATE_COMMITTED]),
        NoteFilter::Expected => Some(vec![InputNoteState::STATE_EXPECTED]),
        NoteFilter::Processing => Some(vec![
            InputNoteState::STATE_PROCESSING_AUTHENTICATED,
            InputNoteState::STATE_PROCESSING_UNAUTHENTICATED,
        ]),
        NoteFilter::Unverified => Some(vec![InputNoteState::STATE_UNVERIFIED]),
        NoteFilter::Unspent => Some(vec![
            InputNoteState::STATE_EXPECTED,
            InputNoteState::STATE_COMMITTED,
            InputNoteState::STATE_UNVERIFIED,
            InputNoteState::STATE_PROCESSING_AUTHENTICATED,
            InputNoteState::STATE_PROCESSING_UNAUTHENTICATED,
        ]),
        NoteFilter::List(_) | NoteFilter::Unique(_) | NoteFilter::Nullifiers(_) => None,
    }
}

// Provide extension methods for NoteFilter via a local trait
pub(crate) trait NoteFilterExt {
    fn to_input_notes_promise(&self, db_id: &str) -> Promise;
    fn to_output_note_promise(&self, db_id: &str) -> Promise;
}

impl NoteFilterExt for NoteFilter {
    fn to_input_notes_promise(&self, db_id: &str) -> Promise {
        match self {
            NoteFilter::All
            | NoteFilter::Consumed
            | NoteFilter::Committed
            | NoteFilter::Expected
            | NoteFilter::Processing
            | NoteFilter::Unspent
            | NoteFilter::Unverified => {
                let states = input_note_state_discriminants(self)
                    .expect("state-based filters always return Some");
                idxdb_get_input_notes(db_id, states)
            },
            NoteFilter::List(ids) => {
                let note_ids_as_str: Vec<String> =
                    ids.iter().map(|id| id.as_word().to_string()).collect();
                idxdb_get_input_notes_from_ids(db_id, note_ids_as_str)
            },
            NoteFilter::Unique(id) => {
                let note_id_as_str = id.as_word().to_string();
                let note_ids = vec![note_id_as_str];
                idxdb_get_input_notes_from_ids(db_id, note_ids)
            },
            NoteFilter::Nullifiers(nullifiers) => {
                let nullifiers_as_str =
                    nullifiers.iter().map(ToString::to_string).collect::<Vec<String>>();

                idxdb_get_input_notes_from_nullifiers(db_id, nullifiers_as_str)
            },
        }
    }

    fn to_output_note_promise(&self, db_id: &str) -> Promise {
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
                        OutputNoteState::STATE_EXPECTED_PARTIAL,
                        OutputNoteState::STATE_EXPECTED_FULL,
                        OutputNoteState::STATE_COMMITTED_PARTIAL,
                        OutputNoteState::STATE_COMMITTED_FULL,
                    ],
                    _ => unreachable!(), // Safety net, should never be reached
                };

                idxdb_get_output_notes(db_id, states)
            },
            NoteFilter::Processing | NoteFilter::Unverified => {
                Promise::resolve(&JsValue::from(Array::new()))
            },
            NoteFilter::List(ids) => {
                let note_ids_as_str: Vec<String> =
                    ids.iter().map(|id| id.as_word().to_string()).collect();
                idxdb_get_output_notes_from_ids(db_id, note_ids_as_str)
            },
            NoteFilter::Unique(id) => {
                let note_id_as_str = id.as_word().to_string();
                let note_ids = vec![note_id_as_str];
                idxdb_get_output_notes_from_ids(db_id, note_ids)
            },
            NoteFilter::Nullifiers(nullifiers) => {
                let nullifiers_as_str =
                    nullifiers.iter().map(ToString::to_string).collect::<Vec<String>>();

                idxdb_get_output_notes_from_nullifiers(db_id, nullifiers_as_str)
            },
        }
    }
}
