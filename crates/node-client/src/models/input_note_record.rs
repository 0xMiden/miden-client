use miden_client::store::InputNoteRecord as NativeInputNoteRecord;

use super::napi_wrap;
use super::note_id::NoteId;

napi_wrap!(clone InputNoteRecord wraps NativeInputNoteRecord, one_way);

#[napi]
impl InputNoteRecord {
    /// Returns the note ID.
    #[napi]
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the consumer transaction ID if the note has been consumed.
    #[napi(js_name = "consumerTransactionId")]
    pub fn consumer_transaction_id(&self) -> Option<String> {
        self.0.consumer_transaction_id().map(ToString::to_string)
    }

    /// Returns the nullifier for this note.
    #[napi]
    pub fn nullifier(&self) -> String {
        self.0.nullifier().to_hex()
    }

    /// Returns true if the record contains authentication data (proof).
    #[napi(js_name = "isAuthenticated")]
    pub fn is_authenticated(&self) -> bool {
        self.0.is_authenticated()
    }

    /// Returns true if the note has already been consumed.
    #[napi(js_name = "isConsumed")]
    pub fn is_consumed(&self) -> bool {
        self.0.is_consumed()
    }

    /// Returns true if the note is currently being processed.
    #[napi(js_name = "isProcessing")]
    pub fn is_processing(&self) -> bool {
        self.0.is_processing()
    }
}
