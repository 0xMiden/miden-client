use js_export_macro::js_export;
use miden_client::note::Note as NativeNote;
use miden_client::transaction::NoteArgs as NativeNoteArgs;

use crate::models::miden_arrays::NoteAndArgsArray;
use crate::models::note::Note;
use crate::models::word::Word;

pub type NoteArgs = Word;

#[derive(Clone)]
#[js_export]
pub struct NoteAndArgs {
    note: Note,
    args: Option<NoteArgs>,
}

#[js_export]
impl NoteAndArgs {
    /// Creates a new note/args pair for transaction building.
    #[js_export(constructor)]
    pub fn new(note: &Note, args: Option<NoteArgs>) -> NoteAndArgs {
        NoteAndArgs { note: note.clone(), args }
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
        let items: Vec<NoteAndArgs> = note_and_args_array.into();
        items.into_iter().map(Into::into).collect()
    }
}

impl From<&NoteAndArgsArray> for Vec<(NativeNote, Option<NativeNoteArgs>)> {
    fn from(note_and_args_array: &NoteAndArgsArray) -> Self {
        let items: Vec<NoteAndArgs> = note_and_args_array.into();
        items.into_iter().map(Into::into).collect()
    }
}

impl_napi_from_value!(NoteAndArgs);
