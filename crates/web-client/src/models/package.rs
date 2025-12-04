use miden_client::vm::Package as NativePackage;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::library::Library;
use crate::models::package_manifest::PackageManifest;
use crate::models::program::Program;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[derive(Clone)]
#[wasm_bindgen]
pub struct Package(NativePackage);

#[wasm_bindgen]
impl Package {
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<Package, JsValue> {
        deserialize_from_uint8array::<NativePackage>(bytes).map(Package)
    }

    /// Returns the underlying library of a `Package`.
    /// Fails if the package is not a library.
    #[wasm_bindgen(js_name = "asLibrary")]
    pub fn as_library(&self) -> Result<Library, JsValue> {
        if !self.0.is_library() {
            return Err(JsValue::from_str("Package does not contain a library"));
        }

        let native_library = self.0.unwrap_library();
        Ok((*native_library).clone().into())
    }

    /// Returns the underlying program of a `Package`.
    /// Fails if the package is not a program.
    #[wasm_bindgen(js_name = "asProgram")]
    pub fn as_program(&self) -> Result<Program, JsValue> {
        if !self.0.is_program() {
            return Err(JsValue::from_str("Package does not contain a program"));
        }

        let native_program = self.0.unwrap_program();
        Ok((*native_program).clone().into())
    }

    pub fn manifest(&self) -> PackageManifest {
        self.0.manifest.clone().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativePackage> for Package {
    fn from(native_package: NativePackage) -> Self {
        Package(native_package)
    }
}

impl From<&NativePackage> for Package {
    fn from(native_package: &NativePackage) -> Self {
        Package(native_package.clone())
    }
}

impl From<Package> for NativePackage {
    fn from(package: Package) -> Self {
        package.0
    }
}

impl From<&Package> for NativePackage {
    fn from(package: &Package) -> Self {
        package.0.clone()
    }
}
