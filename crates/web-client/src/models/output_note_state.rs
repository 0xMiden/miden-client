use miden_client::store::OutputNoteState as NativeOutputNoteState;
use js_export_macro::js_export;

#[derive(Clone)]
#[js_export]
pub enum OutputNoteState {
    ExpectedPartial,
    ExpectedFull,
    CommittedPartial,
    CommittedFull,
    Consumed,
}

// CONVERSIONS
// ================================================================================================

impl From<NativeOutputNoteState> for OutputNoteState {
    fn from(native_note: NativeOutputNoteState) -> Self {
        match native_note {
            NativeOutputNoteState::ExpectedPartial => OutputNoteState::ExpectedPartial,
            NativeOutputNoteState::ExpectedFull { .. } => OutputNoteState::ExpectedFull,
            NativeOutputNoteState::CommittedPartial { .. } => OutputNoteState::CommittedPartial,
            NativeOutputNoteState::CommittedFull { .. } => OutputNoteState::CommittedFull,
            NativeOutputNoteState::Consumed { .. } => OutputNoteState::Consumed,
        }
    }
}

impl From<&NativeOutputNoteState> for OutputNoteState {
    fn from(native_note: &NativeOutputNoteState) -> Self {
        match native_note {
            NativeOutputNoteState::ExpectedPartial => OutputNoteState::ExpectedPartial,
            NativeOutputNoteState::ExpectedFull { .. } => OutputNoteState::ExpectedFull,
            NativeOutputNoteState::CommittedPartial { .. } => OutputNoteState::CommittedPartial,
            NativeOutputNoteState::CommittedFull { .. } => OutputNoteState::CommittedFull,
            NativeOutputNoteState::Consumed { .. } => OutputNoteState::Consumed,
        }
    }
}
