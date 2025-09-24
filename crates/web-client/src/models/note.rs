use miden_client::asset::Asset as NativeAsset;
use miden_client::crypto::RpoRandomCoin;
use miden_client::note::{
    Note as NativeNote,
    NoteAssets as NativeNoteAssets,
    create_p2id_note,
    create_p2ide_note,
};
use miden_client::{BlockNumber as NativeBlockNumber, Felt as NativeFelt};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use super::account_id::AccountId;
use super::felt::Felt;
use super::note_assets::NoteAssets;
use super::note_id::NoteId;
use super::note_metadata::NoteMetadata;
use super::note_recipient::NoteRecipient;
use super::note_script::NoteScript;
use super::note_type::NoteType;
use crate::js_error_with_context;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

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

    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<Note, JsValue> {
        deserialize_from_uint8array::<NativeNote>(bytes).map(Note)
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
        aux: &Felt,
    ) -> Result<Self, JsValue> {
        let mut rng = StdRng::from_os_rng();
        let coin_seed: [u64; 4] = rng.random();
        let mut rng = RpoRandomCoin::new(coin_seed.map(NativeFelt::new).into());

        let native_note_assets: NativeNoteAssets = assets.into();
        let native_assets: Vec<NativeAsset> = native_note_assets.iter().copied().collect();

        let native_note = create_p2id_note(
            sender.into(),
            target.into(),
            native_assets,
            note_type.into(),
            (*aux).into(),
            &mut rng,
        )
        .map_err(|err| js_error_with_context(err, "create p2id note"))?;

        Ok(native_note.into())
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
    ) -> Result<Self, JsValue> {
        let mut rng = StdRng::from_os_rng();
        let coin_seed: [u64; 4] = rng.random();
        let mut rng = RpoRandomCoin::new(coin_seed.map(NativeFelt::new).into());

        let native_note_assets: NativeNoteAssets = assets.into();
        let native_assets: Vec<NativeAsset> = native_note_assets.iter().copied().collect();

        let native_note = create_p2ide_note(
            sender.into(),
            target.into(),
            native_assets,
            reclaim_height.map(NativeBlockNumber::from),
            timelock_height.map(NativeBlockNumber::from),
            note_type.into(),
            (*aux).into(),
            &mut rng,
        )
        .map_err(|err| js_error_with_context(err, "create p2ide note"))?;

        Ok(native_note.into())
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
