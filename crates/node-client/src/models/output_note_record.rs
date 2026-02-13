use miden_client::store::OutputNoteRecord as NativeOutputNoteRecord;

use super::note_id::NoteId;
use super::word::Word;

#[napi]
#[derive(Clone)]
pub struct OutputNoteRecord(pub(crate) NativeOutputNoteRecord);

#[napi]
impl OutputNoteRecord {
    /// Returns the note ID.
    #[napi]
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the recipient digest committed for the note.
    #[napi(js_name = "recipientDigest")]
    pub fn recipient_digest(&self) -> Word {
        self.0.recipient_digest().into()
    }

    /// Returns the expected block height for the note.
    #[napi(js_name = "expectedHeight")]
    pub fn expected_height(&self) -> u32 {
        self.0.expected_height().as_u32()
    }

    /// Returns the nullifier when the recipient is known.
    #[napi]
    pub fn nullifier(&self) -> Option<String> {
        self.0.nullifier().map(|n| n.to_hex())
    }

    /// Returns true if the note has been consumed on chain.
    #[napi(js_name = "isConsumed")]
    pub fn is_consumed(&self) -> bool {
        self.0.is_consumed()
    }

    /// Returns true if the note is committed on chain.
    #[napi(js_name = "isCommitted")]
    pub fn is_committed(&self) -> bool {
        self.0.is_committed()
    }
}

impl From<NativeOutputNoteRecord> for OutputNoteRecord {
    fn from(native: NativeOutputNoteRecord) -> Self {
        OutputNoteRecord(native)
    }
}
