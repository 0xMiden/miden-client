use js_export_macro::js_export;
use miden_client::assembly::Library as NativeLibrary;

#[js_export]
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
