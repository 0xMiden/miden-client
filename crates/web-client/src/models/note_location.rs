use miden_client::note::NoteLocation as NativeNoteLocation;
use wasm_bindgen::prelude::*;

/// Location of a note commitment within a block's note tree.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteLocation(NativeNoteLocation);

#[wasm_bindgen]
impl NoteLocation {
    /// Returns the block height containing the note.
    #[wasm_bindgen(js_name = "blockNum")]
    pub fn block_num(&self) -> u32 {
        self.0.block_num().as_u32()
    }

    /// Returns the index of the note leaf within the block's note tree.
    #[wasm_bindgen(js_name = "nodeIndexInBlock")]
    pub fn node_index_in_block(&self) -> u16 {
        self.0.node_index_in_block()
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
