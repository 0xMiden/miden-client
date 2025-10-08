use miden_client::notes::NoteFile as NativeNoteFile;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use super::note::Note;

#[wasm_bindgen(inspectable)]
pub struct NoteFile {
    pub(crate) inner: NativeNoteFile,
}

#[wasm_bindgen]
impl NoteFile {
    #[wasm_bindgen(js_name = noteType)]
    pub fn note_type(&self) -> String {
        match &self.inner {
            NativeNoteFile::NoteId(_) => "NoteId".to_owned(),
            NativeNoteFile::NoteDetails { .. } => "NoteDetails".to_owned(),
            NativeNoteFile::NoteWithProof(_, _) => "NoteWithProof".to_owned(),
        }
    }
}

impl From<NativeNoteFile> for NoteFile {
    fn from(note_file: NativeNoteFile) -> Self {
        NoteFile { inner: note_file }
    }
}

impl From<NoteFile> for NativeNoteFile {
    fn from(note_file: NoteFile) -> Self {
        note_file.inner
    }
}
