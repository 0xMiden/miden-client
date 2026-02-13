use miden_client::note::NoteId as NativeNoteId;

use super::napi_wrap;
use super::word::Word;

napi_wrap!(copy NoteId wraps NativeNoteId);

#[napi]
impl NoteId {
    /// Creates a NoteId from a hex string.
    #[napi(js_name = "fromHex")]
    pub fn from_hex(hex: String) -> napi::Result<NoteId> {
        let word = Word::from_hex(hex)?;
        Ok(NoteId(NativeNoteId::from_raw(word.0)))
    }

    /// Returns the hex representation.
    #[napi(js_name = "toString")]
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}
