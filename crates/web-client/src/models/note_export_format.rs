use miden_client::store::NoteExportType;
use js_export_macro::js_export;

#[js_export]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteExportFormat {
    Id,
    Full,
    Details,
}

// CONVERSIONS
// ================================================================================================

impl From<NoteExportFormat> for NoteExportType {
    fn from(value: NoteExportFormat) -> Self {
        match value {
            NoteExportFormat::Id => NoteExportType::NoteId,
            NoteExportFormat::Full => NoteExportType::NoteWithProof,
            NoteExportFormat::Details => NoteExportType::NoteDetails,
        }
    }
}
