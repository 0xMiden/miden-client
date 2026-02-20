use miden_client::asset::Asset as NativeAsset;
use miden_client::block::BlockNumber as NativeBlockNumber;
use miden_client::crypto::RpoRandomCoin;
use miden_client::note::{
    Note as NativeNote,
    NoteAssets as NativeNoteAssets,
    create_p2id_note,
    create_p2ide_note,
};
use miden_client::{Felt as NativeFelt, Word as NativeWord};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use crate::prelude::*;

use super::account_id::AccountId;
use super::note_assets::NoteAssets;
use super::note_attachment::NoteAttachment;
use super::note_id::NoteId;
use super::note_metadata::NoteMetadata;
use super::note_recipient::NoteRecipient;
use super::note_script::NoteScript;
use super::note_type::NoteType;
use super::word::Word;

/// A note bundles public metadata with private details: assets, script, inputs, and a serial number
/// grouped into a recipient. The public identifier (`NoteId`) commits to those
/// details, while the nullifier stays hidden until the note is consumed. Assets move by
/// transferring them into the note; the script and inputs define how and when consumption can
/// happen. See `NoteRecipient` for the shape of the recipient data.
#[bindings]
#[derive(Clone)]
pub struct Note(pub(crate) NativeNote);

#[bindings]
impl Note {
    #[bindings(constructor)]
    pub fn new(
        note_assets: &NoteAssets,
        note_metadata: &NoteMetadata,
        note_recipient: &NoteRecipient,
    ) -> Note {
        Note(NativeNote::new(note_assets.into(), note_metadata.into(), note_recipient.into()))
    }

    pub fn serialize(&self) -> JsBytes {
        platform::serialize_to_bytes(&self.0)
    }

    #[bindings(factory)]
    pub fn deserialize(bytes: JsBytes) -> JsResult<Note> {
        platform::deserialize_from_bytes::<NativeNote>(&bytes).map(Note)
    }

    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    pub fn metadata(&self) -> NoteMetadata {
        self.0.metadata().clone().into()
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

    pub fn nullifier(&self) -> Word {
        let nullifier = self.0.nullifier();
        let elements: [miden_client::Felt; 4] =
            nullifier.as_elements().try_into().expect("nullifier has 4 elements");
        let native_word: NativeWord = NativeWord::from(&elements);
        native_word.into()
    }

    #[bindings(factory, js_name = "createP2IDNote")]
    pub fn create_p2id_note(
        sender: &AccountId,
        target: &AccountId,
        assets: &NoteAssets,
        note_type: NoteType,
        attachment: &NoteAttachment,
    ) -> JsResult<Self> {
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
            attachment.into(),
            &mut rng,
        )
        .map_err(|err| platform::error_with_context(err, "create p2id note"))?;

        Ok(native_note.into())
    }

    #[bindings(factory, js_name = "createP2IDENote")]
    pub fn create_p2ide_note(
        sender: &AccountId,
        target: &AccountId,
        assets: &NoteAssets,
        reclaim_height: Option<u32>,
        timelock_height: Option<u32>,
        note_type: NoteType,
        attachment: &NoteAttachment,
    ) -> JsResult<Self> {
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
            attachment.into(),
            &mut rng,
        )
        .map_err(|err| platform::error_with_context(err, "create p2ide note"))?;

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
