use miden_objects::assembly::{Library as NativeLibrary, QualifiedProcedureName as NativeQualifiedProcedureName};
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;

#[wasm_bindgen]
pub struct Library(NativeLibrary);

// CONVERSIONS
// ================================================================================================

impl From<NativeLibrary> for Library {
    fn from(native_library: NativeLibrary) -> Self {
        Library(native_library)
    }
}

impl From<&NativeLibrary> for Library {
    fn from(native_library: &NativeLibrary) -> Self {
        Library(native_library.clone())
    }
}

impl From<Library> for NativeLibrary {
    fn from(library: Library) -> Self {
        library.0
    }
}

impl From<&Library> for NativeLibrary {
    fn from(library: &Library) -> Self {
        library.0.clone()
    }
}
