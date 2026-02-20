use miden_client::assembly::Library as NativeLibrary;

use crate::prelude::*;

/// A compiled assembly library that can be linked into scripts or account components.
#[bindings]
#[derive(Clone)]
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
