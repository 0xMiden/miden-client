use miden_client::note::NoteId as NativeNoteId;
use miden_client::transaction::NoteArgs as NativeNoteArgs;
use wasm_bindgen::prelude::*;

use crate::models::miden_arrays::NoteIdAndArgsArray;
use crate::models::note_id::NoteId;
use crate::models::transaction_request::note_and_args::NoteArgs;

/// Note ID paired with optional arguments for inclusion in a transaction request.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteIdAndArgs {
    note_id: NoteId,
    args: Option<NoteArgs>,
}

#[wasm_bindgen]
impl NoteIdAndArgs {
    /// Creates a new NoteId/args pair.
    #[wasm_bindgen(constructor)]
    pub fn new(note_id: NoteId, args: Option<NoteArgs>) -> NoteIdAndArgs {
        NoteIdAndArgs { note_id, args }
    }
}

impl From<NoteIdAndArgs> for (NativeNoteId, Option<NativeNoteArgs>) {
    fn from(note_id_and_args: NoteIdAndArgs) -> Self {
        let native_note_id: NativeNoteId = note_id_and_args.note_id.into();
        let native_args: Option<NativeNoteArgs> = note_id_and_args.args.map(Into::into);
        (native_note_id, native_args)
    }
}

impl From<&NoteIdAndArgs> for (NativeNoteId, Option<NativeNoteArgs>) {
    fn from(note_id_and_args: &NoteIdAndArgs) -> Self {
        let native_note_id: NativeNoteId = note_id_and_args.note_id.into();
        let native_args: Option<NativeNoteArgs> =
            note_id_and_args.args.clone().map(|args| args.clone().into());
        (native_note_id, native_args)
    }
}

impl From<NoteIdAndArgsArray> for Vec<(NativeNoteId, Option<NativeNoteArgs>)> {
    fn from(note_id_and_args_array: NoteIdAndArgsArray) -> Self {
        note_id_and_args_array.__inner.into_iter().map(Into::into).collect()
    }
}

impl From<&NoteIdAndArgsArray> for Vec<(NativeNoteId, Option<NativeNoteArgs>)> {
    fn from(note_id_and_args_array: &NoteIdAndArgsArray) -> Self {
        note_id_and_args_array.__inner.iter().map(Into::into).collect()
    }
}
