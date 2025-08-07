use miden_client::note::{
    NoteMetadata as NativeNoteMetadata,
    NoteTag as NativeNoteTag,
};
use miden_lib::note::utils;
use miden_objects::{block::BlockNumber as NativeBlockNumber, crypto::rand::{FeltRng, RpoRandomCoin}, note::{Note as NativeNote, NoteExecutionHint as NativeNoteExecutionHint}};
use rand::{Rng, SeedableRng, rngs::StdRng};
use wasm_bindgen::prelude::*;

use super::{
    account_id::AccountId, felt::Felt, note_assets::NoteAssets, note_id::NoteId,
    note_metadata::NoteMetadata, note_recipient::NoteRecipient, note_type::NoteType, word::Word,
};

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

    #[wasm_bindgen(js_name = "createP2IDNote")]
    pub fn create_p2id_note(
        sender: &AccountId,
        target: &AccountId,
        assets: &NoteAssets,
        note_type: NoteType,
        aux: &Felt,
    ) -> Self {
        let mut rng = StdRng::from_os_rng();
        let coin_seed: [u64; 4] = rng.random();
        let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));

        let serial_num = rng.draw_word();
        let recipient = utils::build_p2id_recipient(target.into(), serial_num).unwrap();
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
        reclaim_height: Option<u32>,
        timelock_height: Option<u32>,
        note_type: NoteType,
        aux: &Felt,
    ) -> Self {
        let mut rng = StdRng::from_os_rng();
        let coin_seed: [u64; 4] = rng.random();
        let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));

        let serial_num = rng.draw_word();
        let recipient = utils::build_p2ide_recipient(
            target.into(), 
            reclaim_height.map(NativeBlockNumber::from), 
            timelock_height.map(NativeBlockNumber::from), 
            serial_num
        )
        .unwrap();

        let tag = NativeNoteTag::from_account_id(target.into());
        let execution_hint = match timelock_height {
            Some(height) => NativeNoteExecutionHint::after_block(height.into()).unwrap(),
            None => NativeNoteExecutionHint::always(),
        };

        let metadata = NativeNoteMetadata::new(
            sender.into(),
            note_type.into(),
            tag,
            execution_hint,
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
