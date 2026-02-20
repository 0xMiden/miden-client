use miden_client::note::NoteId as NativeNoteId;
use crate::prelude::*;
use super::word::Word;

/// Returns a unique identifier of a note, which is simultaneously a commitment to the note.
///
/// Note ID is computed as:
///
/// > `hash(recipient, asset_commitment)`
///
/// where `recipient` is defined as:
///
/// > `hash(hash(hash(serial_num, ZERO), script_root), input_commitment)`
///
/// This achieves the following properties:
/// - Every note can be reduced to a single unique ID.
/// - To compute a note ID, we do not need to know the note's `serial_num`. Knowing the hash of the
///   `serial_num` (as well as script root, input commitment, and note assets) is sufficient.
#[bindings]
#[derive(Clone, Copy)]
pub struct NoteId(pub(crate) NativeNoteId);

#[bindings]
impl NoteId {
    #[bindings(constructor)]
    pub fn new(recipient_digest: &Word, asset_commitment_digest: &Word) -> NoteId {
        NoteId(NativeNoteId::new(recipient_digest.into(), asset_commitment_digest.into()))
    }

    #[bindings(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string_js(&self) -> String {
        self.0.to_string()
    }

    #[bindings(factory)]
    pub fn from_hex(hex: String) -> JsResult<NoteId> {
        let native_note_id = NativeNoteId::try_from_hex(&hex)
            .map_err(|err| platform::error_with_context(err, "error instantiating NoteId from hex"))?;
        Ok(NoteId(native_note_id))
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteId> for NoteId {
    fn from(native_note_id: NativeNoteId) -> Self {
        NoteId(native_note_id)
    }
}

impl From<&NativeNoteId> for NoteId {
    fn from(native_note_id: &NativeNoteId) -> Self {
        NoteId(*native_note_id)
    }
}

impl From<NoteId> for NativeNoteId {
    fn from(note_id: NoteId) -> Self {
        note_id.0
    }
}

impl From<&NoteId> for NativeNoteId {
    fn from(note_id: &NoteId) -> Self {
        note_id.0
    }
}
