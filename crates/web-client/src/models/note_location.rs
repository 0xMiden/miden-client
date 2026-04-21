use js_export_macro::js_export;
use miden_client::note::NoteLocation as NativeNoteLocation;

/// Contains information about the location of a note.
#[derive(Clone)]
#[js_export]
pub struct NoteLocation(NativeNoteLocation);

#[js_export]
impl NoteLocation {
    /// Returns the block height containing the note.
    #[js_export(js_name = "blockNum")]
    pub fn block_num(&self) -> u32 {
        self.0.block_num().as_u32()
    }

    /// Returns the index of the note leaf within the block's note tree.
    #[js_export(js_name = "blockNoteTreeIndex")]
    pub fn block_note_tree_index(&self) -> u16 {
        self.0.block_note_tree_index()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteLocation> for NoteLocation {
    fn from(native_location: NativeNoteLocation) -> Self {
        NoteLocation(native_location)
    }
}

impl From<&NativeNoteLocation> for NoteLocation {
    fn from(native_location: &NativeNoteLocation) -> Self {
        NoteLocation(native_location.clone())
    }
}
