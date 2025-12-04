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
use super::word::Word;
use crate::js_error_with_context;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Notes consist of note metadata and details. Note metadata is always public, but details may be
/// either public, encrypted, or private, depending on the note type. Note details consist of note
/// assets, script, inputs, and a serial number, the three latter grouped into a recipient object.
///
/// Note details can be reduced to two unique identifiers: [`NoteId`] and `Nullifier`. The former is
/// publicly associated with a note, while the latter is known only to entities which have access to
/// full note details.
///
/// Fungible and non-fungible asset transfers are done by moving assets to the note's assets. The
/// note's script determines the conditions required for the note consumption, i.e. the target
/// account of a P2ID or conditions of a SWAP, and the effects of the note. The serial number has a
/// double duty of preventing double spend, and providing unlikability to the consumer of a note.
/// The note's inputs allow for customization of its script.
///
/// To create a note, the kernel does not require all the information above, a user can create a
/// note only with the commitment to the script, inputs, the serial number (i.e., the recipient),
/// and the kernel only verifies the source account has the assets necessary for the note creation.
/// See [`NoteRecipient`] for more details.
#[wasm_bindgen]
#[derive(Clone)]
pub struct Note(NativeNote);

#[wasm_bindgen]
impl Note {
    /// Creates a new note from the provided assets, metadata, and recipient.
    #[wasm_bindgen(constructor)]
    pub fn new(
        note_assets: &NoteAssets,
        note_metadata: &NoteMetadata,
        note_recipient: &NoteRecipient,
    ) -> Note {
        Note(NativeNote::new(note_assets.into(), note_metadata.into(), note_recipient.into()))
    }

    /// Serializes the note into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a note from its byte representation.
    pub fn deserialize(bytes: &Uint8Array) -> Result<Note, JsValue> {
        deserialize_from_uint8array::<NativeNote>(bytes).map(Note)
    }

    /// Returns the unique identifier of the note.
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the commitment to the note ID and metadata.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns the public metadata associated with the note.
    pub fn metadata(&self) -> NoteMetadata {
        (*self.0.metadata()).into()
    }

    /// Returns the recipient who can consume this note.
    pub fn recipient(&self) -> NoteRecipient {
        self.0.recipient().clone().into()
    }

    /// Returns the assets locked inside the note.
    pub fn assets(&self) -> NoteAssets {
        self.0.assets().clone().into()
    }

    /// Returns the script that guards the note.
    pub fn script(&self) -> NoteScript {
        self.0.script().clone().into()
    }

    /// Builds a standard P2ID note that targets the specified account.
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

    /// Builds a P2IDE note that can be reclaimed or timelocked based on block heights.
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
