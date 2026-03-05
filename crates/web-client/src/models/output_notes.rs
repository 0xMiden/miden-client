use miden_client::transaction::OutputNotes as NativeOutputNotes;
use js_export_macro::js_export;

use super::output_note::OutputNote;
use super::word::Word;

/// Contains a list of output notes of a transaction. The list can be empty if the transaction does
/// not produce any notes.
#[derive(Clone)]
#[js_export]
pub struct OutputNotes(pub(crate) NativeOutputNotes);

#[js_export]
impl OutputNotes {
    /// Returns the commitment to all output notes.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns the number of notes emitted.
    #[js_export(js_name = "numNotes")]
    pub fn num_notes(&self) -> u32 {
        u32::try_from(self.0.num_notes())
            .expect("only 1024 output notes is allowed per transaction")
    }

    /// Returns true if there are no output notes.
    #[js_export(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the output note at the specified index.
    #[js_export(js_name = "getNote")]
    pub fn get_note(&self, index: u32) -> OutputNote {
        self.0.get_note(index as usize).into()
    }

    /// Returns all output notes as a vector.
    pub fn notes(&self) -> Vec<OutputNote> {
        self.0.iter().cloned().map(Into::into).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeOutputNotes> for OutputNotes {
    fn from(native_notes: NativeOutputNotes) -> Self {
        OutputNotes(native_notes)
    }
}

impl From<&NativeOutputNotes> for OutputNotes {
    fn from(native_notes: &NativeOutputNotes) -> Self {
        OutputNotes(native_notes.clone())
    }
}

impl_napi_from_value!(OutputNotes);
