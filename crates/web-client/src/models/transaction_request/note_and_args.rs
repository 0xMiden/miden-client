use miden_client::note::Note as NativeNote;
use miden_client::transaction::NoteArgs as NativeNoteArgs;
use crate::prelude::*;

use crate::models::note::Note;
use crate::models::word::Word;

pub type NoteArgs = Word;

#[bindings]
#[derive(Clone)]
pub struct NoteAndArgs {
    note: Note,
    args: Option<NoteArgs>,
}

#[bindings]
impl NoteAndArgs {
    /// Creates a new note/args pair for transaction building.
    #[bindings(constructor)]
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
