use miden_client::note::NoteLocation as NativeNoteLocation;
use crate::prelude::*;

/// Contains information about the location of a note.
#[bindings]
#[derive(Clone)]
pub struct NoteLocation(NativeNoteLocation);

#[bindings]
impl NoteLocation {
    pub fn block_num(&self) -> u32 {
        self.0.block_num().as_u32()
    }

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
