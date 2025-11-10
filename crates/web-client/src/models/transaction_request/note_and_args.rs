use miden_client::note::Note as NativeNote;
use miden_client::transaction::NoteArgs as NativeNoteArgs;
use wasm_bindgen::prelude::*;

use crate::models::miden_arrays::NoteAndArgsArray;
use crate::models::note::Note;
use crate::models::word::Word;

pub type NoteArgs = Word;

#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteAndArgs {
    note: Note,
    args: Option<NoteArgs>,
}

#[wasm_bindgen]
impl NoteAndArgs {
    #[wasm_bindgen(constructor)]
    pub fn new(note: Note, args: Option<NoteArgs>) -> NoteAndArgs {
        NoteAndArgs { note, args }
    }
}

impl From<NoteAndArgs> for (NativeNote, Option<NativeNoteArgs>) {
    fn from(note_and_args: NoteAndArgs) -> Self {
        let native_note: NativeNote = note_and_args.note.into();
        let native_args: Option<NativeNoteArgs> = note_and_args.args.map(Into::into);
        (native_note, native_args)
    }
}

impl From<&NoteAndArgs> for (NativeNote, Option<NativeNoteArgs>) {
    fn from(note_and_args: &NoteAndArgs) -> Self {
        let native_note: NativeNote = note_and_args.note.clone().into();
        let native_args: Option<NativeNoteArgs> = note_and_args.args.clone().map(Into::into);
        (native_note, native_args)
    }
}

impl From<NoteAndArgsArray> for Vec<(NativeNote, Option<NativeNoteArgs>)> {
    fn from(note_and_args_array: NoteAndArgsArray) -> Self {
        note_and_args_array.__inner.into_iter().map(Into::into).collect()
    }
}

impl From<&NoteAndArgsArray> for Vec<(NativeNote, Option<NativeNoteArgs>)> {
    fn from(note_and_args_array: &NoteAndArgsArray) -> Self {
        note_and_args_array.__inner.iter().map(Into::into).collect()
    }
}
