use miden_client::note::NoteMetadata as NativeNoteMetadata;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::felt::Felt;
use super::note_execution_hint::NoteExecutionHint;
use super::note_tag::NoteTag;
use super::note_type::NoteType;

/// Metadata associated with a note.
///
/// Note type and tag must be internally consistent according to the following rules:
///
/// - For private and encrypted notes, the two most significant bits of the tag must be `0b11`.
/// - For public notes, the two most significant bits of the tag can be set to any value.
///
/// # Word layout & validity
///
/// `NoteMetadata` can be encoded into a `Word` with the following layout:
///
/// ```text
/// 1st felt: [sender_id_prefix (64 bits)]
/// 2nd felt: [sender_id_suffix (56 bits) | note_type (2 bits) | note_execution_hint_tag (6 bits)]
/// 3rd felt: [note_execution_hint_payload (32 bits) | note_tag (32 bits)]
/// 4th felt: [aux (64 bits)]
/// ```
///
/// The rationale for the above layout is to ensure the validity of each felt:
/// - 1st felt: Is equivalent to the prefix of the account ID so it inherits its validity.
/// - 2nd felt: The lower 8 bits of the account ID suffix are `0` by construction, so that they can
///   be overwritten with other data. The suffix is designed such that it retains its felt validity
///   even if all of its lower 8 bits are be set to `1`. This is because the most significant bit is
///   always zero.
/// - 3rd felt: The note execution hint payload must contain at least one `0` bit in its encoding,
///   so the upper 32 bits of the felt will contain at least one `0` bit making the entire felt
///   valid.
/// - 4th felt: The `aux` value must be a felt itself.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteMetadata(NativeNoteMetadata);

#[wasm_bindgen]
impl NoteMetadata {
    /// Creates metadata for a note.
    #[wasm_bindgen(constructor)]
    pub fn new(
        sender: &AccountId,
        note_type: NoteType,
        note_tag: &NoteTag,
        note_execution_hint: &NoteExecutionHint,
        aux: Option<Felt>, // Create an OptionFelt type so user has choice to consume or not
    ) -> NoteMetadata {
        let native_note_metadata = NativeNoteMetadata::new(
            sender.into(),
            note_type.into(),
            note_tag.into(),
            note_execution_hint.into(),
            aux.map_or(miden_client::Felt::default(), Into::into),
        )
        .unwrap();
        NoteMetadata(native_note_metadata)
    }

    /// Returns the account that created the note.
    pub fn sender(&self) -> AccountId {
        self.0.sender().into()
    }

    /// Returns the tag associated with the note.
    pub fn tag(&self) -> NoteTag {
        self.0.tag().into()
    }

    /// Returns whether the note is private, encrypted, or public.
    #[wasm_bindgen(js_name = "noteType")]
    pub fn note_type(&self) -> NoteType {
        self.0.note_type().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteMetadata> for NoteMetadata {
    fn from(native_note_metadata: NativeNoteMetadata) -> Self {
        NoteMetadata(native_note_metadata)
    }
}

impl From<&NativeNoteMetadata> for NoteMetadata {
    fn from(native_note_metadata: &NativeNoteMetadata) -> Self {
        NoteMetadata(*native_note_metadata)
    }
}

impl From<NoteMetadata> for NativeNoteMetadata {
    fn from(note_metadata: NoteMetadata) -> Self {
        note_metadata.0
    }
}

impl From<&NoteMetadata> for NativeNoteMetadata {
    fn from(note_metadata: &NoteMetadata) -> Self {
        note_metadata.0
    }
}
