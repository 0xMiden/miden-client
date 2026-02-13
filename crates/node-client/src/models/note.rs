use miden_client::note::Note as NativeNote;
use miden_client::utils::{Deserializable, Serializable};
use napi::bindgen_prelude::*;

use super::napi_wrap;
use super::note_id::NoteId;

napi_wrap!(clone Note wraps NativeNote);

#[napi]
impl Note {
    /// Returns the note ID.
    #[napi]
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Serializes the note into bytes.
    #[napi]
    pub fn serialize(&self) -> Buffer {
        self.0.to_bytes().into()
    }

    /// Restores a note from its serialized bytes.
    #[napi]
    pub fn deserialize(bytes: Buffer) -> Result<Note> {
        let native = NativeNote::read_from_bytes(&bytes).map_err(|err| {
            napi::Error::from_reason(format!("Failed to deserialize Note: {err}"))
        })?;
        Ok(Note(native))
    }
}
