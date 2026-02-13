use miden_client::store::OutputNoteRecord as NativeOutputNoteRecord;

use super::note_id::NoteId;
use super::word::Word;
use super::{napi_delegate, napi_wrap};

napi_wrap!(clone OutputNoteRecord wraps NativeOutputNoteRecord, one_way);

napi_delegate!(impl OutputNoteRecord {
    /// Returns the note ID.
    delegate id -> NoteId;
    /// Returns the recipient digest committed for the note.
    delegate recipient_digest -> Word;
    /// Returns true if the note has been consumed on chain.
    delegate is_consumed -> bool;
    /// Returns true if the note is committed on chain.
    delegate is_committed -> bool;
});

#[napi]
impl OutputNoteRecord {
    /// Returns the expected block height for the note.
    #[napi]
    pub fn expected_height(&self) -> u32 {
        self.0.expected_height().as_u32()
    }

    /// Returns the nullifier when the recipient is known.
    #[napi]
    pub fn nullifier(&self) -> Option<String> {
        self.0.nullifier().map(|n| n.to_hex())
    }
}
