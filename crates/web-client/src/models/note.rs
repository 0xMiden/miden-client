use miden_client::note::{
    NoteExecutionHint as NativeNoteExecutionHint,
    NoteInputs as NativeNoteInputs,
    NoteMetadata as NativeNoteMetadata,
    NoteRecipient as NativeNoteRecipient,
    NoteTag as NativeNoteTag,
    WellKnownNote,
};
use miden_lib::note::utils;
use miden_objects::note::Note as NativeNote;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::felt::Felt;
use super::note_assets::NoteAssets;
use super::note_id::NoteId;
use super::note_metadata::NoteMetadata;
use super::note_recipient::NoteRecipient;
use super::note_script::NoteScript;
use super::note_type::NoteType;
use super::word::Word;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Note(NativeNote);

#[wasm_bindgen]
impl Note {
    #[wasm_bindgen(constructor)]
    pub fn new(
        note_assets: &NoteAssets,
        note_metadata: &NoteMetadata,
        note_recipient: &NoteRecipient,
    ) -> Note {
        Note(NativeNote::new(note_assets.into(), note_metadata.into(), note_recipient.into()))
    }

    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    pub fn metadata(&self) -> NoteMetadata {
        (*self.0.metadata()).into()
    }

    pub fn recipient(&self) -> NoteRecipient {
        self.0.recipient().clone().into()
    }

    pub fn assets(&self) -> NoteAssets {
        self.0.assets().clone().into()
    }

    pub fn script(&self) -> NoteScript {
        self.0.script().clone().into()
    }

    #[wasm_bindgen(js_name = "createP2IDNote")]
    pub fn create_p2id_note(
        sender: &AccountId,
        target: &AccountId,
        assets: &NoteAssets,
        note_type: NoteType,
        serial_num: &Word,
        aux: &Felt,
    ) -> Self {
        let recipient = utils::build_p2id_recipient(target.into(), serial_num.into()).unwrap();
        let tag = NativeNoteTag::from_account_id(target.into());

        let metadata = NativeNoteMetadata::new(
            sender.into(),
            note_type.into(),
            tag,
            NativeNoteExecutionHint::always(),
            (*aux).into(),
        )
        .unwrap();

        NativeNote::new(assets.into(), metadata, recipient).into()
    }

    #[wasm_bindgen(js_name = "createP2IDENote")]
    pub fn create_p2ide_note(
        sender: &AccountId,
        target: &AccountId,
        assets: &NoteAssets,
        note_type: NoteType,
        serial_num: &Word,
        recall_height: u32,
        aux: &Felt,
    ) -> Self {
        let note_script = WellKnownNote::P2IDE.script();

        let inputs = NativeNoteInputs::new(vec![
            target.suffix().into(),
            target.prefix().into(),
            recall_height.into(),
        ])
        .unwrap();

        let recipient = NativeNoteRecipient::new(serial_num.into(), note_script, inputs);

        let tag = NativeNoteTag::from_account_id(target.into());

        let metadata = NativeNoteMetadata::new(
            sender.into(),
            note_type.into(),
            tag,
            NativeNoteExecutionHint::always(),
            (*aux).into(),
        )
        .unwrap();

        NativeNote::new(assets.into(), metadata, recipient).into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNote> for Note {
    fn from(note: NativeNote) -> Self {
        Note(note)
    }
}

impl From<&NativeNote> for Note {
    fn from(note: &NativeNote) -> Self {
        Note(note.clone())
    }
}

impl From<Note> for NativeNote {
    fn from(note: Note) -> Self {
        note.0
    }
}

impl From<&Note> for NativeNote {
    fn from(note: &Note) -> Self {
        note.0.clone()
    }
}
